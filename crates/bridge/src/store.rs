//! One-writer, atomic capture journal. Receipts are emitted only after fsync.
use crate::{identifier, BridgeError, CaptureRecord, Result};
use fs2::FileExt;
use std::{
    fs::{self, File, OpenOptions},
    io::{Read, Write},
    path::{Path, PathBuf},
};

pub struct Store {
    root: PathBuf,
    _lock: File,
}
impl Store {
    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        let root = path.as_ref().to_owned();
        #[cfg(unix)]
        {
            use std::os::unix::fs::DirBuilderExt;
            fs::DirBuilder::new()
                .recursive(true)
                .mode(0o700)
                .create(&root)?;
        }
        #[cfg(not(unix))]
        fs::create_dir_all(&root)?;
        let lock = OpenOptions::new()
            .create(true)
            .truncate(false)
            .read(true)
            .write(true)
            .open(root.join(".bridge.lock"))?;
        lock.try_lock_exclusive().map_err(|_| {
            BridgeError::new(
                "store_in_use",
                "Another bridge process owns this capture journal",
            )
        })?;
        Ok(Self { root, _lock: lock })
    }
    pub fn get(&self, id: &str) -> Result<Option<CaptureRecord>> {
        identifier(id)?;
        let path = self.root.join(format!("{id}.json"));
        let file = match File::open(path) {
            Ok(file) => file,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
            Err(e) => return Err(e.into()),
        };
        let mut bytes = Vec::new();
        file.take(16 * 1024 * 1024 + 1).read_to_end(&mut bytes)?;
        if bytes.len() > 16 * 1024 * 1024 {
            return Err(BridgeError::new(
                "invalid_journal",
                "Capture journal record exceeds its size limit",
            ));
        }
        let record: CaptureRecord = serde_json::from_slice(&bytes)?;
        if record.schema_version != 1
            || record.capture.capture_id != id
            || record.request_sha256 != crate::digest(&serde_json::to_vec(&record.capture)?)
        {
            return Err(BridgeError::new(
                "invalid_journal",
                "Capture journal identity or integrity check failed",
            ));
        }
        Ok(Some(record))
    }
    pub fn require(&self, id: &str) -> Result<CaptureRecord> {
        self.get(id)?.ok_or_else(|| {
            BridgeError::new("capture_missing", "Capture has not been durably received")
        })
    }
    pub fn save(&mut self, record: &CaptureRecord) -> Result<()> {
        identifier(&record.capture.capture_id)?;
        let mut temporary = tempfile::NamedTempFile::new_in(&self.root)?;
        serde_json::to_writer(&mut temporary, record)?;
        temporary.write_all(b"\n")?;
        temporary.as_file().sync_all()?;
        temporary
            .persist(
                self.root
                    .join(format!("{}.json", record.capture.capture_id)),
            )
            .map_err(|e| BridgeError::from(e.error))?;
        File::open(&self.root)?.sync_all()?;
        Ok(())
    }
}
