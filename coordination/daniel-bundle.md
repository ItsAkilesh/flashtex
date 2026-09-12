# daniel-bundle handoff

Agent / task / branch: daniel-bundle (FlashTeX agent) / FT-043 "project export
bundle" / `agent/daniel-bundle/project-bundle`
State: ready for integration (new, standalone crate; zero consumers yet).
Owned paths: `crates/project-bundle/**`, `coordination/daniel-bundle.md`
Main integrated through: `3423970b300e878062f2a124fb8004545626200b` (base;
`origin/main` at assignment time). No other crate touched; no workspace root
`Cargo.toml` created (each crate here builds standalone, matching existing
siblings such as `crates/project-files`).

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

25 integration tests (`tests/rooted.rs` 10, `tests/no_discovery.rs` 2,
`tests/determinism.rs` 4, `tests/malformed_and_unicode.rs` 9) + 1 doctest = 26
total, 0 unit tests inside `src/` (all behavior is exercised at the public
API). Malformed-input coverage: empty path, NUL byte, `//`, trailing `/`,
`.` component, bare `..`, duplicate entries. Unicode coverage: non-ASCII
filenames (`café.tex`, `日本語のファイル.tex`, an emoji filename) round-tripping
content and participating correctly in byte-order sorting, plus a non-ASCII
directory component.

## Validation

rustc/cargo 1.98.1, this machine:
- `cargo build`: clean.
- `cargo test`: 26 passed, 0 failed.
- `cargo clippy --all-targets -- -D warnings`: clean, 0 warnings.

Exact tested commit SHA (the commit whose `crates/project-bundle` tree the
above three commands were run against): `PENDING-FILLED-IN-FOLLOWUP-COMMIT`.

## Incomplete behavior

- No archive/serialization format beyond the manifest (no tar/zip writer) —
  out of scope for this lane; `BundleFile::contents` gives a consumer
  everything needed to write one.
- No Windows-specific path handling (drive letters, `\` separator); paths
  are validated by manual `/`-splitting, matching the Unix dev/CI target.
- Not wired into any consumer crate — this is an additive, standalone crate
  per the assignment; a caller integration (e.g. from `project-files`'s
  graph) would need its own follow-up and is not claimed here.

## Needs from others

None to build; a consumer wanting a bundle populated from a `ProjectGraph`
(in `crates/project-files`) would supply the entry list itself — this crate
intentionally does no discovery of its own.

## Peer revisions reviewed

Base `origin/main` at `3423970b300e878062f2a124fb8004545626200b`: reviewed
`crates/project-files/src/path.rs` and `src/save.rs` for their existing
rooted-read precedent (`ProjectPath`, `RootedRead`, `ProjectRoot`) to keep
this crate's error semantics and terminology consistent without depending on
that crate (this lane's `dependencies` are `[]` by assignment). No file under
`crates/project-files` was edited.

Resource: allocation `daniel-claude20x-shared`; no purchases.

Updated: 2026-09-12
