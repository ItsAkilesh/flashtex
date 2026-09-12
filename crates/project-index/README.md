# Project source index

`search_literal(snapshot, &SearchRequest, cancelled)` searches raw document text,
including comments and verbatim, independently of lexical symbol/metadata parsing.
Matching is case-sensitive, nonoverlapping, with no Unicode normalization or regex.
Results use exact UTF8 byte spans in deterministic project-relative file/offset
order. `documents: None` selects all documents; `Some(empty)` selects none; all
selected paths must exist. Empty queries and queries over 64 KiB are rejected.

Search uses an original KMP implementation. `max_work` counts byte comparisons in
both preprocessing and matching. `max_matches` is capped at 100,000. Results state
`Complete`, `MatchLimit`, `WorkLimit`, or `Cancelled`; an incomplete result never
asserts absence of later matches. A reached match limit conservatively reports
`MatchLimit` unless the final document ended at that match. Cancellation callbacks
are checked initially and before every comparison and must not block. Callback
runtime, snapshot/path validation, allocation, and document selection are outside
the comparison budget; this is not a wall-clock deadline guarantee.

`plan_literal_replacement(&complete_search, replacement)` creates reviewable
multi-document edits only after regenerating the exact complete match set against
its full project snapshot. Partial/cancelled, stale, omitted, reordered, or altered
results fail. `validate_literal_replacement_plan(&plan)` regenerates that plan and
checks every expected-text, revision, range, replacement and edit ordering guard.
Aggregate expected/replacement text is capped at 8 MiB. Empty replacement deletes
matches; empty match sets produce empty plans. The caller must explicitly approve
and transactionally apply a validated plan, in reverse byte order within each file,
against the same revisions. This library never applies edits. Validation establishes
consistency, not user approval or authentication of the caller's intended query.

`serialize_literal_replacement_plan(&plan, max_bytes)` validates the plan and
returns standalone UTF8 JSON conforming to `replacement-plan.schema.json`.
Output never exceeds the caller's byte limit or the 32 MiB hard ceiling; failure
returns `SerializationLimit` with no partial output. Decimal strings encode all
revisions, generation, offsets, and work/match counts exactly; native consumers
must parse them with checked integer conversion. Offsets are half-open UTF8 byte
positions, not UTF16 indices. Snapshot documents, selected paths, and edits have
deterministic order. The full project snapshot is included even for selected-file
searches. JSON escapes control characters, quotes, and backslashes, preserving
other Unicode characters without normalization.

The schema describes wire structure only. A consumer must independently enforce
integer ranges, byte limits, project identity, the complete snapshot, exact expected
UTF8 slices, nonoverlap, consistent replacements, and user approval before applying.
Schema conformance alone proves neither source consistency nor user authorization.
There is no deserialization/application endpoint in this crate. For a standalone
native-consumable example, run:

```sh
cargo run --offline --manifest-path crates/project-index/Cargo.toml --example replacement_wire
```

The example prints a deterministic proposal with Unicode text, escaped controls,
and a maximum u64 document revision; it does not write or modify documents.

`citation_metadata(snapshot, key)` inspects bounded bibliography values locally.
Records retain entry/key, field-name/expression, and atom UTF8 source spans.
Braced and quoted literals preserve internal braces and TeX text; decimal atoms
and `#` concatenation are supported. Bare identifiers resolve only against unique,
ASCII-case-insensitive project `@string` declarations. There are no implicit month
macros, declaration-order semantics, TeX expansion, or bibliography formatting.
This is an IDE inspection convention, not BibTeX engine output.

Missing keys, bibitems without metadata, duplicate definitions, malformed records,
and incomplete values have separate states. Duplicate fields invalidate the record;
missing, duplicate, malformed, cyclic, or over-limit macros produce no successful
partial value. Field failures conservatively exclude the rest of that record;
the outer scanner resumes only at a safely delimited following entry. Limits are
128 fields, 256 atoms per expression, 128 nested groups, 32 macro levels, 4096
expansion atom visits per field and 256 KiB of expanded bytes per field, in
addition to the existing document/entry limits. All queries require a fresh snapshot.

