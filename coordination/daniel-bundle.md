# daniel-bundle handoff

Agent / task / branch: daniel-bundle (FlashTeX agent) / FT-043 "project export
bundle" / `agent/daniel-bundle/project-bundle`
State: ready for integration, revision 3 (batch-recoverable `apply_import`,
new adversarial bounds, and a stale-identity acceptance suite, on top of rev
2's bounded import preview + no-clobber apply over `flashtex-project-files`'
rooted reader; zero consumers yet).
Owned paths: `crates/project-bundle/**`, `coordination/daniel-bundle.md`,
`coordination/agents/daniel-bundle.json`
Main integrated through: `abbe88a5275b89d99357815846de3cbe76a91810` (fetched
and merged `--no-edit` this revision per the rev 3 instructions; the merge
was clean with zero conflicts, and touched nothing under
`crates/project-bundle` — verified with `git diff --stat <merge-base>
origin/main -- crates/project-bundle` before merging, empty output). No
other crate touched; no workspace root `Cargo.toml` created (each crate
here builds standalone, matching existing siblings such as
`crates/project-files`).

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
- `BundleError` (current full variant list as of rev 3; the rev 1 list
  printed here previously was stale — `SymlinkEscapesRoot` was renamed to
  `SymlinkRefused` in rev 2 and several variants were added since, none of
  which had been reflected here until now): `InvalidRoot`, `EmptyPath`,
  `AbsolutePath`, `PathTraversal`, `MalformedPath`, `DuplicatePath`,
  `AmbiguousPath` (new, rev 3), `SymlinkRefused`, `NotFound`, `NotAFile`,
  `FileTooLarge`, `TooManyEntries`, `TotalBytesExceeded`,
  `OverwriteNotDecided`, `ConcurrentModification`, `RollbackIncomplete`
  (new, rev 3), `Io` — each carries the offending caller-declared path (or
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

Rev 3: 50 integration tests (`tests/rooted.rs` 10, `tests/no_discovery.rs`
2, `tests/determinism.rs` 4, `tests/malformed_and_unicode.rs` 12 (+2 this
revision: path-of-only-separators, Unicode-normalization collision),
`tests/preview.rs` 3, `tests/apply.rs` 8, `tests/bounds.rs` 5,
`tests/recovery.rs` 4 (new this revision), `tests/stale_identity.rs` 2
(new this revision)) + 1 doctest = 51 total, 0 unit tests inside `src/`
(all behavior is exercised at the public API). Rev 2's coverage
(malformed-input/normalization, Unicode filenames, preview classification,
no-clobber apply, both preview-to-apply races, bundle bounds) is unchanged
and still passing; see the rev 2 section below for what each of those
proves. New this revision: `tests/recovery.rs` proves batch rollback at
the first/middle/last of three writes plus overwritten-conflict restore;
`tests/malformed_and_unicode.rs`'s two additions prove the
path-of-only-separators and Unicode-normalization-collision adversarial
bounds; `tests/stale_identity.rs` proves the deletion case of stale-identity
acceptance, standalone and combined with batch rollback.

## Validation

rustc/cargo 1.98.1, this machine:
- `cargo build --manifest-path crates/project-bundle/Cargo.toml`: clean.
- `cargo test --manifest-path crates/project-bundle/Cargo.toml`: 51 passed
  (50 integration + 1 doctest), 0 failed.
- `cargo clippy --manifest-path crates/project-bundle/Cargo.toml --all-targets -- -D warnings`:
  clean, 0 warnings.

Exact tested commit SHA (the commit whose `crates/project-bundle` tree the
above three commands were run against, after the rev 3 `origin/main`
merge): `10b4f4193a5d2dc41a4ce38b976159e08e70a8f3`.
(Rev 2's tested SHA for reference: `ddd445d421dae99f80c9bed37e253b3c6c69c82e`.
Rev 1's: `5a37954d2ed8f5e73379e7d31381bcfefcd8c6c2`.)

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
- `BundleError::AmbiguousPath`'s Unicode-normalization-collision detection
  (`ProjectRoot::canonical_identity`) is filesystem-identity-based
  (`fs::canonicalize` equality), not a from-scratch NFC/NFD table — this
  crate took no new dependency to build one, and no `unicode-normalization`
  crate was available to add (network access to crates.io was blocked in
  this environment; adding an unverified new external dependency to a
  security-adjacent crate on the strength of an untested fetch was judged
  the wrong tradeoff against the same detection achieved with what is
  already reused here). Consequence: it only fires where the actual
  filesystem folds two spellings together — the concrete hazard this task
  named — not for two Unicode-canonically-equivalent paths on a filesystem
  that keeps them genuinely distinct (correctly, since there they are not
  the same file). It also incidentally catches any other same-file
  aliasing a filesystem folds (e.g. two case variants on a case-insensitive
  volume), which is a reasonable bonus, not a claim of Unicode correctness.

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

Resource: allocation `daniel-claude20x-shared`; no purchases.

Updated: 2026-09-12T16:53:23Z
