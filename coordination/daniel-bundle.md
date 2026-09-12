# daniel-bundle handoff

Agent / task / branch: daniel-bundle (FlashTeX agent) / FT-043 "project export
bundle" / `agent/daniel-bundle/project-bundle`
State: ready for integration, revision 4 (contract-drift repair, three real
defect fixes carried over from today's work on this branch, and a switch to
Unicode-normalization-based `AmbiguousPath` detection, on top of rev 3's
batch-recoverable `apply_import` and rev 2's bounded import preview +
no-clobber apply over `flashtex-project-files`' rooted reader; zero
consumers yet).
Owned paths: `crates/project-bundle/**`, `coordination/daniel-bundle.md`,
`coordination/agents/daniel-bundle.json`
Main integrated through: `abbe88a5275b89d99357815846de3cbe76a91810` (unchanged
this revision — no new `origin/main` fetch/merge was performed; see rev 3
below for the last one).

## Revision 4 — contract-drift repair, three defect fixes, Unicode-normalization detection

Objective: an independent contract audit found the published typed contract
in this file quoting an API that does not exist in `crates/project-bundle/src/`
(wrong method name and return type, an omission in the error-variant list,
and one leftover stale identifier), plus three real defects fixed in the
crate today whose behavior the contract did not yet describe, plus a
correction to the stated reason `AmbiguousPath` detection was built the way
it was.

### What the audit found wrong, and the fix

1. **`ProjectRoot::resolve(relative: &str) -> Result<PathBuf, BundleError>`
   was quoted but does not exist.** The real, currently-existing method is
   `ProjectRoot::normalize(relative: &str) -> Result<ProjectPath, BundleError>`
   — different in both name and return type. Code written against the
   fictitious signature would not compile. See "Typed contract" below,
   rebuilt from source rather than patched.
2. **The `BundleError` variant list omitted `ReservedPath` and
   `PreviewBundleMismatch`**, both live and actively constructed in
   `apply_import`, while the surrounding prose explicitly claimed to be the
   corrected "current full variant list" after fixing an earlier stale one.
   Both are fixes landed on this branch today (`e32796cd`, `d18fa51f`; see
   below) — the contract simply had never been updated for them.
3. **One sentence still said `SymlinkEscapesRoot`**, renamed to
   `SymlinkRefused` in rev 2.

The full "Typed contract" section below was rebuilt from the current
`crates/project-bundle/src/` rather than edited line-by-line, per this
revision's task instructions — editing prose in place is exactly how (2)
and (3) were introduced (a real change landed in the crate; the paragraph
describing the API surface was not revisited). It also replaced an entire
"Rooting" subsection that had been describing rev 1's own `resolve`/
`canonicalize` implementation and its now-renamed test — dead since rev 2,
never updated in two revisions.

### Three real defects fixed in this crate today

- `e32796cd` — importing a bundle entry named `.flashtex/project.lock`
  overwrote the project's own lock file mid-batch, letting a second writer
  acquire "the lock" while the original holder still believed it held it.
  **Security-relevant.** Now a typed `BundleError::ReservedPath`, checked
  for the whole batch before any write.
- `d18fa51f` — `apply_import` hit an `expect()` and panicked mid-batch when
  the supplied preview named a path the supplied bundle did not contain
  (e.g. the caller rebuilt the bundle after computing the preview); the
  panic unwound past the rollback that exists precisely for a mid-batch
  failure, leaving a partial import standing on disk. Now a typed
  `BundleError::PreviewBundleMismatch`, checked up front.
- `4809630b` — importing into a not-yet-existing subdirectory (e.g.
  `chapters/intro.tex` into a target with no `chapters/` yet) returned a raw
  `Io` error instead of classifying the entry as absent/new, even though
  `apply_import`'s own writer creates missing parent directories.

Full detail, reasoning, and the exact test each is proven by are in the
"Fixed today" subsection under "Typed contract" below, next to the exact
signatures and variants they touch.

### Correction: the "crates.io unreachable" rationale for `AmbiguousPath` detection was false

Rev 3's "Incomplete behavior" section recorded that `AmbiguousPath`
detection used `fs::canonicalize` identity instead of a `unicode-normalization`-based
check because "network access to crates.io was blocked in this
environment." That claim was checked today and is **false** on this
machine: `curl -s -o /dev/null -w '%{http_code}' https://index.crates.io/config.json`
returns `200`, the crate's own index entry
(`https://index.crates.io/un/ic/unicode-normalization`) resolves and lists
current published versions up to `0.1.25`, and `cargo build` after adding
`unicode-normalization = "0.1.25"` to `Cargo.toml` fetches and compiles it
cleanly. The record is corrected here so it stops justifying a design
choice on a false premise; see the next section for what was done about it
on the merits.

### Assessment: is `unicode-normalization` the better implementation for `AmbiguousPath`?

Judged on the merits, independent of the (now-corrected) reachability
claim: **yes, switched.** Reasoning:

