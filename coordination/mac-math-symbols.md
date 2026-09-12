# mac-math-symbols — handoff and compiler request list

Lane `mac-math-symbols` (Claude Code subagent, parent `mac-claude-a`, machine
`mac-m1max-a`), branch `agent/mac-render-pipeline/math-symbols` (base
`agent/mac-render-pipeline/text-gaps` 73217526). Owner scope:
`crates/render-pipeline` only; the compiler, math-layout, font-engine and
paragraph-layout are vendored under `crates/render-pipeline/vendor/` and were
not edited.

## Pin change

| vendored crate | old | new | why |
| --- | --- | --- | --- |
| `compiler` | `745f327` (main) | `49e6eb43` (`origin/agent/claude/compiler-foundation`) | ~40 more math control words in `COMMAND_GLYPHS`, `\left`/`\right` pairing (delimiters still emitted as plain symbols), starred headings, title diagnostics no longer dropped, unknown-delimiter errors, `\nu` |

Behaviour changes observed through the pipeline after the re-pin (nothing in
the render-pipeline goldens or the incremental byte-identical gate changed —
no golden regeneration was needed; `cargo test --release`: 62 tests, all pass,
1 ignored as before):

- HW1 (`fixtures/real-world/hw1/HW1.tex`): diagnostics **130 → 86**, pages 3 → 3.
  Gone: 7 `\section*` "requires a braced argument" errors, the `\left`/`\right`
  and `\in`/`\forall`/`\exists`/`\vee`/`\Rightarrow` "not supported in math mode"
  errors. Still there and now visible inside titles: 7 × `\hfill`, 7 × `\normalfont`.
- Visual-oracle ranking (`tools/visual-oracle/rank.py --only hw1`, mac-shell
  9ba9851c tools, reference `HW1-reference.pdf`): p1 median shift
  (−1.97, −50.51) → (−1.31, −14.26) bp, reflowed words 55 → 23, differing
  pixels 7.8 % → 6.8 %; p2 aligned 123 → 170, median dy −199 → −82 bp,
  reflowed 34 → 23; p3 reflowed 90 → 43, differing px 3.9 % → 3.9 %. The
  remaining shifts are the unimplemented `center`, `enumerate[...]`, `array`,
  `\hrule`/`\vspace`, `\bigl`/`\bigr`, `\mathbb`, `\quad`/`\qquad` and the
  `\setlength{\parskip}` / `\parindent` settings listed below. Reports:
  `crates/render-pipeline/docs/evidence/hw1-math-symbols/`.

## Symbol table (render-pipeline side, compiler pin 49e6eb43)

Metrics: plain.tex/fontmath.ltx family+slot from the CM-identical `lmmi`/`lmsy`/
`lmex` TFM tables embedded in math-layout and the installed `rm-lmr12/8/6`
for family 0; painting: Latin Modern Math (families 1–3) and the optical
`lmroman12/8/6` faces (family 0). Oracle = MacTeX 2026 pdflatex, article 12pt,
`lmodern`, one formula per line inline + display
(`crates/render-pipeline/fixtures/math-symbols/*.tex`), compared with
`tools/visual-oracle/rank.py --harness-fixtures` (word origins from the
reference content stream vs our rendering-v2 glyph runs; 144 dpi raster
diff via Ghostscript). "delta" is the |dx|/|dy| of the word after the
formula (which integrates every glyph width and inter-atom space in it) and
of every aligned math word; full tables in
`crates/render-pipeline/docs/evidence/math-symbols/visual-oracle-report.md`.

