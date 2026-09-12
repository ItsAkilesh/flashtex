# flashtex-compiler

Original Rust compiler foundation for FlashTeX (task FT-002). No existing TeX
engine is invoked, linked, or shelled out to. The compiler has no registry
dependencies: it uses the in-repository `../font-engine` and
`../paragraph-layout` crates through path dependencies, so the build remains
offline and deterministic. The JSON transport is hand-written.

Speaks runtime protocol v1 (`docs/contracts/runtime-v1.md`) over JSON Lines on
stdin/stdout.

## Negotiated layout capabilities

A compile request may add `payload.layout_capabilities`. It must be a
duplicate-free list of at most 16 non-empty strings, each at most 64 UTF-8
bytes. Unknown names are ignored for acceptance; malformed fields are rejected
with an explicit diagnostic. A `compile_result` echoes only supported names
that were requested, in request order. When the request field is omitted, the
response field is also omitted and the base runtime-v1 output is unchanged.
Negotiation is per request, and warm incremental sessions are isolated by the
accepted capability set.

This revision supports:

- `rules-v1`: fractions use an opaque black `rule` item with top-left `x_pt`
  and `y_pt`, positive `width_pt` and `height_pt`, and the source range of the
  generating `\frac` command. Without this capability, the legacy U+2500 text
  approximation remains and produces the existing PDF-export warning.
- `font-hints-v1`: every text item adds a `font` object containing the family,
  `normal` or `bold` weight, and `normal` or `italic` style selected by layout.
  Current body text reports Times-Roman, headings report Times-Bold, and
  supported mathematical symbols report Symbol.

This additive extension is not rendering-v2 activation or a claim of exact
LaTeX PDF identity. Font hints do not identify font bytes, glyph IDs, shaping,
encoding, or exact advances.

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
- Project-relative `\input` expansion across supplied documents, with included
  text and diagnostics retaining the included document's path and byte ranges.
- Dependency-aware incremental layout reuse behind unchanged runtime-v1 messages.
  A resumable cursor in `src/layout.rs` is the only layout engine used by both
  clean and incremental builds. Per-block cache validation includes exact macro
  definitions read, preamble bytes, layout constraints, source mapping, and the
  flow geometry entering the block; `ReuseStats` reports actual reuse.
- Diagnostics carrying severity, message, source range, and a recovery note.
- The shared original Rust font engine is the single measurement path. Core 14
  shaping supplies exact AFM advances and pair kerning; its standard ligatures
  are enabled. Every shaping cluster retains the exact input byte range and text.
  Literal cluster-relative ranges are translated back into the originating
  document, while generated macro text keeps its real invocation span.
- TeX-style total-fit paragraph breaking from `flashtex-paragraph-layout`, fed
  by shaped runs from `flashtex-font-engine`. Explicit `\-` discretionaries
  can break within a word; automatic pattern hyphenation is not shipped by the
  adopted crate and remains unsupported here.
- Page breaking onto 612×792 pt pages.
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
- No bidi, joining, complex-script reordering, or automatic pattern
  hyphenation. The font engine reports unsupported shaping and missing glyphs
  explicitly; the compiler never silently substitutes a missing glyph.
- `\textbf`, `\emph`, and `\textit` are parsed and their text is typeset, but the
  visual weight and slant are not yet applied.

## Supported commands

`\documentclass[options]{class}`, `\usepackage[options]{a,b,c}`,
`\newcommand{\name}{body}`, `\newcommand{\name}[n]{body}`,
`\renewcommand{\name}{body}`, `\renewcommand{\name}[n]{body}`,
`\section{...}`, `\subsection{...}`, `\label{key}`, `\ref{key}`,
`\pageref{key}`, `\caption{...}`, `\textbf`, `\emph`, `\textit`,
`\begin`/`\end` for `document`, `equation`, `figure`, `itemize`, and
`enumerate`, `\item`, `\par`, `\-`, and `\\`. Macro
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
The corresponding Unicode glyph must exist in the Symbol face selected by the
export mapping; ordinary math letters and digits use Times-Roman. Unknown math
commands produce an explicit diagnostic naming the command and are rendered
literally, never silently dropped.

Script sizes and shifts and fraction geometry use named classic-proportion
constants in `src/math.rs`. They approximate TeX's font-parameter-driven values;
the compiler does not yet read a real math font.

## Font shaping and layout limits

Layout measures Times-Roman body text, Times-Bold headings, and supported math
symbols in Symbol through `flashtex-font-engine::shape`. The returned cluster
advances already include AFM pair kerning and enabled standard ligatures. Those
shaped clusters become `paragraph-layout` boxes directly; glue and penalties
remain separate, and `layout_paragraph` positions the resulting runs.

The adapter assigns each shaped box a unique paragraph-local range and keeps a
side table back to the exact document `Span` and output kind. The local range is
never exposed as a document offset. Positioned literal runs map back to exact
UTF-8 source slices, including each half of a discretionary word. A generated
discretionary hyphen maps to the complete word construct that produced it.
Macro replacements continue to map to their invocation construct; no generated
per-character source offsets are fabricated.

This is real Core 14 shaping and TeX-style total-fit breaking, but it is not full
TeX paragraph layout. Automatic pattern hyphenation, approximate math constants,
and the lack of a negotiated original-glyph rendering contract still prevent
pixel or PDF identity claims. A changed paragraph is broken again as a whole;
unchanged blocks can still be reused, and incremental output remains required to
be byte-identical to a clean full build.

## Pinned evidence and separate fidelity gates

