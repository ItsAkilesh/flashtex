# daniel-bundle handoff

Agent / task / branch: daniel-bundle (FlashTeX agent) / FT-043 "project export
bundle" / `agent/daniel-bundle/project-bundle`
State: ready for integration, revision 2 (bounded import preview + no-clobber
apply on top of `flashtex-project-files`' rooted reader; zero consumers yet).
Owned paths: `crates/project-bundle/**`, `coordination/daniel-bundle.md`
Main integrated through: `e5901797e8a7ebdd8d714ecdee6793e1097515a9` (the
`origin/main` tip at the rev 2 fetch+merge; `origin/main` has since advanced
with unrelated work from other agents — not reviewed further this
revision). No other crate touched; no workspace root `Cargo.toml` created
(each crate here builds standalone, matching existing siblings such as
`crates/project-files`).

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

Public API (`src/lib.rs` re-exports):
- `ProjectRoot::new(path) -> Result<ProjectRoot, BundleError>` — canonicalizes
  `path`; fails unless it exists and is a directory.
- `ProjectRoot::resolve(relative: &str) -> Result<PathBuf, BundleError>` /
  `ProjectRoot::read_rooted(relative: &str) -> Result<Vec<u8>, BundleError>` —
  the single chokepoint every read goes through.
- `validate_relative_path(path: &str) -> Result<(), BundleError>` — the pure,
  I/O-free syntactic half of that check, exposed standalone.
- `BundleEntry::new(path: impl Into<String>)` — the entire input contract:
  one caller-declared path relative to the root.
- `build_bundle(root: &ProjectRoot, entries: &[BundleEntry]) -> Result<Bundle, BundleError>`
  — resolves, reads and hashes exactly the given entries.
- `Bundle { files: Vec<BundleFile> }` with `BundleFile { path, sha256: [u8;
  32], size: u64, contents: Vec<u8> }`, plus `Bundle::manifest_bytes()`,
  `Bundle::manifest_sha256()`, `Bundle::manifest_hex()`.
- `BundleError`: `InvalidRoot`, `EmptyPath`, `AbsolutePath`, `PathTraversal`,
  `MalformedPath`, `DuplicatePath`, `SymlinkEscapesRoot`, `NotFound`,
  `NotAFile`, `Io` — each carries the offending caller-declared path (or
  message), `Debug + Clone + PartialEq + Eq + Display + std::error::Error`.

### Rooting (security core of this lane)

`ProjectRoot::resolve` runs, in order:
1. `validate_relative_path` — pure, no filesystem access: rejects an empty
   path, a leading `/` (`AbsolutePath`), any `..` component
   (`PathTraversal`), and any other malformed shape (`.` component, empty
   component from `//` or a trailing `/`, embedded NUL) as `MalformedPath`.
   This fires identically whether or not a matching file exists outside the
   root — traversal and absolute paths are rejected on syntax alone.
2. Join onto the canonical root, then `fs::canonicalize` the joined path
   (which follows symlinks) and require the result to still `starts_with`
   the canonical root — otherwise `SymlinkEscapesRoot`.
3. Require the resolved item to be a regular file — otherwise `NotAFile`.
   The crate never reads a directory's contents at any point.

Proven in `tests/rooted.rs`: `dot_dot_traversal_is_rejected` (`..` to a file
that genuinely exists outside root) and
`dot_dot_traversal_is_rejected_even_when_target_does_not_exist` (syntactic,
not existence-dependent); `absolute_path_is_rejected` (absolute path to a
file that *is* inside the root, still rejected); `symlink_escaping_root_is_rejected`
(real `symlink` via `std::os::unix::fs::symlink` to an outside temp dir) vs.
`symlink_staying_inside_root_is_allowed` (control case).

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

Rev 2: 42 integration tests (`tests/rooted.rs` 10, `tests/no_discovery.rs` 2,
`tests/determinism.rs` 4, `tests/malformed_and_unicode.rs` 10,
`tests/preview.rs` 3, `tests/apply.rs` 8, `tests/bounds.rs` 5) + 1 doctest =
43 total, 0 unit tests inside `src/` (all behavior is exercised at the
public API). Malformed-input coverage unchanged in kind, updated in
expected outcome per the normalization change above (empty path, NUL byte,
`//`, trailing `/`, `.` component and internal `..` all now accepted after
normalization; bare `..` and an escaping `..`/absolute path/symlink are
still rejected). Unicode coverage unchanged: non-ASCII filenames (`café.tex`,
`日本語のファイル.tex`, an emoji filename) round-tripping content and
participating correctly in byte-order sorting, plus a non-ASCII directory
component. New: preview classification (new/unchanged/conflict) and
hash-based (not size/mtime-based) differentiation, preview leaving the
target byte-for-byte and file-for-file untouched, apply's no-clobber rules
(undecided conflict, explicit skip, explicit overwrite, default-write new,
never-rewrite unchanged) and both races (target modified / created between
preview and apply) refused rather than clobbered, and bundle entry-count /
total-byte / per-file-size limits at and over the boundary.

## Validation

rustc/cargo 1.98.1, this machine:
- `cargo build --manifest-path crates/project-bundle/Cargo.toml`: clean.
- `cargo test --manifest-path crates/project-bundle/Cargo.toml`: 43 passed
  (42 integration + 1 doctest), 0 failed.
- `cargo clippy --manifest-path crates/project-bundle/Cargo.toml --all-targets -- -D warnings`:
  clean, 0 warnings.

Exact tested commit SHA (the commit whose `crates/project-bundle` tree the
above three commands were run against): `ddd445d421dae99f80c9bed37e253b3c6c69c82e`.
(Rev 1's tested SHA for reference: `5a37954d2ed8f5e73379e7d31381bcfefcd8c6c2`.)

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
- `apply_import` is not atomic across a whole batch: if one file's write
  fails partway through a multi-file call (e.g. a race on the third of five
  files), the writes already made for earlier files in that call stand —
  each individual write is itself safe (never a silent overwrite), but the
  batch as a whole is not rolled back. Not required by the rev 2 objective
  as stated; flagged as a reasonable follow-up if all-or-nothing import
  semantics are wanted later.
- `preview_import`/`apply_import` take a whole `Bundle` already built (and
  therefore already bounded by `BundleLimits`); they do not independently
  re-check bounds, since nothing they do can grow the file set beyond what
  `build_bundle_with_limits` already admitted.

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

Resource: allocation `daniel-claude20x-shared`; no purchases.

Updated: 2026-09-12T09:31:51Z
