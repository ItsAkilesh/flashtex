# flashtex-compiler

Original Rust compiler foundation for FlashTeX (task FT-002). No existing TeX
engine is invoked, linked, or shelled out to. Zero external crate dependencies:
the build is offline and deterministic, and the JSON transport is hand-written.

Speaks runtime protocol v1 (`docs/contracts/runtime-v1.md`) over JSON Lines on
stdin/stdout.

```sh
cd crates/compiler
cargo test
echo '{"protocol_version":1,"id":"a","type":"compile","payload":{"project_id":"demo","revision":1,"entry_path":"main.tex","documents":[{"path":"main.tex","text":"Hello FlashTeX.\n"}]}}' \
  | cargo run --quiet --bin flashtex-compiler
```

## Status of this milestone

Implemented and tested:

- Tokenizer with exact UTF-8 byte spans. Every emitted span slices back to the
  text it describes, verified on multi-byte input (`héllo — naïve café`).
- A finite parser for the subset listed below, with error recovery.
- Diagnostics carrying severity, message, source range, and a recovery note.
- Greedy line breaking and page breaking onto 612×792 pt pages.
- `compile` → `compile_result`, and `error` envelopes for unknown protocol
  versions, unknown message types, and malformed JSON.
- Rejection of absolute paths and parent traversal in document paths.
- An 8 MiB JSON Lines request limit enforced while reading, without buffering an
  arbitrarily large line; the worker consumes an oversized line and continues.

Required, outstanding — this is a foundation, not a LaTeX implementation:

- No macro expansion, no mutable category codes, no registers, no conditionals.
  None of the TeX programmability described in the master plan §5.1 exists yet.
- No math typesetting. `$` is recognised only to report that it is unsupported.
- No packages, no `\usepackage`, no TikZ, no bibliography, no cross-references.
- No PDF output. `pdf_path` is always `null`, as the contract permits for now.
- No incremental reuse yet. Every request recompiles the whole document; the
  revision number is carried through but nothing is cached across revisions.
- Only the entry document is compiled. Multi-document projects produce a warning
  rather than silently compiling part of the project.
- `\textbf`, `\emph`, and `\textit` are parsed and their text is typeset, but the
  visual weight and slant are not yet applied.

## Supported commands

`\section`, `\subsection`, `\textbf`, `\emph`, `\textit`, `\begin`/`\end`
(only `document` is meaningful; other environments warn and typeset their body
as plain text), `\par`, and `\\`. Paragraphs are separated by blank lines.
`%` begins a comment. Any other command produces an explicit
"not supported by this compiler version" diagnostic — never silent output.

## The glyph-metric placeholder

Layout estimates every glyph as `0.5 × font_size` wide and the inter-word space
as `0.28 × font_size` (`src/layout.rs`). These are **placeholders, not font
metrics.** Line breaks therefore will not match a real TeX engine's, and no
compatibility claim can rest on this output until real per-glyph advance widths
from the font are wired in.

One item is emitted per word rather than per line. That keeps each item's source
span exact, which is what click-to-source navigation (FT-003) needs.

## Recovery behaviour

`status` is `ok` with no diagnostics, `recovered` when diagnostics were produced
but text was still positioned, and `failed` when nothing could be produced.
Recovered cases include unmatched `{`, stray `}`, unterminated environments,
mismatched `\end`, unknown commands, and empty required arguments. Each carries a
`recovery` string stating what was rendered provisionally.
