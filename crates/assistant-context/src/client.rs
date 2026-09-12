//! Unix pipe client for a native background worker, never the UI thread.
use serde_json::{json, Value};
use std::{
    io::{Read, Write},
    os::fd::AsRawFd,
    path::Path,
    process::{Child, ChildStdin, ChildStdout, Command, Stdio},
    time::{Duration, Instant},
};

pub struct SessionClient {
    child: Child,
    input: Option<ChildStdin>,
    output: Option<ChildStdout>,
    sequence: u64,
}
fn nonblocking(fd: &impl AsRawFd) -> Result<(), String> {
    // SAFETY: borrowed live descriptor; F_GETFL/F_SETFL do not take pointer args.
    let flags = unsafe { libc::fcntl(fd.as_raw_fd(), libc::F_GETFL) };
    if flags < 0
        || unsafe { libc::fcntl(fd.as_raw_fd(), libc::F_SETFL, flags | libc::O_NONBLOCK) } < 0
    {
        return Err(std::io::Error::last_os_error().to_string());
    }
    Ok(())
}
impl SessionClient {
    /// Launch only the trusted helper executable, without a shell. stderr is
    /// discarded so diagnostics cannot fill a pipe. Use a fresh session ID.
    pub fn spawn(executable: &Path, session: &str) -> Result<Self, String> {
        crate::ExplanationRegistry::new(session.to_owned(), 8, 64)?;
        let mut command = Command::new(executable);
        command
            .args(["--session", session])
            .env_remove("FLASHTEX_GROK_API_KEY");
        Self::spawn_command(command)
    }
    /// Explicit provider helper startup. Credentials go only into this child's
    /// environment, never argv or request JSON. Startup performs no inference.
    #[cfg(feature = "grok")]
    pub fn spawn_provider(
        executable: &Path,
        session: &str,
        model: &str,
        key: &str,
    ) -> Result<Self, String> {
        crate::ExplanationRegistry::new(format!("provider_{session}"), 8, 64)?;
        if key.is_empty() || key.len() > 8192 || key.bytes().any(|b| b.is_ascii_control()) {
            return Err("invalid provider credential".into());
        }
        if model.is_empty()
            || model.len() > 128
            || !model
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b"-._".contains(&b))
        {
            return Err("invalid provider model".into());
        }
        let mut command = Command::new(executable);
        command
            .args(["--provider-session", session, model])
            .env("FLASHTEX_GROK_API_KEY", key);
        Self::spawn_command(command)
    }
    fn spawn_command(mut command: Command) -> Result<Self, String> {
        let mut child = command
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|e| e.to_string())?;
        let input = child.stdin.take().expect("piped stdin");
        let output = child.stdout.take().expect("piped stdout");
        if let Err(error) = nonblocking(&input).and_then(|_| nonblocking(&output)) {
            let _ = child.kill();
            let _ = child.wait();
            return Err(error);
        }
        Ok(Self {
            child,
            input: Some(input),
            output: Some(output),
            sequence: 0,
        })
    }
    fn stop(&mut self) {
        self.input.take();
        self.output.take();
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
    pub fn is_stopped(&self) -> bool {
        self.input.is_none()
    }
    fn exchange(&mut self, bytes: &[u8], deadline: Instant) -> Result<Value, String> {
        let mut written = 0;
        let mut frame = Vec::new();
        loop {
            if Instant::now() >= deadline {
                return Err("helper IO deadline exceeded".into());
            }
            if written < bytes.len() {
                match self
                    .input
                    .as_mut()
                    .ok_or("helper stopped")?
                    .write(&bytes[written..])
                {
                    Ok(0) => return Err("helper input closed".into()),
                    Ok(count) => written += count,
                    Err(e)
                        if matches!(
                            e.kind(),
                            std::io::ErrorKind::WouldBlock | std::io::ErrorKind::Interrupted
                        ) => {}
                    Err(e) => return Err(e.to_string()),
                }
            }
            let mut buffer = [0; 8192];
            match self
                .output
                .as_mut()
                .ok_or("helper stopped")?
                .read(&mut buffer)
            {
                Ok(0) => return Err("helper output closed before complete reply".into()),
                Ok(count) => {
                    frame.extend_from_slice(&buffer[..count]);
                    if frame.len() > 128 * 1024 {
                        return Err("helper output exceeds128KiB".into());
                    }
                    if let Some(end) = frame.iter().position(|b| *b == b'\n') {
                        if end + 1 != frame.len() || written != bytes.len() {
                            return Err("helper sent unsolicited or multiple replies".into());
                        }
                        return serde_json::from_slice(&frame).map_err(|e| e.to_string());
                    }
                }
                Err(e)
                    if matches!(
                        e.kind(),
                        std::io::ErrorKind::WouldBlock | std::io::ErrorKind::Interrupted
                    ) => {}
                Err(e) => return Err(e.to_string()),
            }
            std::thread::sleep(
                Duration::from_millis(1).min(deadline.saturating_duration_since(Instant::now())),
            );
        }
    }
    /// IO/protocol failure permanently stops this client. Restart explicitly
    /// with a fresh session; never automatically replay uncertain requests.
    pub fn call(&mut self, action: Value, timeout: Duration) -> Result<Value, String> {
        if timeout.is_zero() || timeout > Duration::from_secs(120) {
            return Err("invalid IO timeout".into());
        }
        if self.is_stopped() {
            return Err("helper is stopped".into());
        }
        let deadline = Instant::now() + timeout;
        self.sequence = self
            .sequence
            .checked_add(1)
            .ok_or("command sequence exhausted")?;
        let id = self.sequence.to_string();
        let mut bytes =
            serde_json::to_vec(&json!({"id":id,"action":action})).map_err(|e| e.to_string())?;
        if bytes.len() >= 16 * 1024 * 1024 {
            return Err("helper command exceeds frame limit".into());
        }
        bytes.push(b'\n');
        let result = self.exchange(&bytes, deadline).and_then(|response| {
            let valid = response.as_object().is_some_and(|obj| {
                obj.len() == 2
                    && response["id"].as_str() == Some(&id)
                    && ((obj.contains_key("result") && response["result"].is_object())
                        || (obj.contains_key("error") && response["error"].is_string()))
            });
            if valid {
                Ok(response)
            } else {
                Err("helper response correlation/envelope mismatch".into())
            }
        });
        if result.is_err() {
            self.stop();
        }
        result
    }
}
impl Drop for SessionClient {
    fn drop(&mut self) {
        self.stop();
    }
}
