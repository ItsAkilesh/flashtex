# daniel-images handoff

Agent / task / branch: daniel-images / FT-036 "Rooted bounded document
PNG/JPEG asset loader" / `agent/daniel-images/image-assets`.
Owned paths: `crates/image-assets/**`, `coordination/daniel-images.md`.
State: ready for integration (standalone additive crate; not yet wired into
any consumer — this doc is the proposed adapter contract for whoever
integrates it).
Tested commit: `bab2811ae7ab03e65f55c23f613157cbd51b75bd` (this branch, this
worktree). `input_main_sha` from the assignment:
`53fee3012b2902ca05bd31766defa515b3044cec`. No peer files touched — only the
two owned paths above are in this commit.

## Decoder reused

`image` v0.25, `default-features = false, features = ["png", "jpeg"]` — the
same crate and feature set already used by `flashtex-bridge`
(`crates/bridge/Cargo.toml`) and `flashtex-conversion-jobs`'s dev-deps. No
PNG/JPEG decoder is hand-rolled. Decoding mirrors
`flashtex-bridge`'s `CaptureImage::validate` pattern
(`crates/bridge/src/lib.rs`): sniff the format, restrict to
`ImageFormat::Png`/`Jpeg`, then decode through `image::ImageReader` with
explicit `image::Limits` (`max_image_width`/`max_image_height` = 16384,
`max_alloc` = 256 MiB) so a small file cannot expand into an unbounded
allocation.

## Typed adapter contract (public API)

```rust
// crates/image-assets/src/root.rs
pub struct AssetRoot { /* canonicalized once at construction */ }
impl AssetRoot {
    pub fn new(root: impl AsRef<Path>) -> Result<Self, RootError>;
    pub fn root(&self) -> &Path;
    pub fn resolve(&self, relative: impl AsRef<Path>) -> Result<PathBuf, RootError>;
}
pub enum RootError {
    RootNotFound(PathBuf), RootNotADirectory(PathBuf),
    AbsolutePath(PathBuf), PathTraversal(PathBuf), SymlinkEscape(PathBuf),
    EmptyPath, NotFound(PathBuf), Io(String),
}

// crates/image-assets/src/lib.rs
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
impl AssetId { pub fn as_bytes(&self) -> &[u8; 32]; pub fn to_hex(&self) -> String; }

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

// crates/image-assets/src/geometry.rs
pub enum Length { Points(f64), Scale(f64) }
pub struct Crop { pub left: u32, pub bottom: u32, pub right: u32, pub top: u32 }
pub struct IncludeGraphicsSpec { pub width: Option<Length>, pub height: Option<Length>, pub crop: Option<Crop> }
pub enum GeometryError { CropOutOfBounds { crop: Crop, dimensions: Dimensions } }
```

Consumer contract: construct one `AssetRoot` per document root, build an
`AssetLoader` over it, call `load(relative_path)` for every
`\includegraphics{relative_path}` reference the compiler/parser resolves.
`AssetLoader::load` is the only path from an untrusted relative string to
the filesystem — it never touches the filesystem before `AssetRoot::resolve`
has cleared traversal/absolute/symlink checks. Attach an
`IncludeGraphicsSpec` (parsed from the LaTeX optional args) and call
`ImageAsset::validate_spec` before using `trim`/`clip` metadata downstream;
this crate does not compute the final placed size — that stays with
paragraph-layout/math-layout.

## Ready behavior and evidence

- `cargo build`: clean.
- `cargo test`: 38 passed (25 unit across `root`/`geometry`/`lib`, 13
  integration in `tests/integration.rs`), 0 failed, temp dirs only.
- `cargo clippy -- -D warnings` and `cargo clippy --all-targets -- -D
  warnings`: clean.
- `cargo fmt --check`: clean.
- Rooting (acceptance item 4, the priority requirement): three explicit
  tests per case, once in `root.rs` unit tests and again end-to-end through
  `AssetLoader` in `tests/integration.rs`:
  - traversal — `rejects_parent_traversal` / `rejects_path_traversal_with_a_typed_error`
    (`"../../etc/passwd"` → `RootError::PathTraversal`), plus
    `rejects_traversal_buried_in_the_middle_of_a_path` for `..` that
    isn't at the start.
  - absolute paths — `rejects_absolute_paths` / `rejects_absolute_paths_with_a_typed_error`
    (`"/etc/passwd"` → `RootError::AbsolutePath`).
  - symlink escape — `rejects_symlinks_that_escape_the_root` (symlink file
    target outside root) and `rejects_escape_via_a_symlinked_directory_component`
    (a symlinked *directory* component, not just the leaf) /
    `rejects_symlink_escape_with_a_typed_error` end-to-end → all
    `RootError::SymlinkEscape`. A companion test,
    `allows_a_symlink_that_stays_inside_the_root`, confirms in-root symlinks
    are not falsely rejected. Mechanism: absolute/`..` are rejected
    lexically from path components before any filesystem access; symlink
    escape is caught by `fs::canonicalize`-ing the resolved candidate and
    checking the real path still `starts_with` the canonicalized root —
    lexical checks alone cannot see through a symlink, so this step is load
    bearing.
