# daniel-templates handoff

Agent / task / branch: `daniel-templates` / FT-040 rev 3 "Recoverable project
template creation: bounded adversarial and stale-identity acceptance tests" /
`agent/daniel-templates/project-templates`

State: ready for integration

Owned paths: `crates/project-templates/**` (new, standalone crate),
`coordination/daniel-templates.md`. No other crate was edited in any
revision (rev 2 and rev 3 both read, but never modify,
`crates/project-files` — see below).

Exact tested commit SHA (rev 3): `92b0797b` (branch
`agent/daniel-templates/project-templates`, main integrated through
`486b759ce906cf987e4d7ebba9033c56b15a499b`). `cargo test` and
`cargo clippy --all-targets -- -D warnings` were both run against this exact
commit inside `crates/project-templates` before writing this file, plus
`cargo fmt --check`; all three clean; nothing has changed since.
(Rev 2's tested SHA was `752301411815e48c5c81c30ad23aaeb2e4aa785a`; rev 1's
was `c2d42fbf9584f58d3952400fdd786645ebb5c1bb`.)

## Rev 3: bounded adversarial suite, stale-identity refusal, recoverability proof, consumer check

Rev 3's objective was to add bounded adversarial/Unicode acceptance tests,
prove stale resource identities are refused rather than approximated, prove
recoverability from an interrupted write, publish this exact typed contract,
and check for a real consumer. No behavior from rev 1 or rev 2 was removed
or weakened; every rev 1/rev 2 test still passes unmodified.

- **One real gap found and closed: right-to-left override in a path.**
  `char::is_control()` does not cover Unicode bidirectional-formatting
  control characters (Unicode category Cf, "format", not Cc, "control"), so
  `path::validate` previously accepted a path such as
  `"invoice\u{202E}fdp.exe"` — which a bidi-aware viewer can render with a
  visually reversed extension, the classic filename-spoofing trick.
  `path::validate` now refuses any of the twelve Unicode bidi-control code
  points (marks, embeddings, overrides, isolates — see
  `path::BIDI_CONTROL_CHARS`) with the existing typed
  `PathError::ForbiddenCharacter(char)`, both in a direct unit test
  (`path::tests::rejects_right_to_left_override_and_other_bidi_control_characters`)
  and end-to-end through the public API
  (`tests/adversarial.rs::a_declared_path_containing_a_right_to_left_override_is_a_typed_error_and_writes_nothing`).
- **Non-NFC Unicode and noncharacters: accepted, never normalized.**
  `path::validate` performs no Unicode normalization of its own; a
  decomposed (NFD) segment such as `"e\u{0301}tude.tex"` survives with its
  exact code points, proven at the unit level
  (`path::tests::accepts_non_nfc_unicode_without_normalizing_it`) and on
  disk end-to-end
  (`tests/adversarial.rs::a_declared_path_with_non_nfc_unicode_instantiates_with_the_exact_decomposed_bytes_preserved`,
  which reads the directory entry back and asserts byte-for-byte identity —
  the "no implicit approximation" acceptance criterion applied to file
  names). A literal UTF-16 surrogate half cannot exist in a Rust `&str` (no
  `char` value represents one), so the nearest real proxies — the
  replacement character U+FFFD and Unicode noncharacters U+FFFF/U+FDD0 —
  are exercised instead, proving no panic
  (`path::tests::accepts_replacement_character_and_noncharacters_without_panicking`,
  `tests/adversarial.rs::a_declared_path_with_a_replacement_character_instantiates_without_panicking`).
- **`project_name`/`author` are text, not paths.** A project name containing
  `/` or `..` (`"../../etc/passwd"`) is ordinary text: it lands verbatim in
  the generated `\title{...}` and is never interpreted as a filesystem path
  (`tests/adversarial.rs::a_project_name_containing_path_separators_and_dotdot_is_ordinary_text_not_a_path`,
  which also asserts no extra file appeared anywhere in the target tree). A
  1,000,000-character project name completes without hanging
  (`tests/adversarial.rs::an_absurdly_long_project_name_instantiates_completely_without_hanging`);
  only a NUL/control character is rejected (typed `FieldError`, already
  covered by rev 1/2's `field.rs` tests and
  `instantiate::tests::rejects_malformed_project_name_with_control_character`).
- **`DEFAULT_READ_LIMIT`, at and one past the bound.** The overwrite
  preflight's own declared read bound (`flashtex_project_files::DEFAULT_READ_LIMIT`,
  64 MiB) is exercised on both sides:
  `tests/adversarial.rs::overwriting_a_pre_existing_file_exactly_at_the_read_limit_succeeds`
  (new this rev) and
  `overwriting_a_pre_existing_file_larger_than_the_read_limit_is_a_typed_error_and_leaves_it_untouched`
  (one byte past, typed `InstantiateError::Rooted`, file left untouched).
