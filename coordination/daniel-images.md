# daniel-images handoff

Agent / task / branch: daniel-images / FT-036 revision 2, "Use existing rooted
project-files reader for bounded image assets" / `agent/daniel-images/image-assets`.
Owned paths: `crates/image-assets/**`, `coordination/daniel-images.md`,
`coordination/agents/daniel-images.json`.
State: ready for integration.
Tested commit: `988e4bc78aa131a3be0e16309edae1da2497ce20` (HEAD of the code
change on this branch, this worktree; `cargo test` and `cargo clippy
--all-targets -- -D warnings` re-verified clean at exactly this SHA, both
before and after committing). `input_main_sha` from the FT-036 rev-2
assignment: `5499e419b09868e2e248c413351a5646db74fac1`. Merged through
`origin/main` at `e5901797e8a7ebdd8d714ecdee6793e1097515a9` (two merges during
this revision — `origin/main` moved twice while this worktree ran, both via
the shared `.git` remote-tracking refs; neither touched `crates/image-assets`
or `crates/project-files`, so no conflicts). Only `crates/image-assets/**`
plus this doc and the ack JSON are touched by this revision's commits.

## What changed from revision 1

Revision 1 hand-rolled its own rooted resolver in `root.rs`: canonicalize the
configured root once, then for each request canonicalize the joined candidate
and check it still starts with the canonical root. That works, but it is a
second implementation of exactly the filesystem primitive
`crates/project-files` already has, and it has a narrower TOCTOU window than
that crate's: canonicalize-then-compare is two separate filesystem
observations with a gap between them.

Revision 2 replaces that with a thin adapter over
`flashtex_project_files::ProjectRoot` (consumed as a dependency, not edited):

- `flashtex_project_files::ProjectPath::normalize` does the lexical checks
  (absolute, `..` traversal, forbidden characters) before anything touches
  the filesystem.
- `flashtex_project_files::ProjectRoot::read` does the rooted, symlink-
  refusing walk: it opens each path component with `openat(O_NOFOLLOW)`, so
  **no** component — not just the last — may be a symlink, and it checks
  each walked directory's `..` by device/inode against the handle it was
  opened from. Resolution and the bounded read happen in one call; there is
  no separate "resolve to a path" step for a symlink swap to land in between.

`crates/image-assets/src/root.rs` no longer calls `std::fs::canonicalize`,
`std::fs::metadata`, or `std::fs::read` anywhere in production code — those
are now entirely `flashtex_project_files`'s job. `AssetLoader::load` in
`lib.rs` also lost its own `std::fs::metadata`/`std::fs::read` pair; the
file-size bound is now enforced by `ProjectRoot::read`'s own limit parameter
and surfaced as `RootError::TooLarge` → `AssetError::TooLarge`.

### Coverage of image-assets' own rooting guarantees: full, plus one gap-in-reverse

`ProjectRoot` does not just match this crate's three original guarantees
(reject absolute paths, reject `..` traversal, reject symlink escapes with
the in-root positive control still allowed) — it exceeds the third one. The
old check permitted a symlink whose resolved target happened to land back
inside the root (tested by `allows_a_symlink_that_stays_inside_the_root` in
revision 1). `ProjectRoot` refuses **every** symlink component outright, with
no exception for one that would have stayed in-root. That test is replaced
by `rejects_a_symlink_even_when_it_stays_inside_the_root`, which asserts the
new, stricter rejection. This is a tightening, not a regression: nothing this
crate used to reject is now allowed; something it used to allow is now
rejected. No independent symlink or traversal check had to be kept layered
on top — `ProjectRoot` already fully subsumes what `root.rs` used to do by
hand, so there is no security gap to cover.

The one asymmetry, also not a security gap: `ProjectRoot` exposes no bare
"resolve to a path" primitive, only "read the bounded bytes." That is a
deliberately safer shape than a resolve-then-reopen API (no seam for a
symlink swap to land in), and it is exactly what this crate needs anyway
(bytes to decode), so it is a non-issue rather than something worked around.

`RootError` gained five new variants — `ForbiddenCharacter`, `NotADirectory`,
`NotARegularFile`, `NotUtf8`, `TooLarge { limit, size }` — to carry through
`ProjectPath`'s and `ProjectRoot`'s more specific rejections instead of
collapsing them into the old generic `Io(String)`. `AssetRoot::resolve(...)
-> PathBuf` is gone (nothing outside this crate called it — checked
repo-wide); its replacement, `AssetRoot::read_bounded`, is crate-private,
since `AssetLoader::load` is this crate's only public entry point from an
untrusted relative path to the filesystem, same as revision 1.
`AssetLoader`, `AssetError`, `AssetId`, `AssetFormat`, `Dimensions`,
`ImageAsset`, and the `geometry` module are all unchanged.

