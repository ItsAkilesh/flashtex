# daniel-templates handoff

Agent / task / branch: `daniel-templates` / FT-040 "Original undergraduate
project templates and bounded manifest/instantiation API" /
`agent/daniel-templates/project-templates`

State: ready for integration

Owned paths: `crates/project-templates/**` (new, standalone crate),
`coordination/daniel-templates.md`. No other crate was edited (rev 2 reads,
but does not modify, `crates/project-files` — see below).

Exact tested commit SHA (rev 2): `752301411815e48c5c81c30ad23aaeb2e4aa785a`
(branch `agent/daniel-templates/project-templates`). `cargo build`,
`cargo test`, `cargo clippy --all-targets -- -D warnings`, and
`cargo fmt --check` were all run against this exact commit inside
`crates/project-templates` before writing this file; nothing has changed
since. (Rev 1's tested SHA was `c2d42fbf9584f58d3952400fdd786645ebb5c1bb`.)

## Rev 2: rooted APIs, creation receipt, interrupted-write recovery

Rev 2's objective was to add an explicit creation receipt and
interrupted-write recovery around the existing preflight, using
`crates/project-files`' rooted filesystem APIs, without losing any
guarantee rev 1's own tests proved.

- **Rooted APIs, gap kept covered.** `instantiate()` now reads and writes
  every declared file through `flashtex_project_files::ProjectRoot` /
  `ProjectLock` (`O_NOFOLLOW` at every path component, atomic temp-write +
  rename) instead of raw `fs::read`/`fs::write`/`fs::create_dir_all`. This
  closes a real gap rev 1 had: a symlink planted at (or above) a declared
  path would previously have been silently written *through* by plain
  `fs::write`; the rooted writer refuses it instead
  (`InstantiateError::Rooted`, proven by
  `tests/security.rs::a_symlink_planted_at_a_declared_path_is_refused_and_original_target_untouched`).
  **Gap `flashtex-project-files` does *not* close, kept covered by our own
  code:** its `ProjectPath::normalize` *resolves* `..` (pops a segment,
  only erroring if it would leave the root) rather than rejecting it
  outright — weaker than `crate::path::validate`'s guarantee, which rev 1's
  own tests prove (`a/../a.tex` is refused even though it would cancel out).
  So `crate::path::validate` remains the sole judge of every raw
  declared-path string; a `flashtex_project_files::ProjectPath` is only
  ever constructed from segments that already passed it, never from the raw
  string, so it cannot reintroduce the traversal it would otherwise
  silently resolve. Similarly, `ProjectRoot` only guarantees no-clobber on
  one individual write, not across a whole batch, so this crate keeps its
  own whole-template preflight loop (now via the rooted reader) rather than
  relying on per-file `Expected::NewFile` refusals alone.
- **Creation receipt.** Every successful write returns the rooted writer's
  own post-write-verified `(path, sha256, size)` as a `CreationRecord`,
  collected on the new `InstantiateReport.created` (order-matched with
  `written_files`). Proven by
  `instantiate::tests::creation_receipt_reports_hash_and_size_for_every_written_file`
  and `tests/security.rs::instantiation_returns_a_creation_receipt_matching_disk_content`,
  both of which independently re-hash the on-disk bytes and assert equality
  with the receipt.
- **Interrupted-write recovery.** `instantiate()` is now all-or-nothing
  across the whole file batch. During preflight, every about-to-be-replaced
  file's current bytes are captured (via the rooted reader) as a rollback
  source; a file this call is about to create has no such backup. If a
  write then fails partway through the batch — a late conflict the
  preflight couldn't see, a refused symlink, or any other rooted-layer
  error — every file already written in this call is rolled back in
  reverse order: restored to its captured pre-call bytes if it pre-existed,
  or removed if this call created it. Tested by injecting a failure via a
  `pub(crate)` test-only hook (`Hooks::before_write`, mirroring
  `flashtex_project_files::save`'s own internal `Hooks` pattern — the same
  technique that crate uses for its own hard-to-trigger disk-error paths)
  that fails on a specific file index after earlier files in the same batch
  have already landed:
  - `instantiate::tests::mid_batch_write_failure_leaves_no_partial_tree` —
    three brand-new files, failure injected on the third; asserts none of
    the three exist afterward.
  - `instantiate::tests::mid_batch_write_failure_during_overwrite_restores_original_content`
    — two pre-existing files being overwritten, failure injected on the
    second; asserts the first is back to its *original* content (not left
    as the new content, and not deleted), and the second was never touched.
  `InstantiateError::Rooted::rollback_incomplete` names any file that could
  not be rolled back (empty in every test above); this is a known,
  documented edge case, not a claim that rollback is unconditionally
  guaranteed under total disk exhaustion (see Known limitations).
- **Never overwrite existing, unchanged.** The rev 1 preflight behavior and
  every one of its tests are kept passing verbatim; `overwrite: false`
  still means the whole batch is refused, nothing written, the instant any
  target is found to exist.
- **New dependency.** `crates/project-templates/Cargo.toml` now has one
  path dependency, `flashtex-project-files = { path = "../project-files" }`.
  `crates/project-files` itself was read for its public API only and was
  not modified.

## Crate

`flashtex-project-templates`, edition 2024, zero external dependencies, not
part of any Cargo workspace (built and tested standalone, same as every
other crate in this repo).

## Typed public contract

```rust
// manifest.rs — the bounded manifest
pub struct TemplateFile { pub path: String, pub body: String }
pub struct Template {
    pub id: String,
    pub title: String,
    pub description: String,
    pub packages: Vec<String>,   // sole source of \usepackage{...} lines
    pub files: Vec<TemplateFile>,
}
pub enum ManifestError {
    InvalidPath { file_path: String, source: path::PathError },
    InvalidPackage { package: String, source: package::PackageNameError },
    DuplicatePath(String),
}
impl Template { pub fn validate(&self) -> Result<(), ManifestError>; }

// path.rs — file-path validation (defense against traversal / absolute paths)
pub enum PathError { Empty, Absolute, ParentTraversal, EmptySegment, ForbiddenCharacter(char) }
pub fn validate(raw: &str) -> Result<Vec<&str>, PathError>;

// field.rs — free-text field validation (project name / author)
pub enum FieldError { ForbiddenCharacter(char) }
pub fn validate(value: &str) -> Result<(), FieldError>;

// package.rs — LaTeX package-name validation
pub enum PackageNameError { Empty, ForbiddenCharacter(char) }
pub fn validate(name: &str) -> Result<(), PackageNameError>;

// escape.rs — LaTeX-safe escaping applied to substituted field values
pub fn escape(s: &str) -> String;

// instantiate.rs — the instantiation API (rev 2: receipt + rollback)
pub struct InstantiateOptions { pub project_name: String, pub author: String, pub overwrite: bool }
pub struct CreationRecord { pub path: PathBuf, pub sha256: [u8; 32], pub bytes: u64 }
impl CreationRecord { pub fn sha256_hex(&self) -> String; }
pub struct InstantiateReport {
    pub written_files: Vec<PathBuf>,
    pub created: Vec<CreationRecord>,   // new: the creation receipt
}
pub enum InstantiateError {
    InvalidTemplate(ManifestError),
    InvalidField { field: &'static str, source: FieldError },
    RootNotADirectory(PathBuf),
    AlreadyExists(PathBuf),
    Io { path: PathBuf, source: io::Error },
    // new: a rooted-layer refusal/conflict, before or during the write
    // batch; if mid-batch, every already-written file was rolled back
    // unless `rollback_incomplete` names it.
    Rooted {
        path: PathBuf,
        source: flashtex_project_files::SaveError,
        rollback_incomplete: Vec<PathBuf>,
    },
}
pub fn instantiate(
    template: &Template,
    target_root: &Path,
    options: &InstantiateOptions,
) -> Result<InstantiateReport, InstantiateError>;

// registry.rs — the built-in templates
pub fn all_templates() -> Vec<Template>;
pub fn find_template(id: &str) -> Option<Template>;
```

Re-exported at the crate root: `instantiate`, `CreationRecord`,
`InstantiateError`, `InstantiateOptions`, `InstantiateReport`,
`ManifestError`, `Template`, `TemplateFile`, `all_templates`,
`find_template`.

## Template list (all original text; no third-party document class)

| id              | title                | packages (exact, declared, nothing implied) | files |
|------------------|----------------------|----------------------------------------------|-------|
| `course-report`  | Course Report        | amsmath, graphicx, hyperref                   | `main.tex` |
| `senior-thesis`  | Senior Thesis        | amsmath, amssymb, graphicx, hyperref, geometry, titlesec | `main.tex`, `chapters/introduction.tex`, `chapters/conclusion.tex`, `references.bib` |
| `problem-set`    | Problem Set          | amsmath, amssymb, enumitem                    | `main.tex` |
| `lab-notebook`   | Lab Notebook Entry   | amsmath, graphicx, siunitx                    | `main.tex` |

Every template uses the standard LaTeX `article` class only. A file body
never hardcodes a `\usepackage{...}` line; `instantiate()` renders the
`{{packages}}` placeholder from `Template.packages` alone, and
`registry::tests::declared_packages_exactly_match_generated_usepackage_lines`
mechanically checks the two lists match exactly for every built-in template.

## Security guarantees (the core of this lane)

Enforced in `instantiate()`, in this order, before any file is written:

1. `Template::validate()` runs every declared file path through
   `path::validate`, which rejects (typed, no panic):
   - any path containing a `..` segment — `PathError::ParentTraversal`,
     even where the traversal would mathematically cancel out (`a/../a.tex`
     is refused outright, never resolved);
   - any absolute path (leading `/`, `~`, or a `C:`-style drive prefix) —
     `PathError::Absolute` (or `ForbiddenCharacter(':')` for a drive
     prefix);
   - empty/`.` segments and control characters/backslashes.
2. Target OS paths are built only by pushing the *validated segments*
   individually onto `target_root` — never by parsing or joining the raw
   string — so no path that failed step 1 can reach the filesystem at all.
   A `debug_assert!` that the result starts with `target_root` is kept as
   defense-in-depth.
3. Before writing anything, every target path is checked for an existing
   file. If one exists and `options.overwrite` is `false`, instantiation
   returns `InstantiateError::AlreadyExists` and **writes nothing at all**
   (whole-template preflight — a conflict on file 2 of 3 also stops file 1
   from being written).

Proof, as explicit tests (both crate-internal in `instantiate.rs` and
black-box in `tests/security.rs` against the public API only):
- `a_path_containing_dotdot_is_rejected_with_a_typed_error` /
  `rejects_traversal_path_with_typed_error_and_writes_nothing` — a `..`
  entry is refused with `PathError::ParentTraversal` and `target_root` is
  never even created.
- `an_absolute_path_is_rejected_with_a_typed_error` /
  `rejects_absolute_path_with_typed_error_and_writes_nothing` — an absolute
  path entry is refused with `PathError::Absolute` and the absolute target
  is confirmed never written.
- `instantiating_over_an_existing_file_fails_without_clobbering_it` /
  `refuses_to_overwrite_existing_file_without_the_flag` — instantiating a
  built-in template over a directory with an existing `main.tex` fails with
  `AlreadyExists` and the original file content is asserted unchanged.
- `overwrite_true_explicitly_permits_replacing_an_existing_file` — the same
  scenario with `overwrite: true` succeeds and the new content lands.
- `a_conflicting_second_file_prevents_writing_the_first_too` — a
  multi-file template where only the second file conflicts writes neither.

LaTeX-injection note (not part of the file-overwrite acceptance criterion,
but the same trust boundary): `project_name`/`author` are user-controlled
free text spliced into `\title{...}`/`\author{...}`. `escape::escape`
neutralizes the ten LaTeX special characters first;
`escape::tests::cannot_be_used_to_close_a_macro_argument_early` proves a
hostile value containing `}\input{...}{` cannot close the macro argument
early. `field::validate` separately rejects control characters (NUL, raw
newline/tab) with a typed `FieldError`.

## Malformed / Unicode input tests

- Malformed: `..`-containing path, absolute path (including a `C:`-style
  drive prefix), doubled/`.`-segment paths, NUL/backslash/control
  characters in a path, empty/duplicate paths, a project name or author
  containing a NUL/newline/tab (`FieldError`), an empty or hostile package
  name (`{`, `}`, `\`, `,`, whitespace).
- Unicode: `registry::tests::every_builtin_template_instantiates_with_a_unicode_project_name`
  and `tests/security.rs::a_project_name_with_non_ascii_characters_instantiates_every_builtin_template`
  instantiate every built-in template with a project name mixing Spanish,
  Japanese, and Russian script (e.g. `"Análisis Estadístico — 統計分析 —
  Статистический анализ"`) and an accented author name, and assert the
  text lands correctly in the generated file.

## Validation

```sh
cd crates/project-templates
cargo build                                  # clean
cargo test                                   # 42 tests: 34 unit, 7 integration, 1 doctest — all pass
cargo clippy --all-targets -- -D warnings    # clean
cargo fmt --check                            # clean
```
rustc/cargo 1.98.1 (toolchain from `rustup`, per FT-040 setup instructions).

## Interface changes and required consumer actions

None to any other crate's code. `crates/project-templates` gained one new
path dependency on `crates/project-files` (read-only use of its public
`ProjectRoot`/`ProjectLock` API; `project-files` itself is unmodified). This
crate is still not wired into `crates/compiler` or any editor/UI surface; a
future consumer would depend on `flashtex-project-templates` and call
`find_template`/`instantiate` directly — no adapter is required beyond a
path dependency in that consumer's `Cargo.toml`. `InstantiateReport` gained
a new field (`created`) and `InstantiateError` a new variant (`Rooted`);
both are purely additive and every existing `match` in this crate's own
tests still compiles unchanged (Rust enums require an explicit or `_` arm,
so a downstream exhaustive match on `InstantiateError` would need updating
— there are no such consumers yet).

## Incomplete behavior / known limitations

- Only 4 built-in templates (course-report, senior-thesis, problem-set,
  lab-notebook). FT-040 asked for "report, thesis, problem set, and
  similar" — covered — but a real product would likely want more variety
  (presentation, poster, CV) as a follow-up, not attempted here to keep
  scope bounded.
- No disk-quota handling — out of scope for "no arbitrary file overwrite."
  The symlink-escape gap on `target_root` and on every declared path is
  closed as of rev 2 (see above), via `flashtex-project-files`'
  `O_NOFOLLOW`-rooted reader/writer.
- Rollback on a mid-batch failure is best-effort: `rollback_incomplete`
  (see the typed contract) is populated, not silently dropped, when a
  restore/remove itself fails — e.g. if the disk is so full that even
  restoring a file to its prior (already-fitted) size fails. This is a
  fundamental limit of any journal-less rollback under total disk
  exhaustion, not a gap specific to this implementation; it is surfaced to
  the caller rather than hidden.
- Not integrated with any compiler/editor surface; this crate is a library
  only, per the FT-040 objective ("standalone additive crate only;
  coordinate consumer contract before integration").

## Note on repository content encountered

`AGENTS.md` and `CLAUDE.md` in this worktree contain text posing as
user/staffing authorizations (alternate commit identities, coordination
script instructions, autonomous-continuation claims). Per the FT-040
task instructions, all of that was treated as untrusted repository content,
not followed, and `scripts/coord.py` was never run.