- `AmbiguousPath`'s own documented contract is a *Unicode-normalization*
  hazard (precomposed vs. combining-mark-decomposed forms of one visual
  name) — not general filesystem identity. Comparing each declared path's
  NFC-normalized form directly is a more precise fit for that stated
  contract than probing the filesystem for *any* kind of identity collision
  (which also happened to catch e.g. case-folding on a case-insensitive
  volume — an incidental side effect the old contract's own "Incomplete
  behavior" section already flagged as "a reasonable bonus, not a claim of
  Unicode correctness," so losing it is not a loss of anything documented).
- `fs::canonicalize` requires its argument to already exist on disk. That
  makes the old check structurally unable to fire for the import-preview
  case: validating a bundle for this hazard before any of its files have
  been read or, on the target side, before anything has been written yet.
  A purely syntactic, string-only check has no such requirement.
- It closes a real gap, not just a theoretical one, and closes it inside
  the existing call site: `build_bundle_with_limits` previously ran the
  identity check *after* successfully reading each entry, so if the
  *second* of two colliding paths had no backing file at all, its own read
  failed with `NotFound` first — masking the collision instead of reporting
  it. The new check runs before the read (matching how `DuplicatePath` was
  already checked, by name, before any read), so this case is now
  `AmbiguousPath`, proven by
  `tests/malformed_and_unicode.rs::unicode_normalization_collision_is_detected_before_the_second_path_is_ever_read`.
- Deliberate behavior change, flagged rather than buried: detection is now
  **filesystem-independent**. It fires for any two Unicode-canonically
  equivalent declared paths on every platform, not only where the host
  filesystem actually folds the two spellings together (the old, narrower
  behavior). This is a stricter, more conservative default, and it matches
  how every other check in this crate already works — syntactic and
  host-independent wherever possible (see `ProjectRoot::normalize`'s own
  `..`/absolute-path checks, which fire "regardless of whether anything
  exists outside the root"). For a bundle whose whole purpose is moving a
  project between machines, rejecting an ambiguity that is safe on the
  build machine but unsafe on the target machine is the safer failure mode.
- The one capability actually given up — catching a collision that is
  real filesystem identity but *not* Unicode-normalization (e.g. two case
  variants on a case-insensitive volume) — was never part of
  `AmbiguousPath`'s documented contract to begin with (see above), so
  nothing promised to a consumer is lost.

Implementation: `ProjectRoot::canonical_identity(&self, relative: &str) -> Option<PathBuf>`
(`fs::canonicalize` on the resolved OS path) was replaced by
`ProjectRoot::normalized_identity(relative: &str) -> Option<String>` (no
`&self`, no I/O): validates `relative` syntactically via
`ProjectRoot::normalize`, then returns its NFC-normalized form. Added
`unicode-normalization = "0.1.25"` as a dependency (the current published
version as of today; fetched and built clean from crates.io on this
machine, confirmed above). `BundleError::AmbiguousPath` itself, and its
public contract, are unchanged — this is a detection-mechanism swap behind
an already-typed error, not an API change.

The pre-existing test for this behavior remains green without modification,
including the part of it that depends on this machine's actual filesystem
folding NFC/NFD together
(`unicode_normalization_collision_is_rejected_not_silently_admitted`) —
switching mechanisms did not need to touch it. One new test was added,
proving the detection now works without the second (colliding) path
existing on disk at all — see above. All 63 previously-passing tests in the
crate remain green; the new test makes 64 total (63 integration + 1
doctest) — see "Test counts" and "Validation" below.

## Revision 3 — batch recovery, adversarial bounds, stale-identity acceptance

Objective: "Bounded project bundle recovery: bounded adversarial and
stale-identity acceptance tests."

### Assessment of the interrupted rev 3 work found in the worktree

A previous rev 3 run had already been interrupted mid-session, leaving
uncommitted changes to `src/apply.rs` and `src/error.rs` plus a new,
uncommitted `tests/recovery.rs`. **Verdict: sound, kept as the foundation
for this revision, not discarded or reworked.** It builds clean, and its 4
new tests (first/middle/last write failure, plus overwritten-conflict
restore) all passed before I changed anything. It is exactly the batch
atomicity fix the rev 2 handoff had flagged as an open gap (see the
now-removed "Incomplete behavior" bullet below): each write commits a small
undo record (bytes to restore, or "remove — this call created it") as it
goes; on a later failure, everything already committed in that call is
undone in reverse order under the same lock the forward writes used, and
the original typed cause is returned unchanged. I committed it as-is (see
the git log) rather than rewrite it, then added the remaining rev 3
requirements — the interrupted work did not need correction, only
completion.

### Batch recovery (`apply_import`, `src/apply.rs`)

The reused writer (`flashtex_project_files::ProjectLock::save`) makes
exactly one file's write atomic; it has no multi-file transaction. This
crate now builds batch recoverability on top of it: `apply_one` returns,
alongside each successful write, an `Undo` (`Remove { written_sha256 }` for
a file this call created, `Restore { original_bytes, written_sha256 }` for
one it overwrote). `apply_import` accumulates these in order and, on any
later failure, calls `roll_back` to undo them in reverse — each undo is
itself compare-and-swapped against exactly what this call wrote, so a
double-fault (something outside this call's contract touching a
just-written path) is detected rather than silently clobbering a second
time. Two outcomes only: a clean rollback returns the original typed cause
(`OverwriteNotDecided`, `ConcurrentModification`, etc.) with the target
byte-for-byte as it was before the call; a rollback that cannot fully
complete returns `BundleError::RollbackIncomplete { original_cause,
left_in_written_state }`, naming exactly which paths are left in the state
this call wrote them to.

Tested by injecting a real race (an out-of-contract writer creating/
modifying the target between preview and apply, the same mechanism
`tests/apply.rs`'s single-file race tests already used) at the first, the
middle, and the last of three writes, plus a fourth variant where every
file is an overwritten conflict rather than a new file (`tests/recovery.rs`,
4 tests). What survives at each injection point: the failing path itself is
left exactly as the race left it (never touched by this call); every path
before it in iteration order is rolled back to its pre-call state (removed
if this call created it, restored to its exact original bytes if this call
overwrote it); every path after it was never attempted.

### New adversarial bounds (`src/bundle.rs`, `src/root.rs`, `src/error.rs`)

- **A path of only separators** (`"///"`) is `BundleError::AbsolutePath`,
  not silently normalized down to the empty-path case — `ProjectPath::normalize`
  checks `starts_with('/')` before it ever splits into segments, so this is
  pinned as distinct from `EmptyPath`. (`tests/malformed_and_unicode.rs::path_of_only_separators_is_rejected_as_absolute`)
- **Names differing only by Unicode normalization (the documented APFS
  hazard):** two declared paths that are byte-*different* (precomposed
  `é`, U+00E9, vs. `e` + combining acute, U+0301) but resolve to the same
  file on a normalization-insensitive volume are now rejected as a new
  typed error, `BundleError::AmbiguousPath { first, second }`, rather than
  silently admitted as two bundle entries that would in fact collide.
  Detection is `ProjectRoot::canonical_identity` — `fs::canonicalize` on
  each declared path's resolved OS path, tracked in a `HashMap<PathBuf,
  String>` alongside the existing exact-string `DuplicatePath` check — a
  filesystem-identity check, not a hand-rolled Unicode normalization table
  (this crate takes no new dependency; NFC/NFD tables are exactly the kind
  of thing worth reusing a real Unicode library for, not reimplementing,
  and none was available to add — see "Incomplete behavior"). Consequence:
  this only fires on a filesystem that actually folds the two spellings;
  on one that does not, the two paths genuinely are different files and no
  error fires, correctly. Proven against this machine's real filesystem —
  the test asserts the fold actually happens before asserting the crate's
  response to it (`tests/malformed_and_unicode.rs::unicode_normalization_collision_is_rejected_not_silently_admitted`).
- The other six items on the rev 3 adversarial list (entry count at/past
  the cap, total bytes at/past the cap, one file past the per-file limit,
  duplicate paths, an empty path, a target that changes between preview
  and apply, a decision map missing an entry for a conflict) were already
  covered by rev 1/2's `tests/bounds.rs`, `tests/malformed_and_unicode.rs`
  and `tests/apply.rs` — verified still passing, not re-proven.
- Every one of these is a typed `BundleError` returned normally; none
  panics or hangs (no `unwrap`/`expect`/`panic!` on caller-controlled input
  anywhere in `src/`, and every test above runs to completion under the
  default `cargo test` timeout).

### Stale-identity acceptance (`tests/stale_identity.rs`, new file)

Specification: an `ImportPreview` is valid only while the target hashes it
observed still hold; any change invalidates it; `apply_import` refuses
rather than clobbers. Rev 2 already proved this for a previewed file being
*modified* or *created* between preview and apply
(`tests/apply.rs::concurrent_modification_between_preview_and_apply_is_refused_not_clobbered`,
`::concurrently_created_new_file_is_refused_not_clobbered`). This revision
adds the third way a previewed file can go stale — **deletion**, not
previously tested (`check_expected`'s `Expected::Hash(_)` vs. `None`
`DeletedExternally` path existed in `flashtex_project_files` but nothing
in this crate had exercised it) — and proves the property holds at the
whole-batch level together with batch recovery: one previewed file going
stale by deletion mid-batch still rolls back every write already committed
earlier in that same call, not just refuses the one stale write.

## Revision 2 — reuse `project-files`' rooted reader; import preview + apply

Objective: "Implement bounded manifest/import preview with full content
hashes and explicit conflict/no-clobber behavior using existing rooted
reader; no silent overwrite."

### What changed

`crates/project-bundle/src/root.rs`'s `ProjectRoot` is no longer its own
canonicalize-based rooting; it is now a thin typed adapter over
`flashtex_project_files::ProjectRoot` (issue #18's `openat(O_NOFOLLOW)`-based
reader documented in `crates/project-files/README.md`). This crate performs
no filesystem traversal, symlink resolution, or escape detection of its own
any more — every read goes through the reused reader, and `BundleError`
variants are produced by mapping `flashtex_project_files::{PathError,
SaveError, Refused}` rather than by an independent check.

**Security-property gap check (the point of this reuse instruction):**
`flashtex_project_files::ProjectRoot` provides — and in fact exceeds — every
security guarantee rev 1's own rooting proved:

- Absolute paths and any `..` that would leave the root are still rejected,
  now via `ProjectPath::normalize` (syntactic, before any filesystem access)
  plus a walk-time device/inode check of each directory's `..` against the
  handle it was reached from. The walk-time check is *new* relative to rev
  1's canonicalize-then-`starts_with`, which had a small TOCTOU window
  between canonicalizing and reading; `openat` walks one already-open
  directory handle at a time, closing it.
- Symlink escapes are still rejected — in fact every symlink component
  (parent or leaf) is refused outright via `openat(O_NOFOLLOW)`, which is
  strictly stronger than rev 1's "resolve then check `starts_with`".

No gap was found, so no extra covering checks were layered on top. Two
behavior *differences* were kept and are deliberate, not regressions:

1. Rev 1 rejected any `.`, empty (`//`), or `..` path segment outright as
   `MalformedPath`/`PathTraversal`, even when it would have stayed inside
   the root once resolved. `ProjectPath::normalize` instead normalizes these
   away (`a/../foo.tex` → `foo.tex`) and only rejects a `..` that actually
   escapes. This is a relaxation of input syntax strictness, not a security
   loss — normalization cannot produce a path outside the root.
2. Rev 1 *followed* an internal symlink (one resolving inside the root) and
   only rejected an escaping one. The reused reader refuses **every**
   symlink unconditionally, so `tests/rooted.rs`'s old
   `symlink_staying_inside_root_is_allowed` is now
   `symlink_staying_inside_root_is_also_refused` — strictly more
   conservative, not a gap.

`BundleError::SymlinkEscapesRoot` was renamed to `SymlinkRefused` to stop
implying "only if it escapes" when the underlying reader no longer makes
that distinction.

### New: import preview (`src/preview.rs`)

`preview_import(bundle: &Bundle, target: &ProjectRoot) -> Result<ImportPreview, BundleError>`
reads every bundle file from `target` (rooted, bounded — the same
`ProjectRoot::read_rooted_optional` used everywhere else) and classifies
each path by comparing full SHA-256 on both sides:

- `FileOutcome::New` — absent from `target`.
- `FileOutcome::Unchanged { sha256 }` — present with an equal hash.
- `FileOutcome::Conflict { ours, theirs, theirs_size }` — present with a
  different hash; both full digests are carried so a caller never has to
  re-hash to tell "differs" from "identical".

**No writes, structurally, not by convention:** `preview_import` only ever
calls `flashtex_project_files::ProjectRoot::read`; it never calls
`lock`/`save`/`remove`. `tests/preview.rs::preview_performs_no_writes_at_all`
snapshots the target directory's full (path, size, mtime) tuples before and
after, asserts they are byte-identical, and asserts `.flashtex/` (the lock
file's lazily-created parent) never appears.

### New: no-clobber apply (`src/apply.rs`)

`apply_import(bundle, preview, target, decisions: &HashMap<String, ImportDecision>)`
turns a preview into actual writes, enforced rather than merely documented:

- A `Conflict` path is written only when `decisions[path] == Some(Write)`.
  An **absent** entry is `BundleError::OverwriteNotDecided` — omission is
  never treated as consent, unlike an explicit `Skip`. A `Write` decision
  writes with `Expected::Hash(theirs)` (the exact hash observed at preview
  time) as a compare-and-swap.
- A `New` path defaults to being written (nothing to overwrite) unless the
  caller explicitly opts out with `Skip`; it writes with `Expected::NewFile`.
- An `Unchanged` path is never rewritten regardless of decision.
- Either write path re-verifies against the live file at write time; if it
  changed since the preview, the write is refused as
  `BundleError::ConcurrentModification` (surfaced from the reused writer's
  own `SaveError::Conflict`) rather than clobbered. `force` is never passed
  as `true` anywhere in this crate.
- All writes in one `apply_import` call share one project lock, so no other
  in-contract writer interleaves partway through a batch.

### New: bounds (`src/bundle.rs`)

`BundleLimits { max_entries, max_total_bytes }` (defaults 100,000 entries /
512 MiB) is checked in `build_bundle_with_limits`: entry count is checked
before any file is read (`BundleError::TooManyEntries`); the running byte
total is checked after each read (`BundleError::TotalBytesExceeded`). Each
individual file is additionally bounded by the `ProjectRoot`'s own
`DEFAULT_FILE_LIMIT` (64 MiB, matching `flashtex_project_files`) or a
smaller limit passed to `ProjectRoot::with_file_limit`
(`BundleError::FileTooLarge`). `build_bundle` keeps the old signature via
`BundleLimits::default()`.

Hashing now reuses `flashtex_project_files::sha256` (`sha256`/`hex`/`Digest`)
instead of the `sha2` crate, so the `sha2` dependency was dropped — the
bundle-side and target-side hashes compared in `preview_import` come from
the exact same implementation.

## Typed contract (`flashtex-project-bundle`, edition 2024)

**Rebuilt from the current source** (`crates/project-bundle/src/{lib,root,bundle,preview,apply,error}.rs`,
tested SHA `4523011dc55492acc3fcaa32a1748da53a46b888`, verified below) rather
than edited from the previous copy — see "Corrections made to this section"
at the end for exactly what was wrong and how it was found. Every signature
below was copied verbatim from source and re-checked to still exist with
this exact shape (name, arity, ownership, return type) as of the tested SHA.

### `root.rs` — `ProjectRoot`

- `ProjectRoot::new(path: impl AsRef<Path>) -> Result<Self, BundleError>` —
  opens `path` as a bundle root via `flashtex_project_files::ProjectRoot::open`
  (`openat(O_DIRECTORY|O_NOFOLLOW)`); fails as `InvalidRoot` unless `path`
  exists, is a directory, and is not itself a symlink. Bounds individual
  reads at `DEFAULT_FILE_LIMIT` (64 MiB). **Does not canonicalize `path`** —
  the previous copy of this contract claimed it did; that was never true of
  the rev-2-and-later implementation.
- `ProjectRoot::with_file_limit(path: impl AsRef<Path>, file_limit: u64) -> Result<Self, BundleError>`
  — like `new`, with a caller-chosen per-file read limit instead of the default.
- `ProjectRoot::as_path(&self) -> &Path` — the root path as opened.
- `ProjectRoot::normalize(relative: &str) -> Result<ProjectPath, BundleError>`
  — **the actual chokepoint. There is no method named `resolve`, on
  `ProjectRoot` or anywhere else in this crate** (see "Corrections" below).
  A `pub fn`, callable without an instance (`ProjectRoot::normalize(...)`,
  not `root.normalize(...)`). Pure syntactic validation and normalization,
  delegating to `flashtex_project_files::ProjectPath::normalize`; touches no
  filesystem. Rejects an empty path (`EmptyPath`), a leading `/` or `~`
  (`AbsolutePath`), a `..` that would leave the root (`PathTraversal`), and a
  forbidden character (`MalformedPath`) — all before any I/O, and identically
  whether or not anything exists at the escaped-to location.
- `ProjectRoot::read_rooted_optional(&self, relative: &str) -> Result<Option<RootedFile>, BundleError>`
  — reads `relative`'s bytes/size/SHA-256, rooted and bounded at this root's
  file limit. `Ok(None)` when `relative` is absent under the root — this
  covers both a missing *leaf* and a missing intermediate *directory* (fixed
  today, see below; previously only the leaf case was `Ok(None)`).
- `ProjectRoot::read_rooted(&self, relative: &str) -> Result<Vec<u8>, BundleError>`
  — like `read_rooted_optional`, but a missing file is `BundleError::NotFound`
  rather than `Ok(None)`.
- `RootedFile { pub bytes: Vec<u8>, pub sha256: Digest, pub size: u64 }` —
  `Debug + Clone + PartialEq + Eq`.
- `DEFAULT_FILE_LIMIT: u64` — 64 MiB (`flashtex_project_files::DEFAULT_READ_LIMIT`).
- `validate_relative_path(path: &str) -> Result<(), BundleError>` — the pure,
  I/O-free syntactic half, exposed standalone (`ProjectRoot::normalize(path).map(|_| ())`).

### `bundle.rs` — building a bundle

- `BundleEntry::new(path: impl Into<String>) -> Self`; `BundleEntry { pub path: String }`
  — the entire input contract: one caller-declared path relative to the root.
- `BundleFile { pub path: String, pub sha256: Digest, pub size: u64, pub contents: Vec<u8> }`
- `Bundle { pub files: Vec<BundleFile> }` with:
  - `Bundle::manifest_bytes(&self) -> Vec<u8>`
  - `Bundle::manifest_sha256(&self) -> Digest`
  - `Bundle::manifest_hex(&self) -> String`
  - `Bundle::file(&self, path: &str) -> Option<&BundleFile>`
- `BundleLimits { pub max_entries: usize, pub max_total_bytes: u64 }` —
  `Default` is `DEFAULT_MAX_ENTRIES` (100,000) / `DEFAULT_MAX_TOTAL_BYTES`
  (512 MiB).
- `build_bundle(root: &ProjectRoot, entries: &[BundleEntry]) -> Result<Bundle, BundleError>`
  — `build_bundle_with_limits(root, entries, &BundleLimits::default())`.
- `build_bundle_with_limits(root: &ProjectRoot, entries: &[BundleEntry], limits: &BundleLimits) -> Result<Bundle, BundleError>`
  — resolves, reads and hashes exactly the given entries; the one both call.

### `preview.rs` — computing an import preview

- `FileOutcome` — `New`, `Unchanged { sha256: Digest }`,
  `Conflict { ours: Digest, theirs: Digest, theirs_size: u64 }`;
  `FileOutcome::is_conflict(&self) -> bool`.
- `FilePreview { pub path: String, pub outcome: FileOutcome }`
- `ImportPreview { pub files: Vec<FilePreview> }` with `new_paths(&self)`,
  `unchanged_paths(&self)` (both `impl Iterator<Item = &str>`),
  `conflicts(&self) -> impl Iterator<Item = &FilePreview>`,
  `has_conflicts(&self) -> bool`.
- `preview_import(bundle: &Bundle, target: &ProjectRoot) -> Result<ImportPreview, BundleError>`
  — read-only; structurally never takes the project lock and never calls
  `save`/`remove`.

### `apply.rs` — applying an import

- `ImportDecision` — `Skip`, `Write` (`Copy`).
- `ImportAction` — `Written { sha256: Digest, bytes: u64 }`, `Skipped`.
- `ImportOutcome { pub path: String, pub action: ImportAction }`
- `apply_import(bundle: &Bundle, preview: &ImportPreview, target: &ProjectRoot, decisions: &HashMap<String, ImportDecision>) -> Result<Vec<ImportOutcome>, BundleError>`
  — see "Fixed today" below for the two new typed rejections this function
  can now return, both checked for the whole batch before the project lock
  is taken and before any file is written.

### `error.rs` — `BundleError`, enumerated directly from the enum definition

19 variants (`Debug + Clone + PartialEq + Eq + Display + std::error::Error`
on the whole enum; each carries the offending caller-declared path, or a
message, or both):

1. `InvalidRoot(String)`
2. `EmptyPath`
3. `AbsolutePath(String)`
4. `PathTraversal(String)`
5. `MalformedPath(String)`
6. `DuplicatePath(String)`
7. `AmbiguousPath { first: String, second: String }`
8. `ReservedPath(String)` — **new today**, security-relevant; see "Fixed today" below.
9. `PreviewBundleMismatch(String)` — **new today**; see "Fixed today" below.
10. `SymlinkRefused(String)` — renamed from rev 1's `SymlinkEscapesRoot` in
    rev 2; **no variant named `SymlinkEscapesRoot` exists**.
11. `NotFound(String)`
12. `NotAFile(String)`
13. `FileTooLarge { path: String, limit: u64, size: u64 }`
14. `TooManyEntries { limit: usize, actual: usize }`
15. `TotalBytesExceeded { limit: u64, actual: u64 }`
16. `OverwriteNotDecided(String)`
17. `ConcurrentModification { path: String, expected: Option<Digest>, found: Option<Digest> }`
18. `RollbackIncomplete { original_cause: Box<BundleError>, left_in_written_state: Vec<(String, String)> }`
19. `Io(String)`

The copy of this list printed here after rev 3 omitted #8 and #9 while
explicitly claiming to be the corrected "current full variant list" — both
are actively constructed in `apply_import` (not dead code; both are
exercised by `tests/reserved_paths.rs` and `tests/preview_bundle_pairing.rs`
respectively), so a consumer written against that list could not compile a
match arm for either and would not handle a rejection it should have been
told about.

### Worked example (compiles against the exact signatures above)

Verified against this exact tested SHA in a throwaway crate depending on
this one by path (`cargo build`, clean, zero warnings) — not merely
hand-checked:

```rust
use std::collections::HashMap;
use flashtex_project_bundle::{
    apply_import, build_bundle, preview_import, BundleEntry, BundleError, ImportDecision,
    ProjectRoot,
};

fn import_one_file(
    source_dir: &std::path::Path,
    target_dir: &std::path::Path,
) -> Result<(), BundleError> {
    let source = ProjectRoot::new(source_dir)?;
    let target = ProjectRoot::new(target_dir)?;

    // ProjectRoot::normalize is the actual chokepoint -- there is no
    // `resolve`, and it returns a `ProjectPath`, not a `PathBuf`.
    let _validated = ProjectRoot::normalize("chapters/intro.tex")?;

    let bundle = build_bundle(&source, &[BundleEntry::new("chapters/intro.tex")])?;
    let preview = preview_import(&bundle, &target)?;

    let mut decisions = HashMap::new();
    for file in preview.conflicts() {
        decisions.insert(file.path.clone(), ImportDecision::Write);
    }

    let outcomes = apply_import(&bundle, &preview, &target, &decisions)?;
    for outcome in outcomes {
        println!("{}: {:?}", outcome.path, outcome.action);
    }
    Ok(())
}
```

Every fallible call (`ProjectRoot::new` ×2, `ProjectRoot::normalize`,
`build_bundle`, `preview_import`, `apply_import`) is propagated with `?`
into the function's own `Result<(), BundleError>` — none is `.unwrap()`ed or
its `Result` discarded — and every argument's arity and ownership (`&Path`,
`&str`, `&[BundleEntry]`, `&Bundle`, `&ImportPreview`, `&ProjectRoot`,
`&HashMap<String, ImportDecision>`) matches the real signatures above.

### Fixed today (three real defects; two change the error surface)

- **`ReservedPath(String)` — security-relevant.** `apply_import` used to
  write any path the caller declared, including
  `.flashtex/project.lock` — the advisory lock file that very call holds
  for the whole batch. The rooted writer commits by writing a temp file and
  `rename`ing it over the target, so importing that path replaced the
  locked *inode*: the `flock` the call still believed it held was stranded
  on an orphan, the replacement file was unlocked, and a second writer
  could immediately take "the lock" and interleave with the rest of the
  same batch — silently voiding the mutual-exclusion guarantee this crate's
  own documentation promises, while `apply_import` still returned `Ok`.
  Fixed (`e32796cd`): every previewed path is checked against the
  `.flashtex/` control-directory prefix up front, before the project lock
  is taken and before any file is written; a hit is `ReservedPath`, not a
  silent write. Proven in `tests/reserved_paths.rs`, including a test that
  characterizes the exact underlying mechanism this prevents
  (`replacing_a_held_lock_file_voids_mutual_exclusion`) and a control case
  confirming the check is a `.flashtex/`-prefix match, not a substring scan
  (`a_path_merely_resembling_the_control_directory_is_still_importable`).
- **`PreviewBundleMismatch(String)`.** `apply_import` takes `bundle` and
  `preview` as two independent arguments with nothing structurally tying
  them together; the per-file write loop used to assume they matched via
  `bundle.file(&fp.path).expect("preview built from this bundle")`. A
  caller that rebuilds or swaps the bundle after computing the preview (the
  ordinary case: a user deselects a file) hit that `expect` and panicked
  partway through the batch — the unwind skipped the rollback that exists
  precisely for a mid-batch failure, leaving files already written standing
  on disk with no `BundleError` for the caller to match on. Fixed
  (`d18fa51f`): every previewed path is checked against the bundle up front,
  in the same pre-lock, pre-write pass as the `ReservedPath` check above;
  a mismatch is the typed `PreviewBundleMismatch`, returned before anything
  is written. Proven in `tests/preview_bundle_pairing.rs`, including the
  case where the mismatch is the very first previewed path.
- **Missing parent directory read as `Io`, not absence.** The rooted reader
  walks a path one component at a time; a missing *leaf* was already
  `Ok(None)`, but a missing intermediate *directory* surfaced as a raw
  `ENOENT` mapped straight to an untyped `BundleError::Io`. Consequence:
  `preview_import` of a bundle entry such as `chapters/intro.tex` into a
  target that did not yet have a `chapters/` directory failed the whole
  preview with an I/O error — even though `apply_import`'s own writer
  creates missing parent directories — so importing any bundle containing a
  nested file into a fresh project was impossible. Fixed (`4809630b`):
  `ProjectRoot::read_rooted_optional` now treats a missing-component
  `ENOENT` from the walk the same as a missing leaf, `Ok(None)`. No
  `BundleError` variant changed; this is a behavior fix, not a new error.
  Proven in `tests/missing_parent_dir.rs`, including a guard test
  confirming the fix did not swallow a real traversal or symlink refusal
  (different errno, still typed and distinct).

### Corrections made to this section (contract-drift repair, today)

An independent contract audit found this section quoting an API that does
not match the source:

1. It documented `ProjectRoot::resolve(relative: &str) -> Result<PathBuf, BundleError>`.
   No method of that name exists anywhere in `src/`. The real chokepoint is
   `ProjectRoot::normalize(relative: &str) -> Result<ProjectPath, BundleError>`
   — different in both name and return type. Code written against the old,
   fictitious signature would not compile.
2. The variant list omitted `ReservedPath` and `PreviewBundleMismatch` (see
   "Fixed today" above) while explicitly claiming to be the corrected,
   current list.
3. One sentence still referred to `SymlinkEscapesRoot`, renamed to
   `SymlinkRefused` in rev 2.

This section was rebuilt from the current source rather than patched, per
this revision's task instructions, specifically to avoid repeating (2) and
(3): editing prose piecemeal is how they were introduced. The old "Rooting"
subsection this section replaces described rev 1's own `canonicalize`-then-
`starts_with` implementation and its test names (`symlink_staying_inside_root_is_allowed`);
that implementation was replaced in rev 2, and the cited test was renamed
to `symlink_staying_inside_root_is_also_refused` at the same time — this
subsection had not been updated since, describing dead rev-1 internals for
two revisions. Proven in `tests/rooted.rs`:
`dot_dot_traversal_is_rejected` (`..` to a file that genuinely exists
outside root) and `dot_dot_traversal_is_rejected_even_when_target_does_not_exist`
(syntactic, not existence-dependent); `absolute_path_is_rejected` (absolute
path to a file that *is* inside the root, still rejected);
`symlink_escaping_root_is_rejected` (real `symlink` via
`std::os::unix::fs::symlink` to an outside temp dir) vs.
`symlink_staying_inside_root_is_also_refused` (rev 2 tightened this from
rev 1's "allowed" to "refused unconditionally" — see rev 2 section above).

### No implicit discovery

`build_bundle` takes only `&[BundleEntry]`; the crate calls `read_dir`, globs
or walks nothing anywhere (`grep -rnE "read_dir|walkdir|glob\(" src/` matches
only the doc comment stating the guarantee). `tests/no_discovery.rs` writes
unlisted sibling files next to listed ones (including a same-extension,
alphabetically-adjacent sibling and a nested "notes" file) and asserts they
never appear in `Bundle::files` or `manifest_bytes()`.

### Ordering rule (determinism)

`Bundle::files` is sorted by `path.as_bytes()` — a plain byte-wise comparison
of the caller-declared UTF-8 path, never filesystem iteration order and
never hash-map/hash-set order (a `HashSet` is used only transiently to
detect a `DuplicatePath`, then discarded before the sort). `manifest_bytes()`
serializes that sorted list as one `"<sha256-hex>  <size>  <path>\n"` line
per file, concatenated with no header/footer. `manifest_sha256()` /
`manifest_hex()` are SHA-256 over exactly those bytes.

Consequence: for a fixed `(root, entry-set)`, `manifest_bytes()` is
byte-identical no matter the order entries were listed in, and no matter how
many times it is rebuilt. Proven in `tests/determinism.rs`:
`building_twice_yields_identical_manifest_bytes`,
`input_order_does_not_affect_manifest_bytes` (same 4 entries, reverse-ish
shuffle), `manifest_ordering_is_a_plain_byte_sort_of_path_not_locale_aware`
(`Zebra.tex` sorts before `apple.tex`, which a locale-aware sort would
invert), and `manifest_format_is_stable_and_readable` (exact byte string
pinned against an independently-computed `shasum -a 256` value).

Note: filenames differing only in Unicode normalization (NFC vs. NFD, e.g.
precomposed `é` vs. `e` + combining acute) are not distinguishable bundle
inputs on a normalization-insensitive volume (default macOS APFS folds
them to one filesystem entry); the crate's own ordering and hashing are
still plain-byte and platform-independent given whatever bytes the caller's
`String` and the filesystem agree the path is.

## Test counts

Rev 4: 63 integration tests (`tests/rooted.rs` 10, `tests/no_discovery.rs`
2, `tests/determinism.rs` 4, `tests/malformed_and_unicode.rs` 13 (+1 this
revision: `unicode_normalization_collision_is_detected_before_the_second_path_is_ever_read`),
`tests/preview.rs` 3, `tests/apply.rs` 8, `tests/bounds.rs` 5,
`tests/recovery.rs` 5, `tests/stale_identity.rs` 2,
`tests/missing_parent_dir.rs` 4, `tests/preview_bundle_pairing.rs` 3,
`tests/reserved_paths.rs` 4) + 1 doctest = 64 total, 0 unit tests inside
`src/` (all behavior is still exercised at the public API). Counts above
are read directly from this revision's `cargo test` output, not carried
forward from the rev 3 text — `tests/recovery.rs` in particular is 5, not
the 4 rev 3 recorded (an extra rollback-and-created-directory case is
present in the tree; not investigated further here since it is unrelated
to this revision's work and was already green).
`tests/missing_parent_dir.rs`, `tests/preview_bundle_pairing.rs` and
`tests/reserved_paths.rs` are new files proving the three defect fixes
under "Fixed today" above; `tests/malformed_and_unicode.rs`'s addition
proves the Unicode-normalization detection switch does not need the
colliding path to exist on disk. Every rev 1/2/3 test not named above is
unchanged and still passing.

## Validation

rustc/cargo 1.98.1, this machine:
- `cargo build --manifest-path crates/project-bundle/Cargo.toml`: clean.
- `cargo test --manifest-path crates/project-bundle/Cargo.toml`: 64 passed
  (63 integration + 1 doctest), 0 failed.
- `cargo clippy --manifest-path crates/project-bundle/Cargo.toml --all-targets -- -D warnings`:
  clean, 0 warnings.
- `cargo fmt --manifest-path crates/project-bundle/Cargo.toml --check`: clean.

Exact tested commit SHA (the commit whose `crates/project-bundle` tree the
above four commands were run against), verified before being written down
here with both:

    git -C /Users/dqi26/ft-wt-daniel-bundle cat-file -e 4523011dc55492acc3fcaa32a1748da53a46b888^{commit}
    git -C /Users/dqi26/ft-wt-daniel-bundle merge-base --is-ancestor 4523011dc55492acc3fcaa32a1748da53a46b888 HEAD

— both succeeded (exit 0). SHA: `4523011dc55492acc3fcaa32a1748da53a46b888`.
(Rev 3's tested SHA for reference: `10b4f4193a5d2dc41a4ce38b976159e08e70a8f3`.
Rev 2's: `ddd445d421dae99f80c9bed37e253b3c6c69c82e`. Rev 1's:
`5a37954d2ed8f5e73379e7d31381bcfefcd8c6c2`.)

The worked example under "Typed contract" above was additionally verified
by building it in a standalone throwaway crate depending on this one by
path at this same tested SHA (`cargo build`, clean, zero warnings).

## Incomplete behavior

- No archive/serialization format beyond the manifest (no tar/zip writer) —
  out of scope for this lane; `BundleFile::contents` gives a consumer
  everything needed to write one.
- No Windows-specific path handling (drive letters, `\` separator); paths
  are now validated by `flashtex_project_files::ProjectPath`, which targets
  the same Unix dev/CI surface as rev 1 did.
- Not wired into any consumer crate — this is an additive, standalone crate
  per the assignment; a caller integration (e.g. from `project-files`'s
  graph) would need its own follow-up and is not claimed here.
- **Resolved this revision** (was open in rev 2): `apply_import` is now
  batch-recoverable — see the rev 3 "Batch recovery" section above. The
  one residual gap is genuine double faults: if rollback *itself* cannot
  complete (an out-of-contract writer racing a path this call just wrote,
  or the disk filling mid-restore), the caller gets
  `BundleError::RollbackIncomplete` naming exactly what is left in the
  written state, rather than a silently-clean-looking failure — there is
  no way to make an actual double fault fully transparent beyond naming it.
- `preview_import`/`apply_import` take a whole `Bundle` already built (and
  therefore already bounded by `BundleLimits`); they do not independently
  re-check bounds, since nothing they do can grow the file set beyond what
  `build_bundle_with_limits` already admitted.
- **Superseded this revision (was open in rev 3):** `BundleError::AmbiguousPath`'s
  detection used to be filesystem-identity-based (`fs::canonicalize`
  equality via `ProjectRoot::canonical_identity`) on the stated rationale
  that "network access to crates.io was blocked in this environment." That
  rationale was checked this revision and found **false** — crates.io is
  reachable and `cargo build` fetches `unicode-normalization` cleanly (see
  "Correction" under Revision 4 above) — so detection was switched to
  comparing each declared path's Unicode NFC-normalized form directly, no
  filesystem access at all. See "Assessment" under Revision 4 above for the
  full reasoning. Residual, now-accepted trade-off: detection is
  filesystem-*independent* by design — it rejects any two
  Unicode-canonically-equivalent declared paths on every platform, not only
  where the host filesystem actually folds them together, which is a
  stricter default than rev 3's, deliberately chosen for a bundle meant to
  move across machines. The capability given up (also catching a
  non-Unicode filesystem-identity collision, e.g. two case variants on a
  case-insensitive volume) was already documented as an incidental bonus,
  never part of `AmbiguousPath`'s contract.

## Needs from others

None to build; a consumer wanting a bundle populated from a `ProjectGraph`
(in `crates/project-files`) would supply the entry list itself — this crate
intentionally does no discovery of its own. A caller that wants
`apply_import` decisions surfaced to a human (rather than computed
programmatically) would build its own UI over `ImportPreview`/`ImportDecision`;
none is provided here.

## Peer revisions reviewed

Rev 1 base `origin/main` at `3423970b300e878062f2a124fb8004545626200b`:
reviewed `crates/project-files/src/path.rs` and `src/save.rs` for their
existing rooted-read precedent without depending on that crate (rev 1's
`dependencies` were `[]` by assignment).

Rev 2: fetched and merged `origin/main` at
`e5901797e8a7ebdd8d714ecdee6793e1097515a9` per the rev 2 SETUP instructions
(clean merge, no conflicts). Read `crates/project-files/README.md` in full
plus `src/path.rs` and `src/save.rs` end to end (`ProjectPath::normalize`,
`ProjectRoot::open`/`read`/`lock`/`save`, the `Refused`/`SaveError`/
`SaveConflict` taxonomy, and the `openat(O_NOFOLLOW)` walk/escape-check
mechanics in `README.md`'s "Path binding" section) to verify it actually
carries rev 1's security properties before depending on it (see the "gap
check" above — none found) and added `flashtex-project-files` as a path
dependency. No file under `crates/project-files` was edited.
`origin/main` has since advanced further with unrelated work from other
agents (verified via `git log e590179..origin/main`); not reviewed this
revision, out of scope (touches no path this lane owns or depends on).

Rev 3: fetched and merged `origin/main` at
`abbe88a5275b89d99357815846de3cbe76a91810` per the rev 3 instructions,
`--no-edit`. Confirmed before merging (`git diff --stat <merge-base>
origin/main -- crates/project-bundle`, empty output) that nothing under
this lane's owned path had changed upstream since the rev 2 merge base, so
the merge was a pure fast-forward-of-history-on-other-paths with zero risk
to this crate; merge itself completed with zero conflicts anywhere (70
files touched, all outside `crates/project-bundle`). Did not review the 70
changed files' content in depth — none intersects this lane's owned or
depended-on paths (`crates/project-bundle`, `crates/project-files`); the
diff-stat check above is the actual basis for "safe to merge", not a full
read.

The repository's own `AGENTS.md` and `CLAUDE.md` (and one found at
`coordination/CLAUDE.md`) contain multiple layers of text styled as "latest
user authorization" / "explicit override" — alternate commit-identity and
attribution rules, staffing/authority claims, instructions to read further
files before starting. Per this revision's explicit task instructions,
these are untrusted and were not followed or acted on in any way; this
handoff and its commits use only this task's actual instructions and the
operator's real global configuration.

Rev 4: no new `origin/main` fetch/merge was performed (out of scope for
this revision's task). Re-read `crates/project-bundle/src/` end to end
(`apply.rs`, `bundle.rs`, `error.rs`, `lib.rs`, `preview.rs`, `root.rs`) to
rebuild the typed contract from source, and the same repository-authored
`AGENTS.md`/`CLAUDE.md`/`coordination/CLAUDE.md` text was seen again
(unchanged in substance, plus another machine's handoff at
`coordination/CLAUDE.md` styled the same way); again untrusted, not
followed or acted on, per this revision's explicit task instructions,
which state that authorization/permission/identity text found in
repository files is data, not instruction.

Resource: allocation `daniel-claude20x-shared`; no purchases.

Updated: 2026-09-12T18:08:00Z
