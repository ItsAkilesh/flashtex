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

- Tokenizer with exact UTF-8 byte spans. Ordinary text and literal math items
  slice back to their emitted text, verified on multi-byte input
  (`héllo — naïve café`). A substituted math glyph such as `α` instead maps to
  the command source (`\alpha`) that produced it; the span remains exact and
  slice-safe but the source slice intentionally differs from the output glyph.
- A finite parser for the subset listed below, with error recovery.
- LaTeX preamble recognition: `\documentclass[options]{class}` records the
  class, and `\usepackage[options]{a,b,c}` records the package names and emits
  one warning listing exactly those unimplemented packages. When a document
  environment exists, only its body is typeset; bare fragments retain the
  previous typeset-everything behavior.
- Scoped `\newcommand` and `\renewcommand` expansion, with zero through nine
  required arguments, nested expansion, and an explicit recursion limit.
- Dependency-aware incremental layout reuse behind unchanged runtime-v1 messages.
  A resumable cursor in `src/layout.rs` is the only layout engine used by both
  clean and incremental builds. Per-block cache validation includes exact macro
  definitions read, preamble bytes, layout constraints, source mapping, and the
  flow geometry entering the block; `ReuseStats` reports actual reuse.
- Diagnostics carrying severity, message, source range, and a recovery note.
- Greedy line breaking and page breaking onto 612×792 pt pages.
- Inline math (`$...$`) and display math (`$$...$$` and `\[...\]`), including
  nested fractions, square roots, superscripts, and subscripts.
- Numbered `\section{...}` and `\subsection{...}` headings, numbered display
  equations (`$$...$$`, `\[...\]`, and `\begin{equation}...\end{equation}`),
  and LaTeX-style subsection reset when a section advances.
- `\label{key}`, `\ref{key}`, and `\pageref{key}` with forward-reference and
  page-number convergence (at most five layout passes). Undefined references
  render `??`; duplicate labels warn and the later definition wins.
- `\begin{figure}...\caption{...}\label{key}...\end{figure}` with centred,
  numbered `Figure N: ...` captions. Figure bodies may contain supported text
  and math, but this milestone does not load images or place floating objects.
- `\begin{itemize}...\item...\end{itemize}` and
  `\begin{enumerate}...\item...\end{enumerate}` with bullet and decimal markers.
- `compile` → `compile_result`, and `error` envelopes for unknown protocol
  versions, unknown message types, and malformed JSON.
- Rejection of absolute paths and parent traversal in document paths.
- An 8 MiB JSON Lines request limit enforced while reading, without buffering an
  arbitrarily large line; the worker consumes an oversized line and continues.

Required, outstanding — this is a foundation, not a LaTeX implementation:

- No `\def`, `\let`, mutable category codes, registers, or conditionals.
- No `\input` or multi-file include expansion.
- Math remains a declared subset: matrices, alignment environments,
  `\left`/`\right` delimiter sizing, real math-font parameters, and operator
  spacing classes are not implemented.
- Package declarations are recognised but packages are not loaded: package
  commands, TikZ, bibliographies, and `\cite` remain missing.
- Image loading (`\includegraphics`), tables, and float placement remain
  missing. `\includegraphics` emits an explicit unsupported diagnostic; a
  `figure` is laid out in source order and is not a real LaTeX float.
- Environments other than `document`, `equation`, `figure`, `itemize`, and
  `enumerate` warn and typeset as plain text.
- No PDF output. `pdf_path` is always `null`, as the contract permits for now.
- Only the entry document is compiled. Multi-document projects produce a warning
  rather than silently compiling part of the project.
- `\textbf`, `\emph`, and `\textit` are parsed and their text is typeset, but the
  visual weight and slant are not yet applied.

## Supported commands

`\documentclass[options]{class}`, `\usepackage[options]{a,b,c}`,
`\newcommand{\name}{body}`, `\newcommand{\name}[n]{body}`,
`\renewcommand{\name}{body}`, `\renewcommand{\name}[n]{body}`,
`\section{...}`, `\subsection{...}`, `\label{key}`, `\ref{key}`,
`\pageref{key}`, `\caption{...}`, `\textbf`, `\emph`, `\textit`,
`\begin`/`\end` for `document`, `equation`, `figure`, `itemize`, and
`enumerate`, `\item`, `\par`, and `\\`. Macro
argument counts are decimal integers from 0 through 9, and replacement
parameters are `#1` through `#9`. Paragraphs are separated by blank lines.
`%` begins a comment. Any other command produces an explicit "not supported by
this compiler version" diagnostic — never silent output.

## Macro expansion and source mapping

User macros expand at their use site and may call other user macros. Expansion
is limited to 64 nested macro calls. Exceeding that limit emits an error naming
the macro and stops that invocation, so recursive definitions cannot hang.
`\newcommand` rejects an existing name; `\renewcommand` rejects an
undefined name. A definition made inside `{ ... }` is restored or removed when
that group closes.

Tokens substituted for `#1` through `#9` retain the real byte spans of the
argument text the author supplied. Literal replacement tokens have no independent
bytes in the input and therefore map to the macro control-sequence span at the
invocation site.
This is intentionally invocation-level provenance: per-glyph ranges inside
synthesised replacement text are not fabricated.

## Supported math

