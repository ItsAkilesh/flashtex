# mac-bibliography handoff

Agent / task / branch: `mac-bibliography` (Claude Code subagent, parent
`mac-claude-a`, machine `mac-m1max-a`) / issue #2 "BibTeX data layer" (no FT
number, no `coordination/assignments/*.json` yet, so no `ack`) /
`agent/mac-bibliography/bibtex`

State: ready for integration

Owned paths: `crates/bibliography/**`, `coordination/mac-bibliography.md`,
`coordination/agents/mac-bibliography.json`. Compiler files are read-only.

Main integrated through: 1befb923fb8c248a704a1ff5148d77b5f581130f

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

Incomplete behavior: none of the issue #2 scope is outstanding. The compiler
adapter itself is not implemented (proposal only, `crates/bibliography/
ADAPTER-PROPOSAL.md`, awaiting compiler-owner agreement). Goldens match the
author's expectation of `plain.bst` output and have not been diffed against a
real BibTeX run (no TeX installation is used by this project). See README
"Unsupported": biblatex, crossref beyond one level and crossref-specific
formatting, `@string` across files, styles other than unsrt/plain/alpha.

Interface changes and required consumer actions: none; new crate, no shared
contract touched. Diagnostics reuse the runtime-v1 shape and UTF-8 byte spans.

Validation: `cd crates/bibliography && cargo test` (43 tests: 11 unit, 31
integration, 1 doctest), `cargo clippy --all-targets -- -D warnings` clean,
`cargo fmt --check` clean. rustc 1.99.0-nightly.

Needs from others: an FT number/assignment file for `ack`; compiler owner
agreement before the adapter proposal becomes a contract.

Next action: await Commander review/FT number; on compiler-owner agreement,
implement the adapter per the proposal (compiler owner's paths, not mine).

Peer revisions reviewed and adaptations: origin/main 1dd26c5 (branch base).
`crates/compiler/src/diagnostics.rs` read; same severity/message/span/recovery
shape mirrored in `crates/bibliography/src/diagnostics.rs` with a hand-written
`to_json(path)`.

Resource: allocation `claude-mac20x-bibliography` (Claude Max 20x on
mac-m1max-a, shared account quota); usage unknown, no per-call figures exposed.

Updated: 2026-09-12T06:05Z