| family (fixture) | symbols | face / slot | class | delta (bp) | raster diff px |
| --- | --- | --- | --- | --- | --- |
| Greek lower (01) | α β γ δ θ λ μ ν π σ φ ω | lmmi12 `0B–21` → LM Math italic block; `\phi` painted as the straight ϕ (U+1D719) because cmmi `1E` is the straight form | Ord | ≤ 0.01 / 0.00 | 1471 (0.08 %) |
| Greek upper (02) | Γ Δ Θ Λ Ξ Π Σ Υ Φ Ψ Ω | rm-lmr12 `00–0A` → lmroman12-regular | Ord | 0.00 / 0.00 | 152 (0.01 %) |
| Binary (03) | × ÷ ± · ∪ ∩ ∧ ∨ | lmsy10 `02 04 06 01 5B 5C 5E 5F` (`\cdot` U+00B7 reclassed as U+22C5) | Bin | ≤ 0.01 / 0.00 | 1694 (0.09 %) |
| Relations (04) | ≤ ≥ ≠ ≈ ≡ ∼ ⊂ ⊆ ⊃ ⊇ ⊥ ∈ ∋ ∉ | lmsy10 `14 15 (36+cmr 3D) 19 11 18 1A 12 1B 13 3F 32 33 (36+32)`; `\neq`/`\notin` are `\not`+relation composites with the zero-width slash | Rel | 0.00 / 0.00 | 3535 (0.18 %) |
| Arrows/logic (05) | → ← ⇒ ⇔ ∀ ∃ ¬ | lmsy10 `21 20 29 2C 38 39 3A` | Rel / Ord | ≤ 0.01 / 0.00 | 1595 (0.08 %) |
| Ordinary (06) | ∞ ∅ ∂ ∇ | lmsy10 `31 3B 72`, lmmi12 `40` | Ord | 0.00 / 0.00 | 170 (0.01 %) |
| Big operators (07) | ∑ ∫ ∏ with `_{}^{}` (display limits, `\int` nolimits) | lmex10 `50/58 52/5A 51/59` via the MATH vertical variants | Op | ≤ 0.01 / 0.00 (20/20 words) | 2073 (0.11 %) |
| Delimiters (08) | `\left( \frac{a}{b} \right)`, `\left[ … \right]`, `\left( x \right)` | rm-lmr12 `28/29/5B/5D` then lmex10 `00–03` chain | Inner | 0.00 / 0.00 for the trailing word; the 4.56 bp on `x` is the word-alignment merge (`(x)` is one reference word), not geometry | 1601 (0.08 %) |
| `\angle` (09) | ∠ | **unmatched**: base LaTeX's `\angle` is a constructed `\vbox{\not\mkern14mu / leaders\hrule}` macro, not a glyph; math-layout has no constructed-box nucleus | Ord | −6.59 bp (empty box, typed `math_limitation`) | 1085 (0.06 %) |

Raster diffs are ink-weight only (CFF outlines vs Type 1 hinting; see
`docs/oracle-evidence.md`); no fixture has a word farther than 0.01 bp from
the reference except `\angle` and the `(x)` alignment artefact above.

## Request list for the compiler owner (Commander claude / Kabir)

What HW1 (`fixtures/real-world/hw1/HW1.tex`, byte spans on
`origin/agent/mac-claude-a/mac-shell`) and the corpus still need the parser
to expose. The render pipeline typesets everything the compiler exposes
through `Nucleus::Symbol`; each item below is a compiler-side gap that
currently produces an "is not supported" error and drops the tokens.