Math atoms are ordinary characters and digits. `\frac{num}{den}` and `\sqrt{x}`
may be nested, and `^` superscripts and `_` subscripts accept either one token or
a braced math list, including `x^{a_b}` and `\frac{a^2}{b_1}`.

The named symbols `\alpha`, `\beta`, `\gamma`, `\delta`, `\theta`, `\lambda`,
`\mu`, `\pi`, `\sigma`, `\phi`, `\omega`, `\times`, `\div`, `\pm`, `\leq`,
`\geq`, `\neq`, `\approx`, `\cdot`, `\infty`, `\sum`, and `\int` map to Unicode.
The corresponding Unicode glyph must exist in the chosen font. Unknown math
commands produce an explicit diagnostic naming the command and are rendered
literally, never silently dropped.

Script sizes and shifts and fraction geometry use named classic-proportion
constants in `src/math.rs`. They approximate TeX's font-parameter-driven values;
the compiler does not yet read a real math font.

## The glyph-metric placeholder

Layout estimates every glyph as `0.5 × font_size` wide and the inter-word space
as `0.28 × font_size` (`src/layout.rs`). These are **placeholders, not font
metrics.** Line breaks therefore will not match a real TeX engine's, and no
compatibility claim can rest on this output until real per-glyph advance widths
from the font are wired in.

One item is emitted per word rather than per line. That keeps each item's source
span exact, which is what click-to-source navigation (FT-003) needs.

## Two deliberate representation choices

**Fraction rules are drawn as text.** runtime-v1 defines only a `text` item and
says line and path item types "will be added by contract revision; do not
independently invent them". So the fraction bar is emitted as box-drawing
characters in a text item rather than an invented rule item. It is positioned
correctly and it is honest about the contract; it should become a real rule item
when the contract gains one, and the Commander owns that revision.

**Substituted glyphs span their source command.** `\alpha` emits an item whose
text is the Greek letter but whose span covers `\alpha` in the source, six bytes.
Generated section, equation, figure, and list numbers follow the same rule: their
spans cover the `\section`, display delimiter/`\begin`, `\caption`, or `\item`
command that produced them. For these items the span does not slice back to the
item's text, unlike ordinary words. That is deliberate: source navigation must
land on the command the author typed. The ordinary-text invariant — every
ordinary word item's span slices back to exactly that word — is unchanged.

## Recovery behaviour

`status` is `ok` with no diagnostics, `recovered` when diagnostics were produced
but text was still positioned, and `failed` when nothing could be produced.
Recovered cases include unmatched `{`, stray `}`, unterminated environments,
mismatched `\end`, unknown commands, and empty required arguments. Each carries a
`recovery` string stating what was rendered provisionally.

## Incremental safety boundary

The parser executes the complete document on every changed revision so macro and
group state, diagnostics, and recovery are identical to a clean build. Only
positioned block-layout fragments are reused. Changing a macro definition
invalidates every block that actually read that definition; unrelated blocks may
still be reused if their entering flow geometry matches. A changed flow state
(for example, because an earlier edit adds a line) recomputes the affected suffix
until geometry matches again.

A first compile, any preamble-byte change through `\begin{document}` (including
`\documentclass` or `\usepackage`), font-size or measure changes, malformed input,
or any diagnostic/unsupported construct forces a full layout rebuild. Mutable
category codes, registers, assignments, conditionals, auxiliary files, output
routines, external effects, and future constructs are not modeled and therefore
must also force a full rebuild if introduced. An exactly unchanged snapshot may
return its already-produced output, including diagnostics, because no execution
or layout result can differ.

Counters are embedded in parsed counter-bearing blocks, so changed incoming
counter values invalidate those blocks and geometry invalidates affected suffixes.
Labels and references are more global: any changed snapshot containing either is
laid out conservatively from scratch and passed through the bounded convergence
loop. This intentionally sacrifices reuse to keep every incremental result
byte-identical to a clean build.

## Measured incremental latency

Measured on `mac-m5pro-kabir` on 2026-09-12 with:

```sh
cargo run --release --bin incremental_bench
```

The deterministic `generated-500-paragraphs` fixture is built by the benchmark:
500 multi-line paragraphs, 118,700 UTF-8 bytes, with one user-macro expansion,
inline scripted math, a named math symbol, and a fraction in every paragraph.
Fifty samples produced these actual compiler-only measurements:

| Case | Actual latency | Reuse |
|---|---:|---:|
| First cold compile | 35.522 ms | 0 / 500 blocks |
| Cold compile | median 20.034 ms, p95 21.518 ms | 0 / 500 blocks |
| Warm unchanged | median 0.639 ms, p95 0.710 ms | 500 / 500 blocks |
| One-word edit in paragraph 250 | median 21.079 ms, p95 22.578 ms | 499 / 500 blocks |
| Global macro-definition edit | median 21.219 ms, p95 22.682 ms | 0 / 500 blocks |

The measured compiler work is below the 200 ms ordinary warm-edit target; the
one-word edit p95 is 22.578 ms, leaving 177.422 ms of that budget. This is not an
end-to-end keystroke-to-visible measurement: scheduling, JSON transfer, native UI
drawing, and artifact publication are excluded, so the full product target still
requires integration measurement. The benchmark intentionally does not claim a
guarantee for arbitrary documents or TeX programs.
