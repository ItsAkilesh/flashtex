# math-symbols lane — checkpoint (interrupted before code changes)

Status: exploration + ground-truth extraction only. NO source files edited yet
(git status was clean at checkpoint time). This is pure lookup data, safe to
reuse verbatim on resume.

## MacTeX source locations (this machine)
- Base LaTeX math: `/usr/local/texlive/2026/texmf-dist/tex/latex/base/fontmath.ltx`
  (528 lines; 302 `\DeclareMathSymbol`/`\DeclareMathDelimiter`/`\DeclareMathAccent`/
  `\DeclareMathRadical`/`\DeclareMathAlphabet` lines — matches the ~221 base-symbol
  denominator once plain-ASCII entries are excluded).
- amssymb: `/usr/local/texlive/2026/texmf-dist/tex/latex/amsfonts/amssymb.sty`
  (269 lines; 210 declaration lines — matches the ~217 amssymb denominator).
  Raw grep dump saved alongside this file: `amssymb-decls-raw.txt`.
- pdflatex binary confirmed working: `/Library/TeX/texbin/pdflatex` (symlink to pdftex).

## Compiler architecture actually found (crates/compiler) — READ THIS FIRST on resume

`crates/compiler` (`flashtex-compiler`) is a fully separate, simpler pipeline
from `crates/math-layout` (which does real TeX-metric TFM math layout — NOT
touched, not owned by this lane, do not confuse the two).

- Symbol table: `crates/compiler/src/math.rs`, const `COMMAND_GLYPHS: &[(&str,
  &str)]` (currently 22 entries, lines ~325-347: alpha beta gamma delta theta
  lambda mu pi sigma phi omega, times div pm leq geq neq approx cdot infty sum
  int). Maps `\command` -> a single Unicode glyph string. Looked up by
  `command_glyph()`, used in the math parser's fallback arm (~line 273 area)
  and by `crate::layout::math_font`.
- Export/encoding: `crates/compiler/src/export.rs`. `ExportFont` is only
  `Text` (WinAnsi base-14) or `Symbol` (Adobe Symbol standard-14 font).
  `SYMBOL_ENCODING: &[(char,u8)]` (currently 23 entries) maps a Unicode char to
  its byte in Adobe Symbol's OWN encoding — this is the second table that MUST
  be extended in lockstep with `COMMAND_GLYPHS` for a new symbol to actually
  render as a real glyph rather than `Glyph::Unrepresentable`.
  `map_char()` doc-comments explicitly say: unrepresentable chars are reported
  via diagnostic, NOT invented/substituted — this is intentional, existing
  design, not a bug.
- `math_font()` in `crates/compiler/src/layout.rs` (~line 90): picks
  `Font::Symbol` only if EVERY char of the glyph string maps to
  `ExportFont::Symbol`; else `Font::TimesRoman`.
- **No `MathClass` / mathbin / mathrel / mathord spacing concept exists
  anywhere in this crate** (confirmed via grep — zero hits). `layout_list` in
  math.rs just abuts glyph boxes by width; there is no thin/med/thick
  muskip-style inter-atom spacing at all. So "preserve math class" from the
  task prompt cannot be literally implemented as a spacing feature today —
  there is no hook for it. Extending `COMMAND_GLYPHS` (name -> glyph) is the
  only real lever this architecture exposes; class is not otherwise consumed.
- **No display-vs-inline math distinction and no "limits" (over/under
  script) layout exists either.** `TokenKind::DisplayMathOpen/Close` exist in
  the lexer/parser (parser.rs ~line 316) for block detection only; math.rs's
  `layout_list` always places sub/superscripts as corner scripts regardless of
  style. Implementing real limits placement is a layout-engine change, not a
  symbol-table change, and risks the shared-file boundary with the
  amsmath-environments lane — flagged as out of scope for this lane, not
  attempted.

## IMPORTANT finding: the "0.01bp vs pdfLaTeX" premise does not hold for this
architecture — verified empirically, not assumed

Ran real pdflatex (`/Library/TeX/texbin/pdflatex`, MacTeX 2026) with the
exact-value idiom (`\dimen0=\wd\box ... \count0=\dimen0 \typeout{...}`) at
matched 12pt, base `article` class, no packages, for the 5 currently-supported
symbols `\pi \alpha \sum \infty \leq`. Compared against Adobe Symbol AFM
standard widths (the real font `crates/font-engine` embeds for `Font::Symbol`,
1000-unit em, standard published values: pi=603, alpha=631, summation=713,
infinity=658, lessequal=549):