| # | need | HW1 spans (start–end bytes) | what the pipeline needs from the parser |
| --- | --- | --- | --- |
| 1 | `\mathbb{R}` / `\Z` / `\Q` / `\N` (via `\newcommand`) | 1283–1285, 1894–1896, 3092–3094, 3184–3186, 3200–3202, 3218–3220, 3514–3516, 3532–3534, 3630–3632, 3663–3665, 3674–3676 (11) | a `Nucleus::Styled { font: Blackboard, body }` (or `Symbol` with a font tag) — the pipeline maps msbm slots; also `\newcommand` expansion of `\Z`/`\R`/`\Q`/`\N` |
| 2 | `\mid` | 1341–1345, 1377–1381, 1466–1470, 1498–1502 (4) | `("mid", "\u{2223}")` in `COMMAND_GLYPHS` — but as a **Rel** (the pipeline classes U+2223 as Rel already) |
| 3 | `\setminus` | 3632–3641, 3665–3674 (2) | `("setminus", "\u{2216}")` (Bin; lmsy `6E`, already mapped) |
| 4 | `\bigl` `\bigr` (also `\Bigl` `\bigl(` `\bigr)` `\bigl(…\bigr)`) | `\bigl` 2720–2725, 2774–2779, 2865–2870, 3230–3235, 3544–3549; `\bigr` 2746–2751, 2800–2805, 2884–2889, 3247–3252, 3561–3566 (10) | a delimiter atom with an explicit size step (`Big1..Big4`) — the pipeline selects the lmex chain index; today these are dropped and the parenthesis is set at text size |
| 5 | `\quad` / `\qquad` in math | `\qquad` 2459–2465, 2475–2481, 3801–3807, 3820–3826, 3840–3846; `\quad` 2815–2820, 2835–2840 (7) | a `Nucleus::Kern(mu)` / glue atom (`\quad` = 18 mu, `\qquad` = 36 mu); the pipeline has `\,`/`\;` handling ready in math-layout spacing |
| 6 | `\text{…}` adoption | 1984–1989, 2035–2040, 2086–2091, 2137–2142, 2465–2470 (5) | adopt `Nucleus::Text(String)` (isolated candidate in `crates/preview-controller/docs/handoffs/hw1-text-candidate/`); the pipeline's `src/mathtext.rs` arm is behind the `compiler-text-nucleus` feature waiting for the pin |
| 7 | `\left`/`\right` as structure | (fixed on the pipeline side by re-deriving the fence from the source bytes before each delimiter) | ideally `Nucleus::Delimited { left, right, body }` in the compiler so the fence does not depend on source re-reading; `\left.` (null) then needs `None` |
| 8 | `array` environment in math | 1966–1972 (`\begin{array}{ll}`), 2181–2185 (`\end{array}`); the `&` becomes a `math_limitation` "no math glyph for '&'" | an aligned-rows nucleus (`Nucleus::Array { cols, rows }`); the pipeline can set column boxes with math-layout |
| 9 | `\Longrightarrow` | 2820–2835 (1) | `("Longrightarrow", "\u{27F9}")` — lmsy `29` + `3D` composite (`\Relbar\joinrel\Rightarrow`); the pipeline will compose it like `\neq` |
| 10 | `\hfill` (in `\problem` titles and body) | 1257–1265, 1544–1552, 1796–1804, 2340–2348, 3017–3025, 3606–3614, 4103–4109 (7) | an `Inline::Fill` (hfil glue) so the pipeline's paragraph builder can set right-aligned points |
| 11 | `\normalfont`, `\bfseries`, `\Large`, `\LARGE` (font switches inside groups) | `\normalfont` 1257–1265 …, 4110–4121; `\bfseries` 590–599, 637–646; `\Large` 584–590; `\LARGE` 631–637 | scoped style switches in `Inline` (the pipeline already re-derives `\textbf`/`\emph` from spans; declared switches need the group extent) |
| 12 | `center` environment | 545–555 region (`\begin{center}` … `\end{center}`); warning "environment 'center' is not implemented" | a block with `Alignment::Center` (the pipeline's page builder can centre lines) |
| 13 | `enumerate` with `enumitem` options (`\setlist[enumerate]{leftmargin=…}`, `[(a)]`) | 270–278 (`\setlist`), the `\begin{enumerate}` blocks | `Block::List { kind, label_style, items }` with `\item` boundaries; the pipeline's line builder needs the hanging indent |
| 14 | `\setlength{\parindent}{0pt}`, `\setlength{\parskip}{0.65em}` | 213–223, 224–234, 241–251, 252–260 | preamble parameters surfaced on the parsed document (the pipeline already reads `geometry`); `\parskip` changes every inter-paragraph glue on HW1 |
| 15 | `\hrule`, `\vspace{0.6em}`, `\newpage`, `\pagestyle{empty}` | 850–856, 857–864, 4067–4075, 545–555 | `Block::Rule`, `Block::VSpace(dimen)`, an eject marker (the pipeline recovers `\newpage` from the source gap today), page-style flag |
| 16 | `\phi`/`\epsilon` code points | (corpus, not HW1) | per unicode-math, `\phi` → U+03D5 and `\varphi` → U+03C6, `\epsilon` → U+03F5 and `\varepsilon` → U+03B5; today `("phi", "φ")` is the loopy letter while cmmi `1E` (the metrics) is the straight one — the pipeline paints by TFM slot, so this only matters for downstream Unicode consumers; `\epsilon` is not in the table at all |
| 17 | `\angle` | (corpus) | not a glyph in base LaTeX (constructed `\not`+rule vbox); needs a constructed-box nucleus in math-layout, or amssymb's msam `5C` when `amssymb` is loaded (HW1 loads it) |

## Checkpoints

- 2026-09-12T18:00Z: branch created, compiler re-pinned, build green.
- 2026-09-12T18:25Z: composites/fences/extra slots implemented, probe shows
  every HW1 math control word except `\mathbb`/`\mid`/`\setminus`/`\bigl`/
  `\bigr`/`\quad`/`\qquad`/`\text`/`\Longrightarrow` typesetting.
- 2026-09-12T18:40Z: oracle fixtures + visual-oracle runs done (tables above),
  HW1 before/after ranked, tests written (5 new, 62 total).