Parsed records are retained per document. `complete_citations(snapshot, prefix,
limit)` returns sorted cached metadata for definitions and observed unresolved
citations. `metadata.field("author")`, `field("title")`, and `field("year")` expose
literal inspection values and exact expression spans usable with `source_text`.
This preserves source text, including name separators and TeX syntax; it does not
split authors, normalize years, or infer formatted titles.

Replacement/removal refreshes citation keys in the changed document and cached
keys whose visited direct/transitive macro dependencies intersect declarations in
that document. Missing macro dependencies are retained for later repair. Other
metadata results are reused; duplicate counts and field edits refresh even when
definition availability stays unchanged. `metadata_cache_metrics(snapshot)` reports
recomputed/reused/removed key counts. These are work counters, not an asymptotic
performance claim: dependency selection scans cached metadata and resolving each
dirty key inspects retained project records without rescanning source. Macro
evaluation stops at its first error; dependencies beyond that error are discovered
when an earlier dependency is repaired. Fresh rebuild equivalence is tested.

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
| Citations | `\bibitem[display]{key}` and explicitly declared bibliography entry keys | `\cite`, `\citep`, `\citet`, `\citeauthor`, `\citeyear`, `\parencite`, `\textcite`, `\autocite`, `\nocite` |
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

Limitations remain explicit: no BibTeX/Biber evaluation or formatting, custom citation or
reference macro recognition, conditional execution, macro expansion, runtime scoping,
include graph execution, `lstlisting`/`minted` catcodes, or proof that a document
compiles. Definitions inside macro bodies are lexical source occurrences, not a
claim they execute. Incremental reuse occurs between documents; each changed
document is scanned in full. Measurements below concern lexical indexing only,
not compiler or editor latency. Malformed inputs are handled conservatively,
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

## Dependency diagnostics and measured document reuse

`unresolved_references(snapshot)` returns label/citation names that have no lexical
definition anywhere in the indexed project, each with its exact source span.
This query is separate from syntax `diagnostics`, preserving the distinction
between a malformed argument and a missing lexical target. Command uses are not
reported as unresolved because builtin definitions, packages and macro scopes are
not interpreted. Only explicitly declared bibliography documents contribute scanned
keys; unresolved citations are lexical findings, not a claim that the actual
TeX/Biber workflow fails.

The index maintains definition and reader maps keyed by `(Category, name)`, with
sets of contributing documents. Updating a document replaces only its lexical
index. Its own reference diagnostics are refreshed for new offsets/revision; other
documents' diagnostics are rechecked only when availability of a definition they
read changes. A second definition disappearing does not invalidate readers while
another definition survives. Deletion removes cached diagnostics and updates the
same dependency maps. Label and citation namespaces remain independent.

Every successful replacement returns `UpdateSummary.metrics: ReindexMetrics`.
`last_reindex_metrics(snapshot)` also exposes metrics for replacement or deletion,
with stale-query checks. Counters record the input bytes reindexed, number of
document indexes rescanned/reused, changes to definition availability, and reference
documents rechecked. Input byte counts are not parser instruction counts; malformed
groups can require rescanning. Nanosecond lexical and total update timings use the
local monotonic clock and are actual local samples, not deterministic outputs or
provider costs. Failed updates leave the prior metrics and index unchanged.

Run the bounded release-build example:

```sh
cargo run --release --offline --manifest-path crates/project-index/Cargo.toml \
  --example reindex_measure
```

The example fixes its workload to 24 source documents, 80 lines each, and 12 edits
to one document's label definition. It compares complete symbols and unresolved
references with a clean rebuilt index after every edit, retaining actual cold,
edit and clean-build timings plus reuse/diagnostic counters in JSON on stdout.
It starts no provider, compiler, native app or background loop. This is a small
synthetic index benchmark; its latency does not establish full-document typesetting
performance or any end-to-end editor guarantee. Every sample is from a single
local run and should be regenerated on the target hardware.

[Retained sample](evidence/reindex-sample.json) binds the library/example source
hashes, compiler version and exact command to the measured result: all 12 comparisons
equal, 276 document indexes reused over 12 edits, and 144 dependent-document
diagnostic refreshes. The median edit sample was 85,370 ns on this host. This is
evidence for this small lexical workload only, with the limitations above.
The hashes identify the earlier measurement checkpoint; later lexical/bibliography
extensions do not retrospectively change those timing samples.