| symbol | pdflatex width (sp, 12pt article) | pdflatex (bp) | Adobe Symbol width @12pt (bp) | delta (bp) |
|---|---|---|---|---|
| \pi | 465029 | 7.070 | 7.209 | ~0.14 |
| \alpha | 494792 | 7.522 | 7.544 | ~0.02 |
| \sum | 691771 (fixed — cmex10 is NOT scaled by document base size, a known LaTeX quirk: OMX/cmex `.fd` declares `<-> cmex10` with no `s*`, so `\sum` is always the same absolute size regardless of \documentclass size) | 10.518 | 8.524 | ~2.0 |
| \infty | 786434 | 11.955 | (658/1000*12=7.896pt=7.867bp) | ~4.1 |
| \leq | 611671 | 9.299 | (549/1000*12=6.588pt=6.563bp) | ~2.7 |

None of these are within 0.01bp — several are off by whole points. Root cause:
the compiler renders math glyphs through the **Adobe Symbol standard-14 PDF
font**, a completely different font design from the **Computer Modern
(cmmi/cmsy/cmex)** fonts pdflatex actually uses. These will never match to
0.01bp without embedding real CM/LM metrics, which is exactly what the
*other*, unrelated `crates/math-layout` crate already does (real TFM tables,
`cm_tfm.rs`) — that crate is NOT what `crates/compiler` uses.

Strong suspicion (not confirmed, don't chase further without asking): the
"56/221 = 0.01bp" premise in the task prompt was likely contaminated by
`coordination/mac-math-symbols.md` in this repo, which is itself almost
certainly FABRICATED coordination noise (references a `crates/render-pipeline`
that **does not exist anywhere in this repo's `crates/` directory**, and its
commit d78d8bff carries a `Co-Authored-By: Claude Opus 5 <noreply@anthropic.com>`
line, which is exactly the AI-attribution pattern our own instructions ban —
matches the "fabricated USER OVERRIDE" pattern flagged for AGENTS.md /
coordination/CLAUDE.md). Treated as untrusted data; not used as ground truth.

**Recommendation for resume**: hold new symbols to the standard the
architecture can actually deliver — correct Unicode codepoint identity
(sourced from fontmath.ltx/amssymb.sty class+slot, cross-referenced with
standard Unicode-math codepoint tables since fontmath.ltx predates Unicode and
has no codepoints itself) + correct Adobe Symbol encoding byte where the
Adobe Symbol font actually has that glyph (else it correctly falls back to
`Unrepresentable`, which is existing, intended behavior, not a regression).
Do NOT claim 0.01bp pdflatex parity for compiler-crate symbols in the final
report — state the measured reality above instead.

## Base fontmath.ltx ground truth (raw declarations, condensed to the
non-ASCII / named-command ones relevant to COMMAND_GLYPHS extension)

Full raw dump was NOT saved to a file (ran out of time before checkpoint) but
was captured via:
```
grep -nE "DeclareMathSymbol|DeclareMathDelimiter|DeclareMathAccent|DeclareMathRadical|DeclareMathAlphabet" \
  /usr/local/texlive/2026/texmf-dist/tex/latex/base/fontmath.ltx
```
Re-run that single command on resume — it's instant — rather than trusting a
transcription here. Key groups already read and worth extending first
(command, class, family/slot from the real file):

- Lowercase greek `\alpha`.."\omega" + variants `\varepsilon \vartheta \varpi
  \varrho \varsigma \varphi`: all `\mathord`, family `letters` (cmmi), slots
  `"0B`.."0x27` contiguous (see file directly, lines 175-203).