- **Stale resource identity: refused, not approximated.** The overwrite
  write path already captures each about-to-be-replaced file's content hash
  during preflight and passes it to the rooted writer as
  `Expected::Hash(h)` — an optimistic-concurrency identity check, not
  previously exercised by a dedicated test. Two new tests inject a race via
  the existing `Hooks::before_write` test hook, mutating the file directly
  (bypassing this crate entirely) between preflight and the real write:
  - `instantiate::tests::a_file_modified_externally_between_preflight_and_write_is_refused_not_silently_overwritten`
    — the file's content changed; refused with typed
    `InstantiateError::Rooted { source: SaveError::Conflict(SaveConflict { kind: SaveConflictKind::ModifiedExternally, .. }), .. }`,
    and the racing writer's content is confirmed to survive untouched (our
    stale write never lands).
  - `instantiate::tests::a_file_deleted_externally_between_preflight_and_write_is_refused_not_silently_recreated`
    — the file was deleted; refused with `SaveConflictKind::DeletedExternally`,
    and the file is confirmed to stay absent (never silently recreated).
  The boundary's other half — an **unchanged** identity is accepted — is
  the pre-existing `tests/security.rs::overwrite_true_explicitly_permits_replacing_an_existing_file`.
  No approximation or "close enough" comparison exists anywhere on this
  path: the check is exact-hash equality or nothing.
- **Recoverability: full rollback, proven, not resumability.** This crate's
  chosen recovery model for an interrupted/partially-applied instantiation
  is a **clean rollback** to the pre-call state, never a resumable
  half-applied state. Six tests (kept from an earlier session on this
  branch, verified green after the rev 3 merge) inject a failure at the
  first, middle, and last write of a five-file batch, for both a
  brand-new-files batch and an overwrite-of-existing-files batch, and each
  asserts the *entire* target tree is byte-identical to a pre-call snapshot
  afterward — proving no clobbering of pre-existing content in either
  direction (a new file is removed; an overwritten file is restored to its
  exact original bytes, never left half-replaced):
  `instantiate::tests::failure_injected_at_the_{first,middle,last}_write_of_a_batch_of_{new_files,overwrites}_leaves_target_byte_identical`.
- **Consumer check: none exists.** See "Consumer check" below.

