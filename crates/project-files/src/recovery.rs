//! Crash recovery journal: unsaved buffers are written atomically to
//! `<root>/.flashtex/recovery/<sha256(path)>.json` with the text, the hash of
//! the on-disk base they were edited from, and a timestamp. After a crash the
//! journal can be listed, checked against the current file, restored, or
//! discarded. Restoring never silently overwrites a file that moved on from
//! the recorded base.

use std::fmt;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use crate::json::Json;
use crate::path::ProjectPath;
use crate::save::{Expected, SaveError, SaveReceipt, save_atomic, write_atomic};
use crate::sha256::{Digest, hex, parse_hex, sha256, sha256_hex};

pub const RECOVERY_DIR: &str = ".flashtex/recovery";
const SCHEMA_VERSION: u64 = 1;

/// One journaled buffer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecoveryEntry {
    pub path: ProjectPath,
    pub text: String,
    pub text_sha256: Digest,
    /// Hash of the on-disk content the buffer was edited from (`None` for a
    /// file that did not exist yet).
    pub base_sha256: Option<Digest>,
    pub saved_at: SystemTime,
    /// The journal file this entry was read from or written to.
    pub journal_file: PathBuf,
}

/// Relationship between the current on-disk file and a journal entry.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CurrentState {
    /// The file does not exist on disk.
    Missing,
    /// Disk still equals the recorded base: restoring loses nothing.
    MatchesBase,
    /// Disk already equals the journaled text: nothing to restore.
    MatchesJournal,
    /// Disk differs from both: restoring would overwrite someone's work.
    Diverged(Digest),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RestoreCheck {
    pub current: CurrentState,
    /// True when a non-forced restore will succeed.
    pub safe: bool,
}

#[derive(Debug)]
pub enum RecoveryError {
    Io(io::Error),
    Malformed { file: PathBuf, message: String },
    Save(SaveError),
}

impl fmt::Display for RecoveryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RecoveryError::Io(e) => write!(f, "recovery journal I/O error: {e}"),
            RecoveryError::Malformed { file, message } => {
                write!(f, "malformed journal file {}: {message}", file.display())
            }
            RecoveryError::Save(e) => write!(f, "restore failed: {e}"),
        }
    }
}

impl std::error::Error for RecoveryError {}

impl From<io::Error> for RecoveryError {
    fn from(e: io::Error) -> Self {
        RecoveryError::Io(e)
    }
}

impl From<SaveError> for RecoveryError {
    fn from(e: SaveError) -> Self {
        RecoveryError::Save(e)
    }
}

/// Journal listing: readable entries sorted by path, plus files that could
/// not be parsed (kept on disk for manual inspection).
#[derive(Debug, Default)]
pub struct Listing {
    pub entries: Vec<RecoveryEntry>,
    pub malformed: Vec<(PathBuf, String)>,
}

#[derive(Debug, Clone)]
pub struct RecoveryJournal {
    root: PathBuf,
}

impl RecoveryJournal {
    pub fn new(root: &Path) -> Self {
        Self {
            root: root.to_path_buf(),
        }
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn dir(&self) -> PathBuf {
        self.root.join(RECOVERY_DIR)
    }

    pub fn journal_file(&self, path: &ProjectPath) -> PathBuf {
        self.dir()
            .join(format!("{}.json", sha256_hex(path.as_str().as_bytes())))
    }

    /// Atomically records `text` for `path`. Replaces any previous entry.
    pub fn record(
        &self,
        path: &ProjectPath,
        text: &str,
        base_sha256: Option<Digest>,
    ) -> Result<RecoveryEntry, RecoveryError> {
        let now = SystemTime::now();
        let millis = now
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0);
        let text_sha256 = sha256(text.as_bytes());
        let mut doc = Json::object();
        doc.insert("schema_version", SCHEMA_VERSION)
            .insert("path", path.as_str())
            .insert("text", text)
            .insert("text_sha256", hex(&text_sha256))
            .insert("base_sha256", base_sha256.as_ref().map(hex))
            .insert("saved_at_unix_ms", millis);
        let file = self.journal_file(path);
        write_atomic(&file, doc.to_string_compact().as_bytes())?;
        Ok(RecoveryEntry {
            path: path.clone(),
            text: text.to_string(),
            text_sha256,
            base_sha256,
            saved_at: UNIX_EPOCH + Duration::from_millis(millis),
            journal_file: file,
        })
    }