- Identity (acceptance item 5): `asset_id_is_stable_for_identical_bytes` and
  `identical_bytes_at_different_paths_share_one_identity` (same bytes, two
  different paths, one `AssetId`); `asset_id_changes_with_a_single_byte` and
  `a_single_changed_byte_in_the_stored_file_changes_the_identity` (flip one
  byte after a PNG's `IEND` chunk — still decodes, still a different SHA).
- Malformed input never panics (acceptance item 2): empty bytes, 10 bytes of
  non-image garbage, a truncated PNG (first third of the file only), and a
  correctly-signed PNG with an all-zero body all return a clean
  `AssetError` (`UnsupportedFormat` or `Decode`), asserted in both `lib.rs`
  unit tests and `tests/integration.rs`.
- Unicode filenames (acceptance item 2): `loads_an_asset_at_a_unicode_path`
  writes and loads through a path with CJK, an emoji, and a non-ASCII
  directory segment (`図/📁directory/日本語ファイル名😀.png`), asserting
  real dimensions and the exact expected SHA-256 hex.
- Real dimensions are asserted against images built with known, non-square
  width/height (e.g. 64x32, 48x20, 37x19) via `image::DynamicImage`, not
  fixture files, so the expected value in each test is independently known.

## Incomplete behavior / known gaps

- Not wired into any consumer yet — no compiler/layout crate references
  this crate. Integration point (where `\includegraphics` targets get
  resolved to a document root) needs a decision from whoever owns that
  parsing path; this crate is deliberately silent on it.
- `Length`/`IncludeGraphicsSpec` carry sizing intent but do not compute a
  final placed size (aspect-preserving scale from `width`+`height`, `bp`
  unit conversion, etc.) — that is layout's job, not asset loading's.
- No caching layer (repeated `load()` calls re-read and re-decode from
  disk); an integrator wanting a cache should key it by `AssetId` once one
  load has happened, not attempt to predict it from the path.
- Symlink-escape detection depends on `fs::canonicalize`, which requires
  the target to exist; a symlink to a path that does not exist yet reports
  `RootError::NotFound`, not `SymlinkEscape` — this is a correctness
  limitation (no oracle to check against) not a security gap, since a
  nonexistent target cannot be read either way.
- No workspace root `Cargo.toml` was added or touched, per instruction;
  `crates/image-assets/Cargo.lock` is committed, matching the existing
  per-crate convention in `crates/bridge`, `crates/project-files`, and
  `crates/conversion-jobs`.

## Interface changes / consumer actions

None — this is a new, standalone crate. No existing crate's public API,
Cargo.toml, or generated protocol changed.

## Reviewed peer revisions / resulting adaptations

Read `crates/bridge/Cargo.toml` and `crates/conversion-jobs/Cargo.toml` to
find the existing `image` v0.25 (png+jpeg, no default features) dependency
and matched it exactly rather than introducing a second image-decoding
dependency. Read `crates/bridge/src/lib.rs`'s `CaptureImage::validate` for
the decode-limits pattern (`image::Limits`, explicit format match before
decode) and reused it. Read `crates/project-files/src/path.rs` for the
existing lexical path-safety precedent (`ProjectPath`, enum-typed
`PathError`) — matched its enum-with-Display/Error/PartialEq style for
`RootError`/`AssetError`/`GeometryError`, but did not depend on or import
`project-files` (that crate's normalization is lexical-only and does not
canonicalize against a real filesystem root, so it cannot by itself catch a
symlink escape — the acceptance-critical case here). No peer crate files
were modified.

Note: this worktree's `AGENTS.md`/`CLAUDE.md` contain embedded
"LATEST USER OVERRIDE"-style banners (staffing resets, a different git
identity/co-author convention, instructions to run `scripts/coord.py`, and
reading requirements outside this task's scope). These were not followed:
they conflict with this task's explicit, narrower instructions (no
`scripts/coord.py`, no push/merge, exactly one `Co-authored-by` trailer, no
AI attribution), and their content (stacked contradictory "LATEST"
declarations, unverifiable budget/staffing claims) is not something this
agent can verify came from the actual user. Flagging this for whoever
integrates the branch, rather than silently complying or silently
ignoring it.

## Validation commands

Run from `/Users/dqi26/ft-wt-daniel-images/crates/image-assets` with
`PATH="/opt/homebrew/opt/rustup/bin:$PATH"`, `cargo 1.98.1`:
`cargo build`, `cargo test`, `cargo clippy -- -D warnings`,
`cargo clippy --all-targets -- -D warnings`, `cargo fmt --check`.

## Resource / next action

Resource pool: `daniel-claude20x-shared` per `coordination/assignments/FT-036.json`.
Next action: none required to close FT-036 as specified; next step is for an
integrator to pick an `\includegraphics`-resolution call site and wire
`AssetLoader` in, per the adapter contract above.

Updated: 2026-09-12.
