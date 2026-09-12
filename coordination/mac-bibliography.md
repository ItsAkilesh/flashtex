# mac-bibliography handoff

Agent / task / branch: `mac-bibliography` (Claude Code subagent, parent
`mac-claude-a`, machine `mac-m1max-a`) / issue #2 "BibTeX data layer" (no FT
number, no `coordination/assignments/*.json` yet, so no `ack`) /
`agent/mac-bibliography/bibtex`

State: in progress

Owned paths: `crates/bibliography/**`, `coordination/mac-bibliography.md`,
`coordination/agents/mac-bibliography.json`. Compiler files are read-only.

Main integrated through: d9dd2d2f9731353acb6afd8d653542970aadbd72

Ready behavior (crate `flashtex-bibliography`, edition 2024, zero deps):
- `.bib` parser: `@entry{key, f = {..} | ".." | 123 | macro # ..}`, `@string`
  with `#` concatenation, `@preamble`, `@comment` (balanced group or rest of
  line), `{`/`(` delimiters, nested braces, quotes with embedded braces,
  case-insensitive names, byte-exact spans for entries/types/keys/fields/values
  and errors, skip-to-next-`@` recovery with a recovery note naming the byte.
- Model: `Entry { key, entry_type, fields (ordered, case-insensitive), span }`,
  `Name { first, von, last, jr }`, 14 standard entry types with required/
  optional tables; diagnostics for duplicate keys (error, later dropped),
  unknown types (warning), missing required fields (warning), unresolved
  macros (warning), repeated fields, dangling crossref. One-level crossref.
- LaTeX text: accent/letter decoding to Unicode (documented table), BibTeX
  `purify$`, `change.case$` t/l/u, `text.length$`/`text.prefix$`, `add.period$`.
- Names: `and` split at depth 0, three comma forms, BibTeX von rule incl.
  special characters, `format.name$` templates used by plain/abbrv/alpha.
- Resolution: `resolve(&[Citation{key, span}], &db, Style)` → items, per-cite
  mapping, missing-key warnings on the `\cite` span; `unsrt` (cite order),
  `plain` (presort key), `alpha` (`Knu84`, `GMS94`, `AHU+00`, `a`/`b` suffix).
- Layout data: `format_entry` → blocks of styled runs (Plain/Emphasis) per
  `plain.bst` for all 13 entry functions; decoded text plus raw LaTeX; `.bbl`
  rendering for comparison.

Incomplete behavior: README (in progress), compiler citation adapter proposal
(follow-up 2, after owner agreement). See README "Unsupported" for biblatex,
multi-level crossref, `@string` across files, crossref-specific formatting.

Interface changes and required consumer actions: none; new crate, no shared
contract touched. Diagnostics reuse the runtime-v1 shape and UTF-8 byte spans.

Validation: `cd crates/bibliography && cargo test` (43 tests: 11 unit, 31
integration, 1 doctest), `cargo clippy --all-targets -- -D warnings` clean,
`cargo fmt --check` clean. rustc 1.99.0-nightly.

Needs from others: an FT number/assignment file for `ack`; compiler owner
agreement before the adapter proposal becomes a contract.

Next action: README with accent table, style provenance, unsupported list;
then adapter proposal doc.

Peer revisions reviewed and adaptations: origin/main 1dd26c5 (branch base).
`crates/compiler/src/diagnostics.rs` read; same severity/message/span/recovery
shape mirrored in `crates/bibliography/src/diagnostics.rs` with a hand-written
`to_json(path)`.

Resource: allocation `claude-mac20x-bibliography` (Claude Max 20x on
mac-m1max-a, shared account quota); usage unknown, no per-call figures exposed.

Updated: 2026-09-12T05:50Z
