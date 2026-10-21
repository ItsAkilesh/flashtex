//! The files a compile wrote, written out to the disk on another thread
//! between keystrokes (lane T7-HEADROOM, experiment).
//!
//! On the M5 Pro's APFS, cutting a file shorter (`ftruncate`) right after a
//! write to it blocked for 0.35-0.8 ms: a deleted letter's `apply_changes`
//! (reference run 2026-10-21d). Belief under test: the cut first writes out
//! the file's dirty pages, so a file whose pages are already clean is cut at
//! once. `FLASHTEX_FLUSH=src` writes out the sources the host wrote, `all`
//! also the output directory's files written in the last 10 s (16 MB at
//! most each); `FLASHTEX_FLUSH_AB=1` does it after every other pair of
//! compiles only, so that one run measures both.

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::mpsc;
use std::sync::{Mutex, OnceLock};

/// Flushes asked for and finished, and the last one's time (µs).
static ISSUED: AtomicU64 = AtomicU64::new(0);
static DONE: AtomicU64 = AtomicU64::new(0);
static LAST_US: AtomicU64 = AtomicU64::new(0);

/// 0: off; 1: the sources; 2: and the outputs.
pub fn mode() -> u8 {
    static M: OnceLock<u8> = OnceLock::new();
    *M.get_or_init(|| match std::env::var("FLASHTEX_FLUSH").as_deref() {
        Ok("src") => 1,
        Ok("all") => 2,
        _ => 0,
    })
}

fn ab() -> bool {
    static M: OnceLock<bool> = OnceLock::new();
    *M.get_or_init(|| std::env::var_os("FLASHTEX_FLUSH_AB").is_some())
}

fn sender() -> &'static Mutex<mpsc::Sender<Vec<PathBuf>>> {
    static S: OnceLock<Mutex<mpsc::Sender<Vec<PathBuf>>>> = OnceLock::new();
    S.get_or_init(|| {
        let (tx, rx) = mpsc::channel::<Vec<PathBuf>>();
        std::thread::Builder::new()
            .name("flashtex-flush".into())
            .spawn(move || {
                while let Ok(paths) = rx.recv() {
                    let t = std::time::Instant::now();
                    for p in paths {
                        if let Ok(f) = std::fs::File::open(&p) {
                            sync(&f);
                        }
                    }
                    LAST_US.store(t.elapsed().as_micros() as u64, Ordering::Relaxed);
                    DONE.fetch_add(1, Ordering::Release);
                }
            })
            .expect("the flush thread");
        Mutex::new(tx)
    })
}

/// `fsync`: the file's dirty pages to the disk (not `F_FULLFSYNC`, which
/// `File::sync_all` is on macOS and which also empties the drive's cache).
fn sync(f: &std::fs::File) {
    #[cfg(unix)]
    {
        use std::os::fd::AsRawFd;
        // SAFETY: a valid descriptor of an open file.
        unsafe { libc::fsync(f.as_raw_fd()) };
    }
    #[cfg(not(unix))]
    {
        let _ = f.sync_data();
    }
}

/// After compile number `n` (the document's): write out `sources` and, in
/// mode 2, the output directory's recent files. Returns whether it did.
pub fn after_compile(n: u64, sources: Vec<PathBuf>, out_dir: &Path) -> bool {
    let m = mode();
    if m == 0 || (ab() && (n / 2) % 2 == 1) {
        return false;
    }
    let mut paths = sources;
    if m == 2 {
        if let Ok(rd) = std::fs::read_dir(out_dir) {
            for e in rd.flatten() {
                let Ok(md) = e.metadata() else { continue };
                let recent = md
                    .modified()
                    .ok()
                    .and_then(|t| t.elapsed().ok())
                    .is_some_and(|d| d.as_secs() < 10);
                if md.is_file() && recent && md.len() <= 16 << 20 {
                    paths.push(e.path());
                }
            }
        }
    }
    if paths.is_empty() {
        return false;
    }
    if let Ok(s) = sender().lock() {
        if s.send(paths).is_ok() {
            ISSUED.fetch_add(1, Ordering::Release);
            return true;
        }
    }
    false
}

/// For the next compile's record: (flushes asked for, finished, the last
/// one's µs).
pub fn state() -> (u64, u64, u64) {
    (
        ISSUED.load(Ordering::Acquire),
        DONE.load(Ordering::Acquire),
        LAST_US.load(Ordering::Relaxed),
    )
}
