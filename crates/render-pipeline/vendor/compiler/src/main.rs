//! JSON Lines worker. Requests on stdin, replies on stdout, logs on stderr.

use flashtex_compiler::json;
use flashtex_compiler::protocol::{self, RequestLine};
use std::io::{self, Write};

fn main() {
    let stdin = io::stdin();
    let stdout = io::stdout();
    let mut out = stdout.lock();

    let mut input = stdin.lock();
    loop {
        let reply = match protocol::read_request_line(&mut input) {
            Ok(Some(RequestLine::Data(bytes))) => match std::str::from_utf8(&bytes) {
                Ok(line) if line.trim().is_empty() => continue,
                _ => protocol::handle_request_bytes(&bytes),
            },
            Ok(Some(RequestLine::TooLarge)) => json::write(&protocol::error_envelope(
                "",
                "payload_too_large",
                &format!("line exceeds the {}-byte limit", protocol::MAX_LINE_BYTES),
            )),
            Ok(None) => break,
            Err(e) => {
                eprintln!("flashtex-compiler: read error: {}", e);
                break;
            }
        };
        if writeln!(out, "{}", reply).is_err() {
            break;
        }
        let _ = out.flush();
    }
}
