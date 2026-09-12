# mac-project-files handoff

Agent / task / branch: mac-project-files (Claude Code subagent, parent
mac-claude-a, machine mac-m1max-a) / issue #2 dispatch "project file layer"
+ issue #18 urgent fix (no FT number, no assignment file, so no `coord.py
ack` is possible) / `agent/mac-project-files/graph`
State: ready for integration. Issue #18 fix pushed at
d92db378f25cc0b882a7a45e0a83a96f58ca1306 (branch merged origin/main df9e26e
at 210e7e1; reviewed 38155b1 — no overlap).
Owned paths: `crates/project-files/**`, `coordination/mac-project-files.md`,
`coordination/agents/mac-project-files.json`
Main integrated through: df9e26e (merged); reviewed
38155b125775d84438b5694c9bfb9fb71a2c0556 (coordination + protocol typed
rules; nothing under crates/project-files).

## Issue #18 — exact API at d92db37 (`crates/project-files/src/save.rs`)

```rust
pub struct ProjectRoot;                      // open dir handle, O_DIRECTORY|O_NOFOLLOW
impl ProjectRoot {
    pub fn open(path: &Path) -> Result<ProjectRoot, SaveError>;
    pub fn path(&self) -> &Path;
    pub fn read(&self, path: &ProjectPath, limit: u64) -> Result<Option<RootedRead>, SaveError>;
    pub fn read_text(&self, path: &ProjectPath, limit: u64) -> Result<Option<(String, RootedRead)>, SaveError>;
    pub fn lock(&self) -> Result<ProjectLock<'_>, SaveError>;                 // flock(LOCK_EX|LOCK_NB) on .flashtex/project.lock
    pub fn save(&self, path: &ProjectPath, bytes: &[u8], expected: Expected, force: bool) -> Result<SaveReceipt, SaveError>;
    pub fn remove(&self, path: &ProjectPath) -> Result<bool, SaveError>;
}
pub struct ProjectLock<'a>;                  // unlock on drop
impl ProjectLock<'_> {
    pub fn root(&self) -> &ProjectRoot;
    pub fn save(&self, path: &ProjectPath, bytes: &[u8], expected: Expected, force: bool) -> Result<SaveReceipt, SaveError>;
    pub fn remove(&self, path: &ProjectPath) -> Result<bool, SaveError>;
}
pub fn save_atomic(root: &Path, path: &ProjectPath, text: &str, expected: Expected, force: bool) -> Result<SaveReceipt, SaveError>;
pub fn save_atomic_bytes(root: &Path, path: &ProjectPath, bytes: &[u8], expected: Expected, force: bool) -> Result<SaveReceipt, SaveError>;
pub enum Expected { NewFile, Hash(Digest), Any }
pub struct SaveReceipt { path: ProjectPath, bytes: u64, sha256: Digest, mtime: SystemTime, identity: FileIdentity }
pub struct RootedRead  { path, bytes: Vec<u8>, sha256: Digest, mtime, identity: FileIdentity, mode: u32 }
pub struct FileIdentity { dev: u64, ino: u64 }
pub enum SaveError { Conflict(Box<SaveConflict>), Refused(Refused), Io(io::Error), DirectorySync(io::Error) }
pub enum Refused { SymlinkComponent{component: String}, NotADirectory{component}, NotARegularFile{component},
                   EscapesRoot{component}, TooLarge{limit: u64, size: u64}, LockUnavailable{lock_path: PathBuf}, Unsupported }
pub struct SaveConflict { path, kind: SaveConflictKind, ours: Option<Digest>, theirs: Option<Digest>, mtime: Option<SystemTime>, size: Option<u64> }
pub enum SaveConflictKind { ModifiedExternally, DeletedExternally, AlreadyExists, ModifiedDuringSave }
pub const LOCK_FILE: &str = ".flashtex/project.lock";
pub const DEFAULT_READ_LIMIT: u64 = 64 * 1024 * 1024;
// recovery.rs
pub struct RecoveryJournal<'r>;  RecoveryJournal::new(&ProjectRoot);
    record(&self, &ProjectLock, &ProjectPath, text: &str, base: Option<Digest>) -> Result<RecoveryEntry, RecoveryError>
    list() / load(&ProjectPath) / check(&RecoveryEntry) -> RestoreCheck
    restore_to_disk(&self, &ProjectLock, &RecoveryEntry, force) -> Result<Option<SaveReceipt>, RecoveryError>
    discard(&self, &ProjectLock, &ProjectPath) -> Result<bool, RecoveryError>
```

Mechanism: `src/sys.rs` binds `openat/renameat/unlinkat/mkdirat/flock`
directly to the C library std already links (no external crate; std has no
openat). Root opened `O_DIRECTORY|O_NOFOLLOW`; each component walked with
`openat(O_DIRECTORY|O_NOFOLLOW)` and its `..` compared by dev/ino to the
parent handle; file opened `O_NOFOLLOW`. Save: observe → compare → temp
`O_CREAT|O_EXCL|O_NOFOLLOW` + fsync + fchmod → re-observe (symlink → Refused;
change → Conflict{ModifiedDuringSave}) → renameat → dir fsync (hard error
`DirectorySync`) → reopen, verify dev/ino == temp and hash == written.
Contract (README "Concurrency and durability contract"): in-contract writers
serialize on the flock; out-of-contract writers are detected (pre-rename
re-verify + post-rename identity/hash), not prevented; residual window
between re-verify and renameat documented; no power-loss test claimed.
Targets: macOS, Linux x86_64/aarch64; others `Refused::Unsupported`.

Ready behavior: everything in the previous handoff (graph, scanner,
SHA-256, revisions, snapshot/diff, recovery) plus the rooted API above.
Incomplete behavior: no JSON Lines/FFI adapter yet (README proposal step 2,
pending parent agreement); no `apps/mac` changes (parent-owned).
Interface changes and required consumer actions: `RecoveryJournal::new` now
takes `&ProjectRoot` and its write methods take `&ProjectLock`;
`SaveError` gained `Refused`/`DirectorySync`; `SaveReceipt` gained
`identity`. No consumer exists on main yet; root's preview-controller export
should use `ProjectRoot::open` + `lock` + `save`.
Validation (this Mac, rustc 1.99.0-nightly): `cargo test` 47 pass (16 unit,
10 graph, 4 recovery, 8 rooted, 6 save/watch, 3 sha256); tests: exact #18
repro (symlinked parent, Hash and forced) refused with outside file
unchanged, nested symlinked dir, symlinked file for read/save/remove,
symlinked root, not-a-directory, lock exclusive across handles + released on
drop, durability bytes/hash/dev/ino/mtime + bounded read, expected-hash
conflicts, injected races (dir fsync failure → DirectorySync; out-of-contract
write between check and rename → Conflict, temp cleaned; target swapped for
symlink → Refused, victim untouched; NewFile vs concurrently created →
Conflict), 400-iteration thread race file↔symlink (outcomes
ok/refused/conflict = [0,267,133], outside file never touched).
`cargo clippy --all-targets -- -D warnings`, `cargo fmt --check`,
`RUSTDOCFLAGS=-D warnings cargo doc` clean. Linux flag values are from
headers, not executed here.
Needs from others: Commander review/close of #18 against d92db37; parent
decision on process-vs-FFI integration.
Next action: JSON adapter + binary on parent agreement.
Peer revisions reviewed and adaptations:
- origin/main df9e26e merged (210e7e1): no conflicts; new crates
  (preview-controller, document-style, font-engine, math-layout,
  paragraph-layout, bibliography, vector-graphics) do not touch owned paths.
  preview-controller README states it does not write exported `.tex` files;
  this crate is the intended primitive (issue #18).
- origin/main 38155b1: coordination/staffing + protocol typed-rules
  negotiation; no adaptation.
- Earlier: compiler `path_is_safe`, mac-shell `DocumentFiles.swift`, issue
  #8, project-index/edit-ledger/document-runtime — unchanged from previous
  handoff.
Resource: allocation claude-mac20x-project-files (parent's Max 20x quota);
per-call usage not exposed; no purchases.
Updated: 2026-09-12T06:58Z