## New tests for revision 2

- `replacing_the_file_at_the_same_path_changes_the_content_identity`
  (`src/lib.rs`): loads an asset, overwrites the same relative path with a
  different image (different dimensions, different bytes), reloads, and
  asserts the `AssetId` changed and the new dimensions/bytes are the
  replacement's, not the original's.
- `decode_bounded_rejects_dimensions_over_the_configured_axis_limit` and
  `asset_loader_rejects_oversized_images_without_decoding_them`
  (`src/lib.rs`): a PNG whose header declares one axis one pixel past
  `AssetLoader::DEFAULT_MAX_PIXELS_PER_AXIS` (16384) is rejected with
  `AssetError::Decode`, both at the `decode_bounded` unit level and through
  the full `AssetLoader::load` path — never silently decoded. The fixture
  keeps the other axis at 1px so the test stays cheap; `image`'s `Limits`
  check rejects the header before any large allocation.
- `read_bounded_rejects_files_over_the_limit` (`src/root.rs`): confirms the
  new `RootError::TooLarge` path this rewrite introduced.

## Typed adapter contract (public API, updated)

```rust
// crates/image-assets/src/root.rs
pub struct AssetRoot { /* wraps flashtex_project_files::ProjectRoot */ }
impl AssetRoot {
    pub fn new(root: impl AsRef<Path>) -> Result<Self, RootError>;
    pub fn root(&self) -> &Path;
    // read_bounded is pub(crate); AssetLoader::load is the public entry point.
}
pub enum RootError {
    RootNotFound(PathBuf), RootNotADirectory(PathBuf),
    AbsolutePath(PathBuf), PathTraversal(PathBuf), SymlinkEscape(PathBuf),
    EmptyPath, NotFound(PathBuf), NotADirectory(PathBuf),
    NotARegularFile(PathBuf), ForbiddenCharacter(PathBuf, char),
    NotUtf8(PathBuf), TooLarge { limit: u64, size: u64 }, Io(String),
}

// crates/image-assets/src/lib.rs -- unchanged from revision 1
pub struct AssetLoader { /* AssetRoot + byte-size bound */ }
impl AssetLoader {
    pub fn new(root: AssetRoot) -> Self;
    pub fn with_max_file_bytes(self, max_file_bytes: u64) -> Self;
    pub fn root(&self) -> &AssetRoot;
    pub fn load(&self, relative: impl AsRef<Path>) -> Result<ImageAsset, AssetError>;
}
pub enum AssetError {
    Root(RootError), Io(io::Error), TooLarge { limit: usize, actual: u64 },
    Empty, UnsupportedFormat, Decode(String),
}
pub struct AssetId([u8; 32]);            // SHA-256 of raw encoded bytes
pub enum AssetFormat { Png, Jpeg }
pub struct Dimensions { pub width: u32, pub height: u32 }
pub struct ImageAsset { /* id, format, dimensions, raw bytes */ }
impl ImageAsset {
    pub fn id(&self) -> AssetId;
    pub fn format(&self) -> AssetFormat;
    pub fn dimensions(&self) -> Dimensions;   // real decoded pixel size
    pub fn bytes(&self) -> &[u8];             // raw encoded bytes
    pub fn validate_spec(&self, spec: &IncludeGraphicsSpec) -> Result<(), GeometryError>;
}
// geometry.rs is unchanged from revision 1.
```

Consumer contract is unchanged: one `AssetRoot` per document root, one
`AssetLoader` over it, `load(relative_path)` per `\includegraphics` reference.

## Ready behavior and evidence

- New dependency: `flashtex-project-files = { path = "../project-files" }`
  (path dependency, same convention as `conversion-jobs`, `document-runtime`,
  `font-resources`, `preview-controller`, `rendering-core`). `project-files`
  itself was read, not edited.
- `cargo test` (in `crates/image-assets`): 42 passed (29 unit across
  `root`/`geometry`/`lib`, 13 integration in `tests/integration.rs`), 0
  failed.
- `cargo clippy --all-targets -- -D warnings`: clean.
- No workspace root `Cargo.toml` was created; each crate builds standalone
  from its own directory, same as before.

## Incomplete / not attempted

Nothing outstanding for this revision's stated objective. Not attempted
(out of scope for FT-036): wiring `AssetLoader` into any compiler/parser
consumer — still nobody in the repo depends on `flashtex-image-assets` yet,
confirmed by a repo-wide grep before and after this revision.
