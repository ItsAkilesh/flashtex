//! The host's resident engine (`crate::resident`): runs of the engine on an
//! engine thread of this process, instead of a child process per run
//! (docs/design/xetex/INCREMENTAL.md §5 step 2).
//!
//! The engine's display list comes to the compile through a socket pair, so
//! a run's pages go through `compile::forward` as a child's do (the
//! client's fonts and accepted items), and are sent as they are shipped.
//! A cancel sets the run's flag, which the engine reads between commands,
//! at macro calls, lines and pages (`flashtex_poll`).
//!
//! The engine's runtime is per process (`flashtex_engine::system`), so
//! there is one engine thread in the process, and the connections' runs
//! take turns on it ([`Shared::run`]).
//!
//! `FLASHTEX_UNICODE_INPROCESS=0` runs every compile in a child process
//! instead, with hot spares (`spare.rs`), as before.

use crate::fontmgr::FontCatalog;
use crate::resident::{Engine, Fonts, Report, Spec};
use std::path::PathBuf;
use std::sync::atomic::AtomicBool;
use std::sync::mpsc;
use std::sync::{Arc, Mutex};

/// Whether compiles run in this process (the default).
pub fn enabled() -> bool {
    std::env::var("FLASHTEX_UNICODE_INPROCESS").map_or(true, |v| v != "0")
}

/// One run for the engine thread, and where its report goes.
struct Req {
    spec: Spec,
    reply: mpsc::Sender<Report>,
}

/// The engine thread, which every connection shares.
pub struct Shared {
    tx: Mutex<Option<mpsc::Sender<Req>>>,
    /// Held for a whole run: one run at a time.
    turn: Mutex<()>,
}

impl Default for Shared {
    fn default() -> Self {
        Shared {
            tx: Mutex::new(None),
            turn: Mutex::new(()),
        }
    }
}

type Dirs = Vec<(PathBuf, Option<super::spare::Ident>)>;

/// The fonts' catalog, kept while the font directories are unchanged (the
/// rule `spare.rs` keeps a spare's catalog by, which is fontconfig's own):
/// the directories are read first, then the catalog made from them.
///
/// The directories are walked once, and after that each one's identity is
/// checked (`spare::font_dirs_unchanged`: a status call a directory, no
/// listing): the walk itself took 4-12 ms on macOS, at every keystroke of a
/// document that names a font.
#[derive(Default)]
struct KeptFonts {
    /// The last walk of the directories.
    walked: Option<Dirs>,
    /// The catalog made from them, and the walk it was made after.
    kept: Option<(Dirs, Arc<FontCatalog>)>,
}

impl KeptFonts {
    /// The font directories as they are now.
    fn now(&mut self) -> &Dirs {
        let same = self
            .walked
            .as_ref()
            .is_some_and(|w| super::spare::font_dirs_unchanged(w));
        if !same {
            self.walked = Some(super::spare::font_dirs());
        }
        self.walked.as_ref().expect("walked above")
    }
}

impl Fonts for KeptFonts {
    fn catalog(&mut self) -> Arc<FontCatalog> {
        let dirs = self.now().clone();
        if let Some((d, c)) = &self.kept {
            if *d == dirs {
                return c.clone();
            }
        }
        let c = Arc::new(FontCatalog::for_lookup());
        self.kept = Some((dirs, c.clone()));
        c
    }

    fn dirs(&mut self) -> [u64; 2] {
        let d = format!("{:?}", self.now());
        flashtex_engine::persist::hash128(d.as_bytes())
    }
}

/// The engine thread's scheduling: as the pdfTeX host's engine thread
/// (`FLASHTEX_HOST_QOS=default` leaves it), user-interactive on macOS, so
/// that a keystroke's run is not put on the efficiency cores.
fn interactive_qos() {
    #[cfg(target_os = "macos")]
    {
        if std::env::var("FLASHTEX_HOST_QOS").as_deref() == Ok("default") {
            return;
        }
        extern "C" {
            fn pthread_set_qos_class_self_np(qos_class: u32, relative_priority: i32) -> i32;
        }
        // <sys/qos.h>: QOS_CLASS_USER_INTERACTIVE
        const QOS_CLASS_USER_INTERACTIVE: u32 = 0x21;
        // SAFETY: sets the calling thread's own scheduling class.
        unsafe {
            pthread_set_qos_class_self_np(QOS_CLASS_USER_INTERACTIVE, 0);
        }
    }
}

impl Shared {
    /// The engine thread's queue, started on first use.
    fn sender(&self) -> Result<mpsc::Sender<Req>, String> {
        let mut tx = self.tx.lock().unwrap();
        if let Some(t) = tx.as_ref() {
            return Ok(t.clone());
        }
        let (t, rx) = mpsc::channel::<Req>();
        std::thread::Builder::new()
            .name("unicode-engine".into())
            // the recursion TeX may need (the pdfTeX host's engine thread's)
            .stack_size(512 << 20)
            .spawn(move || {
                interactive_qos();
                let clock = flashtex_engine::host::pin_clock_from_env();
                let mut engine = Engine::new(clock, Some(Box::new(KeptFonts::default())));
                for req in rx {
                    let rep = engine.run(req.spec);
                    let _ = req.reply.send(rep);
                }
            })
            .map_err(|e| format!("the engine thread: {e}"))?;
        *tx = Some(t.clone());
        Ok(t)
    }

    /// Run `spec` on the engine thread; `during` runs on this thread while
    /// the engine does (it reads the display list). Runs take turns.
    pub fn run<T>(&self, spec: Spec, during: impl FnOnce() -> T) -> Result<(Report, T), String> {
        let _turn = self.turn.lock().unwrap_or_else(|p| p.into_inner());
        let tx = self.sender()?;
        let (reply, rx) = mpsc::channel();
        tx.send(Req { spec, reply })
            .map_err(|_| "the engine thread has ended".to_string())?;
        let t = during();
        let rep = rx
            .recv()
            .map_err(|_| "the engine thread has ended".to_string())?;
        Ok((rep, t))
    }
}

/// A new run's cancel flag.
pub fn cancel_flag() -> Arc<AtomicBool> {
    Arc::new(AtomicBool::new(false))
}