`tests/pinned_fixtures.rs` compiles plain text, a heading, inline/display math,
a macro, an include, a forward reference, and `AV Wa To Ty`. It compares the
complete live `compile_result` JSON against committed JSONL bytes in
`tests/pinned/`, once with no capabilities and once with `rules-v1` plus
`font-hints-v1`. It does not parse, round, reorder, or normalize either stream;
on failure it lists every differing byte offset with context. The regeneration
command is pinned in the test header and is ignored during ordinary test runs.

This compiler evidence is distinct from three other gates: raw PDF-byte equality,
raster pixel equality, and incremental-versus-clean equivalence. The first two
require downstream PDF/native artifacts and are not asserted in this crate;
incremental-clean equivalence remains covered by the compiler's session tests.
No reference TeX engine is invoked in production.

## Negotiated representation and source attribution

**Fraction rules retain a legacy route.** A client that negotiates `rules-v1`
receives a typed rectangle and no box-drawing fraction glyph. An old client, or
one that does not request the capability, still receives the original text item
and its explicit export-approximation warning. Typed rule paint order is its
position in the page's item list.

**Substituted glyphs span their source command.** `\alpha` emits an item whose
text is the Greek letter but whose span covers `\alpha` in the source, six bytes.
Generated section, equation, figure, and list numbers follow the same rule: their
spans cover the `\section`, display delimiter/`\begin`, `\caption`, or `\item`
command that produced them. For these items the span does not slice back to the
item's text, unlike ordinary words. That is deliberate: source navigation must
land on the command the author typed. The ordinary-text invariant — every
ordinary word item's span slices back to exactly that word — is unchanged.

## Scaling, measured

Run `cargo run --release --bin scaling_bench`. It deterministically generates
the three inputs from a fixed seed and takes 30 samples per case (five samples
for a fresh cold session). The post-adoption run used binary SHA-256 prefix
`396d6d06005b0377`.

| Input | Bytes / blocks | Input SHA-256 |
|---|---:|---|
| 5 KB | 5,019 / 84 | `343a997f84568daf` (prefix) |
| 50 KB | 50,124 / 792 | `5e98259d014082a1` (prefix) |
| 500 KB | 500,056 / 7,754 | `925e62ec2e45b868` (prefix) |

| Input | Case | p50 | p95 | p99 |
|---|---|---:|---:|---:|
| 5 KB | Cold fresh session | 1.901 ms | 2.857 ms | 2.857 ms |
| 5 KB | Warm unchanged | 0.054 ms | 0.064 ms | 0.091 ms |
| 5 KB | One-word edit | 0.393 ms | 0.468 ms | 0.502 ms |
| 5 KB | Global macro edit | 1.042 ms | 1.339 ms | 1.339 ms |
| 50 KB | Cold fresh session | 7.872 ms | 8.554 ms | 8.554 ms |
| 50 KB | Warm unchanged | 0.276 ms | 0.313 ms | 0.390 ms |
| 50 KB | One-word edit | 2.219 ms | 2.425 ms | 2.461 ms |
| 50 KB | Global macro edit | 8.115 ms | 8.802 ms | 8.802 ms |
| 500 KB | Cold fresh session | 84.699 ms | 89.221 ms | 89.221 ms |
| 500 KB | Warm unchanged | 3.122 ms | 3.411 ms | 4.801 ms |
| 500 KB | One-word edit | 26.833 ms | 28.829 ms | 30.864 ms |
| 500 KB | Global macro edit | 90.118 ms | 91.543 ms | 91.543 ms |

The benchmark checks exact incremental-versus-clean output outside every timed
interval. At 500 KB the ordinary edit reused 7,753 of 7,754 blocks; the global
macro edit correctly reused none. No measured case crossed 200 ms. Relative to
the immediately preceding run on the same inputs, 500 KB cold p95 changed from
92.341 ms to 89.221 ms, one-word-edit p95 from 29.125 ms to 28.829 ms, and
global-macro-edit p95 from 84.732 ms to 91.543 ms. These are observed timings,
not a claim that total-fit is intrinsically faster; short runs remain noisy and
the algorithm does more work per changed paragraph than the former greedy loop.

Revision 7's earlier `incremental_bench` found two superlinear costs. Before the
fixes, on that benchmark's inputs,
the 500 KB one-word edit was 530.636/549.694/563.445 ms p50/p95/p99 and cold was
569.313/579.248/579.635 ms. The old reuse loop made more than 3.15 million
candidate comparisons for the one-word edit and 6,300,100 for the macro edit.
The span-signature index reduced both to 2,510 hash-bucket confirmations, each
still gated by full dependency and shifted-block equality. The parser also used
two tail-moving vector operations per macro invocation; one range replacement
preserves argument and recursion semantics while avoiding the second move. The
resulting 500 KB one-word-edit p95 is 32.797 ms, 16.8 times faster.

The other suspects remain linear and were left alone. Reused positioned items
must each receive a current-revision source span, so span shifting and page
reconstruction are one pass over reused items. Label/reference convergence is a
fixed maximum of five whole-document passes, not a block-squared loop. Diagnostic
append is linear in diagnostic count, and the protocol export check is one pass
over emitted items and characters. A separate 500 KB probe measured parse at
12.937 ms p50 / 14.274 ms p95 and the complete one-word edit at 31.581 ms p50 /
36.382 ms p95; no additional superlinear curve was observed in those linear
walks.

These are COMPILER-WORK-ONLY measurements, from source text to laid-out result.
UI paint, scheduling, IPC transport, PDF writing, and native-shell work are not
included. Native paint equality and raw PDF byte equality remain separate gates
and are not claimed here. The product target of under 200 ms from keystroke to
visible output REMAINS UNPROVEN and can only be established in the real app.

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