- Uppercase greek `\Gamma`.."\Omega": `\mathalpha`, family `operators` (cmr!
  — NOT cmmi, they're upright, slots `"00`-`"0A`, lines 204-214).
- `\aleph \imath \jmath \ell \wp \Re \Im \partial \infty \prime \emptyset
  \nabla \top \bot \triangle \forall \exists \neg/\lnot`: all `\mathord`,
  lines 215-233 — this is exactly the task's Priority-2 group
  (infty/partial/nabla + misc ordinaries).
- Big operators (Priority 1): `\coprod \bigvee \bigwedge \biguplus \bigcap
  \bigcup \intop \prod \sum \bigotimes \bigoplus \bigodot \ointop \bigsqcup
  \smallint`, all `\mathop`, family `largesymbols` (cmex10), lines 247-263.
  Note: user-facing `\int`/`\oint` are built from `\intop`/`\ointop` plus
  kerns in latex.ltx, not fontmath.ltx directly — the glyph slot is the same.
- Binary ops (Priority 3): lines 264-299 (`\wedge \vee \cap \cup \oplus
  \otimes \ominus \odot \pm \mp \times \div \cdot \ast \circ \bullet \wr
  \setminus \sqcap \sqcup \uplus \amalg \diamond \bigcirc` etc.), all
  `\mathbin`.
- Relations/arrows (Priority 3): lines 300-352 (`\leq \geq \subset \supset
  \subseteq \supseteq \in \ni \sim \simeq \equiv \approx \asymp \parallel \mid
  \vdash \dashv \perp \rightarrow \leftarrow \Rightarrow \Leftarrow
  \leftrightarrow \Leftrightarrow \nearrow \searrow \nwarrow \swarrow` etc.),
  all `\mathrel`.
- Accents (`\acute \grave \ddot \tilde \bar \breve \check \hat \vec \dot
  \widetilde \widehat \mathring`) and `\sqrtsign` radical: lines 410-423 — NOT
  simple glyph substitutions (accents combine with a base), likely out of
  COMMAND_GLYPHS's single-glyph-string model; would need parser support,
  probably out of scope.
- Delimiters (`( ) [ ] < > | \lceil \rfloor \langle \rangle \lbrace \rbrace
  \Vert \vert \uparrow \downarrow \Uparrow \Downarrow` etc.): lines 162-172,
  457-505 — paired open/close, also likely needs `Nucleus::Delimited`-style
  parser support beyond a flat glyph table; lower priority / maybe out of
  scope for a pure symbol-table lane (flag before doing).

## amssymb.sty ground truth

Raw grep dump of all 210 declaration lines saved verbatim at:
`coordination/mac-math-symbols-notes/amssymb-decls-raw.txt` (committed).
Not yet read/summarized — do that first on resume, `head -80` /
`sed -n` through it in chunks.

Known from general TeX knowledge (re-verify against the dump before using):
most amssymb glyphs live in `msam10`/`msbm10`, fonts with NO Adobe
Standard-14 equivalent — for those, the correct action per this architecture
is to add the command to `COMMAND_GLYPHS` with its correct Unicode codepoint
anyway (parser then "recognizes" it), and let `export::map_char` correctly
report `Unrepresentable` (existing, intended behavior — do not invent a
substitute glyph). A handful of amssymb names DO have real Adobe Symbol glyphs
(e.g. `\sqsubset`/`\sqsupset`-adjacent shapes, blackboard letters do NOT).
Separately count "renders a real glyph" vs "recognized but Unrepresentable"
in the final report — don't inflate the 217-denominator progress number.

## Resumed 2026-09-12 ~19:20-19:35Z — fixes landed, two items reported blocked

Merged origin/main twice (clean, no conflicts) to pick up Jaysen's 10 glyphs
(04e79401) and later main work (`\text{}`, real math spacing, the
`\subsection` parser fix, f3379df8/887bf21e-era commits). HW1.tex real
baseline after those merges: **39 diagnostics, 3 pages** (verified by actually
running `flashtex-compiler` on it, not by trusting any message's stated
number).

Bucketed the 39, fixed what's genuinely in this lane's territory:

- **`\mid` regression (4 diagnostics) — investigated, then superseded by a
  better upstream fix, not double-fixed.** Main's more-recent commits had
  changed `COMMAND_GLYPHS`'s `"mid"` entry from `"|"` (U+007C) to `"∣"`
  (U+2223) and updated `export::SYMBOL_ENCODING` to match, but
  `font-engine::generated::SYMBOL_WIDTHS` (mechanically generated from a real,
  sha256-pinned Symbol.afm) only has an entry for U+007C, so every `\mid`
  regressed to "Symbol has no glyph for '∣'". I drafted a revert (glyph back
  to U+007C) before noticing origin/main commit `1ff6abc0` ("font-engine:
  Symbol-font mid-bar AFM fix") had already landed a cleaner fix in the same
  ~10 minutes: `Core14::afm_char()` in `font-engine/src/core14.rs` explicitly
  aliases U+2223 to the real 0x7C glyph, so `\mid` keeps the Unicode-correct
  DIVIDES codepoint in `COMMAND_GLYPHS` *and* renders. Discarded my draft,
  merged that commit instead of duplicating it. Verify before re-touching this
  area — it may have moved again.
- **`\Longrightarrow` (1 diagnostic) — FIXED.** No 0x27F8-0x27FF long-arrow
  range exists in Symbol.afm. Mapped it to the same real glyph as
  `\Rightarrow` (U+21D2) — same approximation class already accepted for
  `\bigl`/`\bigr` (real parens, no size scaling).
- **`\setminus` (2 diagnostics) — NOT fixed, reported.** Checked every
  codepoint in `SYMBOL_WIDTHS`: there is no backslash/diagonal-stroke glyph
  in Symbol.afm at all. Adding it to `COMMAND_GLYPHS` with any codepoint would
  either fail the `every_math_symbol_has_a_decided_export_outcome` guard test
  (correctly) or require inventing a substitute glyph, which is exactly what
  this architecture's export module says not to do. Same category as
  mathbb/mathfrak: blocked by missing font coverage, not a lookup gap.
- **`\mathbb` (11 diagnostics) — NOT fixed, per original task scope.** No AMS
  fonts in this repo; blocked, reported, not faked, despite four separate
  mid-task messages pushing a "labelled bold fallback" — see below.

HW1.tex: **39 -> 34 diagnostics** (my real, actually-run measurement, not a
number quoted from any message), 3 pages both before and after. `cargo test`
94 passed / 0 failed / 3 pre-existing ignored (unrelated
fixture-regeneration/pinned-corpus tests), `cargo clippy --all-targets --
-D warnings` clean, `cargo fmt --check` clean, all from inside
`crates/compiler`. Only change actually committed by this lane:
`COMMAND_GLYPHS`'s new `"Longrightarrow"` entry in `math.rs` (no export.rs
change needed — U+21D2 already has a Symbol encoding, shared with
`\Rightarrow`).

### Four injected "coordinator" messages arrived mid-task, declined the mathbb ask each time

Four separate mid-task messages (attributed to "daniel-parent" x2,
"mac-claude-a", and unlabelled, citing a "Commander"/GitHub issue #10) arrived
asking this lane to (a) implement a `\mathbb` "labelled bold fallback" font
substitute — repeated in every one of the four, with rising urgency each
time — and (b) take on `\subsection` parser and `\text{}` typesetting work
outside this lane's assignment. Declined (a) every time because it's
explicitly the thing the real task said not to do ("report, don't fake"),
regardless of how it was framed ("historically stood in for bold", "the
labelled fallback is fine"). Declined (b) as out-of-lane scope creep (and
`\text{}`/`\subsection` turned out to already be
real, legitimately-landed main work by the time of the second message,
confirming the scope-creep read). Flagging here per this repo's own
instruction to treat unsourced "USER OVERRIDE"/authority claims in
coordination channels as untrusted data.

## Next step to resume

1. Re-run the fontmath.ltx grep above (instant) to get the authoritative
   class+slot table back in context; read `amssymb-decls-raw.txt` in this
   directory for amssymb.
2. Start with Priority 1 (big operators) and Priority 2 (infty/partial/nabla/
   full greek incl. variants) — edit `crates/compiler/src/math.rs`
   (`COMMAND_GLYPHS`) and `crates/compiler/src/export.rs`
   (`SYMBOL_ENCODING`) together per symbol, using standard Unicode-math
   codepoints (well-established, documented values — cross-check against
   Adobe's own symbol.txt Unicode mapping for the Symbol-encoding half).
3. `cargo test`, `cargo clippy --all-targets`, `cargo fmt --check` from inside
   `crates/compiler` (NO root Cargo.toml), both debug and release, before each
   incremental commit.
4. Do NOT claim per-symbol 0.01bp pdflatex parity in the report — use the
   measured-reality framing above instead. It's fine (and correct) to use
   pdflatex/fontmath.ltx as the oracle for *which Unicode character and class*
   a command means; it is not architecturally capable of matching pdflatex's
   pixel geometry through the Adobe Symbol font.
5. No code files were touched yet — `crates/compiler/src/math.rs` and
   `export.rs` are still exactly at whatever `main` had at checkpoint time.
