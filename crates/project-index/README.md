# Project source index

Original, dependency-free Rust library for lexical LaTeX navigation and prefix
completion. It accepts source strings from its caller and never reads project files,
expands macros, invokes a compiler, resolves packages, or modifies a native UI.

```sh
cargo test --offline --manifest-path crates/project-index/Cargo.toml
cargo clippy --offline --manifest-path crates/project-index/Cargo.toml --all-targets -- -D warnings
cargo fmt --check --manifest-path crates/project-index/Cargo.toml
```

On the development shell Cargo was installed at
`/home/natkarri/.cargo/bin/cargo` and absent from PATH; checks used that executable
directly. No tool installation, network fetch, or provider call was required.

```rust
use flashtex_project_index::{Category, ProjectIndex};

let mut index = ProjectIndex::new("project-123")?;
index.replace_document("main.tex", 1, r"\label{sec:intro}")?;
index.replace_document("parts/body.tex", 4, r"See \ref{sec:intro}")?;
let view = index.snapshot();
let definitions = index.definitions(&view, Category::Label, "sec:intro")?;
let completions = index.complete(&view, Category::Label, "sec:", 20)?;
assert_eq!(index.source_text(&view, &definitions[0].source)?, "sec:intro");
```

Each `SourceSpan` contains the exact project-relative file, that document's `u64`
revision, and zero-based UTF-8 byte offsets `[start_byte, end_byte)`. Symbol spans
cover names only: no leading backslash, argument braces, comma separators, or
surrounding whitespace. Slicing the corresponding source yields the exact symbol
name. Filenames may contain Unicode; absolute paths, backslashes, drive notation,
empty components, `.` and `..` components, and NUL are rejected without mutation.
The caller defines the project root; the library performs no filesystem canonicalization.

Every replacement must increase that file's revision; initial revision zero is
allowed. Replacing one document drops its old symbols and diagnostics atomically
without requiring unrelated documents to use the same revision. Deletion also needs
a newer revision and retains a tombstone so delayed replacements cannot resurrect
old content. A failed update does not advance generation or discard existing state.

Capture a `VersionSnapshot` after the last accepted update and pass it to every
query. The index requires the same project ID, generation and complete document
revision map. Any intervening replacement/addition/removal rejects an older snapshot
with `StaleSnapshot`; another project ID returns `WrongProject`. Even limit-zero
completion queries check the snapshot. IDs and revision numbers are caller-managed
identities, not cryptographic credentials or proof of source authenticity.

Before applying a retained selection, call `source_text` with its `SourceSpan` and
the current snapshot. This rejects the range if its document revision changed,
even if the caller supplies a fresh snapshot. Bounds, ordering and UTF-8 boundaries
are also checked. `navigate` accepts a byte position inside a name's half-open span;
out-of-range and non-character-boundary positions fail, while ordinary text or
end-of-document positions return no navigation target.

Public queries are `symbols`, `diagnostics`, `occurrences`, `definitions`,
`navigate`, `source_text`, and `complete`. Results are owned values ordered by
project-relative filename then source byte position. `navigate` retains the origin
and every matching lexical definition, including duplicate definitions across files.
It never guesses a TeX scope winner. Missing targets remain empty rather than
invented. Completion is case-sensitive, sorted, deduplicated and limit-bounded;
it includes observed unresolved names, with separate definition and occurrence
locations so consumers can distinguish unresolved suggestions. Command completion
prefixes omit the leading backslash.

Indexed constructs:

| Category | Definitions | References / uses |
| --- | --- | --- |
| Labels | `\label{key}` | `\ref`, `\eqref`, `\pageref`, `\autoref`, `\cref`, `\Cref`, `\nameref` |
| Citations | `\bibitem[display]{key}` | `\cite`, `\citep`, `\citet`, `\citeauthor`, `\citeyear`, `\parencite`, `\textcite`, `\autocite`, `\nocite` |
| Commands | `\newcommand`, `\renewcommand`, `\providecommand`, `\DeclareRobustCommand`, `\def`, `\gdef`, `\edef`, `\xdef` | Lexical control words/symbols elsewhere |

Citation key lists and `cref`/`Cref` lists split into individual trimmed names.
Optional bracket arguments and starred forms are skipped structurally; a bibliography
wildcard `\nocite{*}` does not become a citation key. Command declarations accept
braced or bare command targets. Their target tokens are definitions, not duplicated
as uses. Command names use ASCII letters/`@`, or one following Unicode control
symbol; active character/catcode execution is outside this lexical index.

Unescaped `%` comments, `\verb`/`\verb*` bodies and `verbatim`/`verbatim*`
environments are skipped. Escaped `%` and escaped backslashes do not create phantom
commands. Malformed literal arguments produce source-located diagnostics and the
scanner resumes; an unclosed `\verb` resumes after its line. Empty/dynamic names
containing braces, backslashes or comments are not expanded into targets.

Limitations remain explicit: no BibTeX/Biber database parsing, custom citation or
reference macro recognition, conditional execution, macro expansion, runtime scoping,
include graph execution, `lstlisting`/`minted` catcodes, or proof that a document
compiles. Definitions inside macro bodies are lexical source occurrences, not a
claim they execute. Whole-document replacement is supported; this is not a measured
incremental parser or a latency guarantee. Malformed inputs are handled conservatively,
and highly nested/unclosed groups may require rescanning. Native consumers must
adapt the Rust API and preserve the snapshot/source-range checks.

Checkpoint: 19 isolated tests cover Unicode cross-file navigation, comments and
verbatim exclusions, declarations/citations, malformed recovery, duplicate targets,
sorted completion, atomic revision replacement, deletion tombstones, stale queries,
retained-range revalidation, and 200 deterministic malformed Unicode source samples.
This validates the index behavior only, not compiler compatibility or native UI.

## Reviewable label rename plans

`plan_label_rename(snapshot, old_name, new_name)` returns all lexical label
definitions and references across the project as a `RenamePlan`. It does not
change source or index state. `plan_label_rename_at` additionally requires a retained
`SourceSpan` identifying an entire current label name, with exact revision/range
checks. Citation keys, comments and verbatim bodies remain outside label edits.

Plans contain the captured project snapshot, both names, and `TextEdit` entries
with file/revision, UTF-8 byte range, expected old text and replacement. Edits have
deterministic file/start-byte order and do not overlap. A caller can preview them
against copied sources, applying each file's edits in descending byte order to
avoid offset drift when Unicode replacement lengths differ.

Before a caller applies a reviewed plan, `validate_rename_plan` rechecks the snapshot
and regenerates the entire exact edit set. Omitted, duplicated, reordered, shifted
or changed entries fail. Any intervening project update invalidates the old plan.
The caller owns its native editing transaction and must preserve those guards
through application; this library never applies edits automatically.

A source definition is required. Existing definitions or unresolved references
using the new name block rename to avoid accidental rebinding. Empty names,
whitespace/control characters, commas, braces, backslashes and percent signs are
rejected. Renaming to the same valid name produces an empty plan. Duplicate lexical
definitions are all included, without inferring a scope winner. Macro expansion,
runtime scoping and indirect labels remain outside this lexical operation.

Rename checkpoint: all 26 tests pass, including seven plan/source-guard tests;
strict Clippy and formatting pass. No automatic document mutation was introduced.
