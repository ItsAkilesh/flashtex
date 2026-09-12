//! Atomic saves: write to a temporary file in the same directory, fsync,
//! rename over the target, fsync the directory. Existing permissions are
//! preserved. A save is refused when the file on disk no longer matches the
//! hash the caller last observed, unless `force` is set.

use std::fmt;
use std::fs::{self, File, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::SystemTime;

use crate::path::ProjectPath;
use crate::sha256::{Digest, hex, sha256};

/// Proof of a completed save.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SaveReceipt {
    pub path: ProjectPath,
    pub bytes: u64,
    pub sha256: Digest,
    /// Modification time reported by the filesystem after the rename.
    pub mtime: SystemTime,
}

impl SaveReceipt {
    pub fn sha256_hex(&self) -> String {
        hex(&self.sha256)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SaveConflictKind {
    /// The file exists with content different from `expected`.
    ModifiedExternally,
    /// `expected` was given but the file no longer exists.
    DeletedExternally,
    /// No `expected` hash (new file) but a file already exists there.
    AlreadyExists,
}

/// Why a non-forced save was refused.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SaveConflict {
    pub path: ProjectPath,
    pub kind: SaveConflictKind,
    /// The hash the caller expected on disk (`None` for a new file).
    pub ours: Option<Digest>,
    /// The hash actually on disk (`None` if deleted).
    pub theirs: Option<Digest>,
    pub mtime: Option<SystemTime>,
    pub size: Option<u64>,
}

#[derive(Debug)]
pub enum SaveError {
    Conflict(Box<SaveConflict>),
    Io(io::Error),
}

impl fmt::Display for SaveError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SaveError::Conflict(c) => write!(f, "save of {} refused: {:?}", c.path, c.kind),
            SaveError::Io(e) => write!(f, "save failed: {e}"),
        }
    }
}

impl std::error::Error for SaveError {}

impl From<io::Error> for SaveError {
    fn from(e: io::Error) -> Self {
        SaveError::Io(e)
    }
}

/// What the caller believes is on disk.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Expected {
    /// The path should not exist yet.
    NewFile,
    /// The path should contain bytes hashing to this digest.
    Hash(Digest),
    /// Do not check (equivalent to `force`).
    Any,
}

/// Saves `text` to `<root>/<path>` atomically.
///
/// With `force == false`, the on-disk state must match `expected`; otherwise a
/// [`SaveConflict`] is returned and nothing is written. With `force == true`
/// the check is skipped. Parent directories are created as needed.
pub fn save_atomic(
    root: &Path,
    path: &ProjectPath,
    text: &str,
    expected: Expected,
    force: bool,
) -> Result<SaveReceipt, SaveError> {
    save_atomic_bytes(root, path, text.as_bytes(), expected, force)
}

pub fn save_atomic_bytes(
    root: &Path,
    path: &ProjectPath,
    bytes: &[u8],
    expected: Expected,
    force: bool,
) -> Result<SaveReceipt, SaveError> {
    let target = path.to_os_path(root);
    if !force {
        check_expected(path, &target, expected)?;
    }
    let receipt = write_atomic(&target, bytes)?;
    Ok(SaveReceipt {
        path: path.clone(),
        bytes: receipt.0,
        sha256: receipt.1,
        mtime: receipt.2,
    })
}

fn check_expected(path: &ProjectPath, target: &Path, expected: Expected) -> Result<(), SaveError> {
    let current = match fs::read(target) {
        Ok(b) => Some(b),
        Err(e) if e.kind() == io::ErrorKind::NotFound => None,
        Err(e) => return Err(SaveError::Io(e)),
    };
    let (kind, ours) = match (expected, &current) {
        (Expected::Any, _) => return Ok(()),
        (Expected::NewFile, None) => return Ok(()),
        (Expected::NewFile, Some(_)) => (SaveConflictKind::AlreadyExists, None),
        (Expected::Hash(h), None) => (SaveConflictKind::DeletedExternally, Some(h)),
        (Expected::Hash(h), Some(bytes)) => {
            if sha256(bytes) == h {
                return Ok(());
            }
            (SaveConflictKind::ModifiedExternally, Some(h))
        }
    };
    let meta = fs::metadata(target).ok();
    Err(SaveError::Conflict(Box::new(SaveConflict {
        path: path.clone(),
        kind,
        ours,
        theirs: current.as_deref().map(sha256),
        mtime: meta.as_ref().and_then(|m| m.modified().ok()),
        size: meta.as_ref().map(|m| m.len()),
    })))
}

static TEMP_COUNTER: AtomicU64 = AtomicU64::new(0);

/// Writes `bytes` to `target` via temp file + fsync + rename in the same
/// directory. Returns `(bytes, sha256, mtime)`.
pub(crate) fn write_atomic(target: &Path, bytes: &[u8]) -> io::Result<(u64, Digest, SystemTime)> {
    let dir = target
        .parent()
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "target has no parent"))?;
    fs::create_dir_all(dir)?;
    let name = target
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("file");
    let temp = unique_temp_path(dir, name)?;

    let existing_mode = fs::metadata(target).ok().map(|m| m.permissions());
    let result = (|| -> io::Result<()> {
        let mut f = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temp)?;
        f.write_all(bytes)?;
        f.sync_all()?;
        if let Some(perms) = existing_mode {
            f.set_permissions(perms)?;
        }
        drop(f);
        fs::rename(&temp, target)?;
        Ok(())
    })();
    if let Err(e) = result {
        let _ = fs::remove_file(&temp);
        return Err(e);
    }
    // Make the rename durable. Directory fsync is best effort: some
    // filesystems reject it, and the rename itself is already complete.
    if let Ok(d) = File::open(dir) {
        let _ = d.sync_all();
    }
    let meta = fs::metadata(target)?;
    Ok((meta.len(), sha256(bytes), meta.modified()?))
}

fn unique_temp_path(dir: &Path, name: &str) -> io::Result<PathBuf> {
    let pid = std::process::id();
    for _ in 0..1000 {
        let n = TEMP_COUNTER.fetch_add(1, Ordering::Relaxed);
        let candidate = dir.join(format!(".{name}.flashtex-tmp-{pid}-{n}"));
        if !candidate.exists() {
            return Ok(candidate);
        }
    }
    Err(io::Error::new(
        io::ErrorKind::AlreadyExists,
        "could not allocate a temporary file name",
    ))
}