Dependency checkpoint: 32 total tests pass, including six dependency/cache tests
and 16 successive clean-rebuild equivalence checks inside the test suite. The
bounded measurement additionally checks 12 whole-index equivalences. Strict Clippy
and formatting pass. Rename remains a plan-only operation.

## Bounded lexical exclusions

The literal-environment allowlist now includes `verbatim`/`verbatim*`,
`Verbatim`/`Verbatim*`, `BVerbatim`, `LVerbatim`, `lstlisting`, `minted`, `comment`,
and `filecontents`/`filecontents*`. Inline `verb`, `lstinline` and `mintinline`
forms are skipped, including optional options, language arguments and braced
minted bodies. Percent signs inside literal bodies stay literal. These are source
exclusions only: packages, shell escape, output files and catcode execution are
never invoked or simulated.

Literal environment ends require the exact `\end{NAME}` marker at an unescaped
command-token boundary. Escaped backslashes cannot start a begin/end token; normal
command and percent-comment recognition follows odd/even backslash parity. The
allowlist is case-sensitive and does not infer custom environment aliases. These
conservative conventions are not a full reproduction of each package's parser.
Missing environment ends exclude the remaining source and emit a diagnostic;
malformed inline forms resume after the line. Code inside excluded bodies never
becomes a label/citation target or a rename edit.

Each source document is limited to 8 MiB and a rejected oversize replacement leaves
all prior state intact. Individual balanced-argument scans allow at most 64 KiB and
128 levels; over-limit symbol arguments use the same diagnosed recovery as malformed
ones. Literal-environment searching is a single forward scan with no recursive
expansion. Seven additional tests cover exclusions, escaped delimiters, malformed
recovery and bounds: 39 total tests and strict lint pass at this checkpoint.

## Declared bibliography key sources

Call `replace_bibliography_document(file, revision, text)` to explicitly declare
source containing bibliography entries. `replace_document` continues to mean LaTeX,
even for a `.bib` filename. `document_kind(snapshot, file)` reports that declaration
and rejects stale snapshots. A newer revision may switch a document's kind; the
old kind's symbols and dependency memberships are removed atomically. The library
still receives strings only and never opens a bibliography file itself.

The original bounded header scanner recognizes `@TYPE{key, ...}` and
`@TYPE(key, ...)`, with case-insensitive ASCII-letter entry types. Literal Unicode
keys produce `CitationDefinition` symbols with exact name-only UTF-8 ranges.
`comment`, `preamble`, and `string` records do not define citation keys. Nested
braced fields, quoted fields and escaped delimiters shield entry-looking text;
LaTeX commands in field values never become label definitions or rename targets.
Outside records, percent comments and escaped `@` markers are skipped.

`navigate`, `definitions`, `occurrences`, and `complete` use bibliography keys
alongside `bibitem` definitions. An unresolved cite can therefore acquire a target
when its bibliography document is added. Removing/changing a key invalidates only
the dependent citation diagnostics, using the existing document maps. Duplicate
keys retain every lexical target; no bibliography precedence is invented. Retained
key ranges and all queries keep the same revision/snapshot guards as LaTeX source.

A recognizable header key may remain indexed when its entry body is malformed,
with a source-located diagnostic. Recovery at a following line's entry header is
allowed only outside nested or quoted values. An unclosed value excludes the
remaining source rather than exposing apparent entries inside it. Keys are bounded
to 4096 bytes, entry-body scans to 1 MiB and nesting to 128 levels; an over-limit
body excludes the tail with a diagnostic. The overall 8 MiB document limit still
applies. These conservative lexical rules are documented behavior, not full BibTeX
syntax compatibility, field validation, crossref inheritance, string expansion,
bibliography generation, package execution or native UI integration.

Bibliography checkpoint: nine additional tests cover Unicode navigation, explicit
document kinds, field/metadata exclusions, duplicate definitions, dependency
invalidation, malformed recovery, bounds and 200 generated malformed Unicode
sources. All 48 tests and strict Clippy pass; no bibliography engine is invoked.