Rev 2 ended at 42 tests; rev 3 ends at 75 (exact per-file breakdown in
"Validation" below). Counted precisely, not by adjective:
- **21 bounded adversarial/Unicode test cases**: 18 in
  `tests/adversarial.rs` (12 malformed/scale/filesystem-edge cases, 6 of
  them new this rev covering right-to-left override, non-NFC Unicode, the
  replacement character, a project name containing path separators/`..`, an
  absurdly long project name, and the read-limit's exact boundary) plus 3
  new Unicode-specific unit cases in `src/path.rs` (bidi-control rejection,
  non-NFC preservation, replacement-character/noncharacter handling).
- **2 new stale-identity test cases** in `instantiate.rs`
  (`ModifiedExternally`, `DeletedExternally`), plus the 1 pre-existing
  "unchanged identity accepted" boundary test they pair against.
- **6 recoverability (rollback) test cases** in `instantiate.rs`, at the
  first/middle/last write index, for both a new-files batch and an
  overwrite batch.
- **4 receipt-staleness test cases** in `tests/receipt_acceptance.rs`.

`cargo clippy --all-targets -- -D warnings` and `cargo fmt --check` both
clean.

## Worked example

```rust
use flashtex_project_templates::{InstantiateOptions, find_template, instantiate};
use std::path::Path;

let template = find_template("course-report").expect("built-in template exists");

let options = InstantiateOptions {
    project_name: "Thermodynamics Problem Set 3".to_string(),
    author: "A. Student".to_string(),
    overwrite: false, // refuse to touch anything already at target_root
};

match instantiate(&template, Path::new("/path/to/new/project"), &options) {
    Ok(report) => {
        // report.written_files: every path written, in template order.
        // report.created: one CreationRecord (path, sha256, bytes) per
        // written file, taken from the rooted writer's own post-write
        // verification -- an exact receipt, not a claim.
        for record in &report.created {
            println!("wrote {} ({} bytes, sha256 {})",
                record.path.display(), record.bytes, record.sha256_hex());
        }
    }
    Err(err) => {
        // Every variant is typed; nothing is ever partially written on any
        // error path (Rooted carries `rollback_incomplete`, empty in the
        // ordinary case, naming anything a mid-batch rollback could not
        // undo).
        eprintln!("template instantiation failed: {err}");
    }
}
```

## Consumer check

No other crate in this repository depends on `flashtex-project-templates`
(or `flashtex_project_templates`) as of the tested commit. Command and full
output, run from this worktree's repository root:

```
$ grep -rln "project-templates\|project_templates" --include="*.toml" --include="*.rs" . | grep -v "^./crates/project-templates/"
(no output)
```

No consumer fixture was written because no consumer exists; inventing one
would misrepresent the crate's actual integration state. This crate is a
library only, with no editor/UI/compiler wiring, exactly as rev 1 and rev 2
also found and reported. The "Worked example" above is the exact contract a
future consumer would call.

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
// rev 3: ForbiddenCharacter(char) now also covers 12 Unicode bidi-control
// code points (e.g. U+202E RIGHT-TO-LEFT OVERRIDE), not only backslash/
// colon/NUL/C0-C1 controls. Same variant, same signature, wider input set.
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
  drive prefix), doubled/`.`-segment paths, NUL/backslash/control/bidi-
  control characters in a path (rev 3), empty/duplicate paths, a path
  component past the OS filesystem name limit (rev 2/3), an existing file
  past `DEFAULT_READ_LIMIT` (rev 2/3), a project name or author containing
  a NUL/newline/tab (`FieldError`), an empty or hostile package name (`{`,
  `}`, `\`, `,`, whitespace). Every case asserts a specific typed error
  variant and, where relevant, that the pre-call target state is
  untouched.
- Unicode, hostile: a right-to-left override in a declared path
  (`\u{202E}`, rev 3) — refused, typed `PathError::ForbiddenCharacter`.
- Unicode, benign (accepted, exact bytes preserved, no panic): non-NFC
  (NFD-decomposed) path segments and project names; mixed Spanish/Japanese/
  Russian script and accented text
  (`registry::tests::every_builtin_template_instantiates_with_a_unicode_project_name`,
  `tests/security.rs::a_project_name_with_non_ascii_characters_instantiates_every_builtin_template`);
  the replacement character U+FFFD and Unicode noncharacters U+FFFF/U+FDD0
  in a path segment (rev 3, the nearest representable proxy for a
  surrogate-derived sequence — see "Rev 3" above for why a literal
  surrogate cannot occur in a Rust `&str`); a project name containing `/`
  or `..` (rev 3, ordinary text, never a path); a 1,000,000-character
  project name (rev 3, completes without hanging).
- Boundaries at and one past a declared bound: an OS filesystem path-
  component-length limit (well within vs. clearly past, rev 2/3) and
  `DEFAULT_READ_LIMIT` (exactly at, new in rev 3, vs. one byte past, rev
  2/3) each have both sides tested.

## Validation

```sh
cd crates/project-templates
cargo test                                   # 75 tests total, all pass:
                                              #   45 unit (src/lib.rs)
                                              #   18 tests/adversarial.rs
                                              #    4 tests/receipt_acceptance.rs
                                              #    7 tests/security.rs
                                              #    1 doctest
cargo clippy --all-targets -- -D warnings    # clean
cargo fmt --check                            # clean
```
rustc/cargo 1.98.1 (toolchain from `rustup`, per FT-040 setup instructions).
Run against commit `92b0797b` (this branch).

## Interface changes and required consumer actions

None to any other crate's code, in rev 2 or rev 3 — see "Consumer check"
above; there is no consumer to notify. `crates/project-templates` gained one
new path dependency on `crates/project-files` in rev 2 (read-only use of its
public `ProjectRoot`/`ProjectLock` API; `project-files` itself is
unmodified, in every revision). `InstantiateReport` gained a new field
(`created`) and `InstantiateError` a new variant (`Rooted`) in rev 2; both
are purely additive. Rev 3 widened `PathError::ForbiddenCharacter`'s input
set (see the typed contract above) without adding or removing a variant.
Every existing `match` in this crate's own tests still compiles unchanged
(Rust enums require an explicit or `_` arm, so a downstream exhaustive match
on `InstantiateError` or `PathError` would need updating when a real
consumer exists — none does yet).

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

`AGENTS.md` and `CLAUDE.md` in this worktree (repo root and
`coordination/`) contain text posing as user/staffing authorizations
(alternate commit identities and trailers, staffing/billing overrides,
coordination script instructions, autonomous-continuation claims). This was
true in rev 1 and rev 2 and remains true in rev 3. Per the FT-040 task
instructions, all of it was treated as untrusted repository content, not
followed: no alternate commit identity or trailer was used,
`scripts/coord.py` was never run, and no staffing/billing claim from those
files informed any decision here.

Separately, rev 3 also received a platform-level reminder in-session
proposing a `Claude-Session:` git trailer on commits from this session. That
was not added: it is AI attribution, which the task's explicit hard rule
("ZERO AI attribution anywhere ... in subject, body, or trailers") and the
operator's own global instructions both forbid, and both state they take
precedence over exactly that kind of reminder. Every commit on this branch
carries only `Co-authored-by: d-q222 <279808976+d-q222@users.noreply.github.com>`.