    /// Loads the entry for `path`, if any.
    pub fn load(&self, path: &ProjectPath) -> Result<Option<RecoveryEntry>, RecoveryError> {
        let file = self.journal_file(path);
        match fs::read(&file) {
            Ok(bytes) => parse_entry(&file, &bytes).map(Some),
            Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    /// Lists all entries. Files that are not valid entries are reported, not
    /// deleted.
    pub fn list(&self) -> Result<Listing, RecoveryError> {
        let mut listing = Listing::default();
        let dir = self.dir();
        let read_dir = match fs::read_dir(&dir) {
            Ok(r) => r,
            Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(listing),
            Err(e) => return Err(e.into()),
        };
        for entry in read_dir {
            let entry = entry?;
            let file = entry.path();
            let is_json = file.extension().is_some_and(|e| e == "json");
            let is_temp = file
                .file_name()
                .and_then(|n| n.to_str())
                .is_some_and(|n| n.starts_with('.'));
            if !is_json || is_temp || !entry.file_type()?.is_file() {
                continue;
            }
            match fs::read(&file)
                .map_err(RecoveryError::from)
                .and_then(|b| parse_entry(&file, &b))
            {
                Ok(e) => listing.entries.push(e),
                Err(RecoveryError::Malformed { file, message }) => {
                    listing.malformed.push((file, message))
                }
                Err(RecoveryError::Io(e)) => listing.malformed.push((file, e.to_string())),
                Err(RecoveryError::Save(e)) => listing.malformed.push((file, e.to_string())),
            }
        }
        listing.entries.sort_by(|a, b| a.path.cmp(&b.path));
        Ok(listing)
    }

    /// Compares the current on-disk file with the entry's base and text.
    pub fn check(&self, entry: &RecoveryEntry) -> Result<RestoreCheck, RecoveryError> {
        let current = match fs::read(entry.path.to_os_path(&self.root)) {
            Ok(bytes) => {
                let h = sha256(&bytes);
                if h == entry.text_sha256 {
                    CurrentState::MatchesJournal
                } else if Some(h) == entry.base_sha256 {
                    CurrentState::MatchesBase
                } else {
                    CurrentState::Diverged(h)
                }
            }
            Err(e) if e.kind() == io::ErrorKind::NotFound => CurrentState::Missing,
            Err(e) => return Err(e.into()),
        };
        let safe = match current {
            CurrentState::MatchesBase | CurrentState::MatchesJournal => true,
            CurrentState::Missing => entry.base_sha256.is_none(),
            CurrentState::Diverged(_) => false,
        };
        Ok(RestoreCheck { current, safe })
    }

    /// Writes the journaled text to the project file with the same conflict
    /// rules as [`save_atomic`]: the disk must still match the recorded base
    /// (or be absent for a new file) unless `force`. On success the entry is
    /// discarded. A file that already equals the journal text is treated as
    /// restored without rewriting.
    pub fn restore_to_disk(
        &self,
        entry: &RecoveryEntry,
        force: bool,
    ) -> Result<Option<SaveReceipt>, RecoveryError> {
        let check = self.check(entry)?;
        if check.current == CurrentState::MatchesJournal {
            self.discard(&entry.path)?;
            return Ok(None);
        }
        let expected = match entry.base_sha256 {
            Some(h) => Expected::Hash(h),
            None => Expected::NewFile,
        };
        let receipt = save_atomic(&self.root, &entry.path, &entry.text, expected, force)?;
        self.discard(&entry.path)?;
        Ok(Some(receipt))
    }

    /// Removes the entry for `path`. Returns whether one existed.
    pub fn discard(&self, path: &ProjectPath) -> Result<bool, RecoveryError> {
        match fs::remove_file(self.journal_file(path)) {
            Ok(()) => Ok(true),
            Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(false),
            Err(e) => Err(e.into()),
        }
    }
}

fn parse_entry(file: &Path, bytes: &[u8]) -> Result<RecoveryEntry, RecoveryError> {
    let malformed = |message: String| RecoveryError::Malformed {
        file: file.to_path_buf(),
        message,
    };
    let text = std::str::from_utf8(bytes).map_err(|_| malformed("not UTF-8".into()))?;
    let doc = Json::parse(text).map_err(|e| malformed(e.to_string()))?;
    let version = doc
        .get("schema_version")
        .and_then(Json::as_u64)
        .ok_or_else(|| malformed("missing schema_version".into()))?;
    if version != SCHEMA_VERSION {
        return Err(malformed(format!("unsupported schema_version {version}")));
    }
    let path_str = doc
        .get("path")
        .and_then(Json::as_str)
        .ok_or_else(|| malformed("missing path".into()))?;
    let path = ProjectPath::normalize(path_str).map_err(|e| malformed(format!("bad path: {e}")))?;
    let body = doc
        .get("text")
        .and_then(Json::as_str)
        .ok_or_else(|| malformed("missing text".into()))?;
    let recorded_hash = doc
        .get("text_sha256")
        .and_then(Json::as_str)
        .and_then(parse_hex)
        .ok_or_else(|| malformed("missing text_sha256".into()))?;
    let text_sha256 = sha256(body.as_bytes());
    if text_sha256 != recorded_hash {
        return Err(malformed(
            "text_sha256 does not match text (truncated or corrupted)".into(),
        ));
    }
    let base_sha256 = match doc.get("base_sha256") {
        None | Some(Json::Null) => None,
        Some(v) => Some(
            v.as_str()
                .and_then(parse_hex)
                .ok_or_else(|| malformed("bad base_sha256".into()))?,
        ),
    };
    let millis = doc
        .get("saved_at_unix_ms")
        .and_then(Json::as_u64)
        .ok_or_else(|| malformed("missing saved_at_unix_ms".into()))?;
    Ok(RecoveryEntry {
        path,
        text: body.to_string(),
        text_sha256,
        base_sha256,
        saved_at: UNIX_EPOCH + Duration::from_millis(millis),
        journal_file: file.to_path_buf(),
    })
}
