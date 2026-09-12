# mac-realworld-corpus — real-world corpus acceptance harness

Lane: `mac-realworld-corpus` (Claude Code subagent, parent `mac-claude-a`, machine
`mac-m1max-a`). Branch `agent/mac-realworld-corpus/corpus` from
`origin/agent/mac-claude-a/mac-shell` @ `daa541e3`. Owned paths:
`fixtures/real-world/**`, `tools/real-world-corpus/**`,
`docs/evidence/real-world-corpus-<UTC>/`, this file, `coordination/agents/mac-realworld-corpus.json`.
No crate is edited; no parent-retained Swift file is touched.

## Coverage audit (2026-09-12T14:30Z, mandatory first step)

Searched `apps/mac/Tests/FlashTeXMacTests/*`, `apps/mac/Sources`, `tools/**`,
`docs/evidence/*`, `tools/native-validation/mac-live/reports/20260912T110944Z.md`,
`crates/*/tests`, `crates/pdf/scripts` for `hw1`, `real-world`, `corpus`.

Already covered (does NOT need re-doing):

| where | what it covers |
|---|---|
| `fixtures/real-world/hw1/{HW1.tex,HW1-reference.pdf,gap-report-2026-09-12.md}` | the one user-provided document + a hand-run diagnostic count for `flashtex-render` 9aaec57a (status `recovered`, 3 pages) |
| `tests/tex-corpus/` (FT-011, 14 synthetic cases, `validate.py --emit-request`) + `crates/compiler/tests/corpus_gate.rs` (`corpus_statuses_are_pinned`-style pin over the 14 cases) | synthetic feature cases; pins today's status per case; no pdfLaTeX reference, no pixels |
| `crates/compiler/tests/unsupported_inventory.rs` → `UNSUPPORTED.md` | hand-written per-feature snippets, exact diagnostic per snippet; not derived from real documents |
| `crates/pdf/scripts/corpus_report.py` + `crates/pdf/docs/corpus-report.md` | runs `tests/tex-corpus` through compiler → `flashtex-pdf --verify` → `sips` opens; no reference PDF, no pixel comparison |
| `tools/raster-compare/compare.py` (+ `test_compare.py`) | exact RGBA8 PNG pair comparator with provenance JSON; no producer, no reference generation |
| `crates/rendering-core/tests/pdf_compare.rs`, `crates/font-engine/examples/corpus_evidence.rs` | crate-internal fixtures (rendering-core/font-engine owned; not touched) |
| `tools/native-validation/mac-live/reports/20260912T110944Z.md` | typing bench + packaged capture cycle; contains no `hw1`/real-world mention |
| `apps/mac/Tests/FlashTeXMacTests/PreviewAnchoringTests.swift` | the word "corpus" only in a synthetic anchoring paragraph |

NOT covered anywhere (this lane's work): a harness that takes *real documents*
with a pdfLaTeX reference, runs both producers (`flashtex-compiler`,
`flashtex-render`) over runtime-v1 JSON Lines, rasterizes producer vs reference,
derives a construct inventory from the .tex sources by tokenizer and reports
per-construct supported/recovered/unsupported with the exact diagnostic; a corpus
of more than one document; a ranked unsupported list with crate owner.


## Commander ruling (issue #2 5646504285) — acknowledged and applied

- `fixtures/real-world/hw1/HW1.tex` and `HW1-reference.pdf` are byte-immutable (diff vs daa541e3: none).
  The MacTeX regeneration lives beside them as `reference-mactex2026.pdf`
  (sha256 `0f6ff3345b4b07acfd963c091ff6099bd501a78f39576b3fb57192378983feef`,
  pdfTeX 3.141592653-2.6-1.40.29 (TeX Live 2026)); it is NOT byte-identical to the
  user's PDF and differs by 5113 / 704 / 2402 pixels at 144 dpi on pages 1-3.
- Every ranked gap carries its owner (compiler / math-layout / render-pipeline);
  no render-pipeline workaround for parser gaps is suggested anywhere.
- Rank 1 is pinned: `\subsection requires a braced argument` (severity error,
  recovery "used an empty argument and continued"), emitted for each
  `\problem{n}{p}` call whose body expands to `\subsection*{Problem #1 \hfill \normalfont[#2 points]}`.
  Spans (HW1.tex UTF-8 byte offsets, all 7 `\problem` calls): 1257-1265, 1544-1552,
  1796-1804, … (full list in report.json) plus lecture-notes/main.tex
  `\subsection*{Exercises}`; rank 21 `\section requires a braced argument` is the same
  gap for a bare `\section*` (math-sheet, 7 occurrences).

## Results (docs/evidence/real-world-corpus-2026-09-12T144246Z)

- `uptime` at start: `10:42  up 1 day,  9:35, 3 users, load averages: 4.04 7.13 9.80` (1-min < 15).
- 8 fixtures, all compile with MacTeX (0 LaTeX warnings, 0 overfull boxes); references committed with SHA + version.
- `flashtex-compiler` (this checkout, crates/compiler @ daa541e3): 8/8 `recovered`.
- `flashtex-render` @ 9aaec57a: 6/8 `recovered`, 2/8 **panic** (exit 101, no reply) — `cv`, `article-twocolumn`;
  minimal reproduction `Hello \\` + blank line → `vendor/paragraph-layout/src/linebreak.rs:988:21 slice index starts at 6 but ends at 5`
  (in `tools/real-world-corpus/README.md`; owner crates/paragraph-layout, not patched by this lane).
- Page counts: render matches the reference on hw1 (3), input-bibliography (2), letter (1), unicode-accents (1);
  short by one page on lecture-notes (1 vs 2) and math-sheet (1 vs 2). Compiler: 2 vs 3 on hw1, 2 vs 1 on cv.
- Pixel differences (144 dpi grey, gs 10.08.0) are 3.9 %–17.4 % of the page on every compared page — reported, not ranked, not a score.
- Construct inventory: 253 distinct constructs, 1489 occurrences. Per producer — compiler: 168 unsupported / 32 recovered / 53 no-diagnostic;
  render: 147 / 43 / 63 (render's lower unsupported count is partly the two crashed documents contributing no diagnostics).
- 188 distinct diagnostic keys ranked; top 30 below.

### Top-30 (paste-ready; owner in brackets; counts render / compiler)

1. [crates/compiler] `\subsection requires a braced argument` — render 8 / compiler 8 occurrences in 2 fixture(s) (hw1, lecture-notes); first span `hw1/HW1.tex:1257-1265` `re using.⏎⏎⏎\problem{1}{4}⏎Let $`
2. [crates/compiler] `math script marker used outside math mode` — render 41 / compiler 41 occurrences in 1 fixture(s) (math-sheet); first span `math-sheet/main.tex:405-406` `lign}⏎  \sum_{k=0}^{n} k `
3. [crates/compiler] `\in is not supported in math mode` — render 32 / compiler 32 occurrences in 3 fixture(s) (hw1, lecture-notes, math-sheet); first span `hw1/HW1.tex:1280-1283` `{4}⏎Let $a,b\in\Z_{>0}$.⏎\b`
4. [crates/compiler] `\mathbb is not supported in math mode` — render 30 / compiler 30 occurrences in 3 fixture(s) (hw1, lecture-notes, math-sheet); first span `hw1/HW1.tex:1283-1285` `⏎Let $a,b\in\Z_{>0}$.⏎\beg`
5. [crates/compiler] `\mid is not supported in math mode` — render 16 / compiler 16 occurrences in 2 fixture(s) (hw1, lecture-notes); first span `hw1/HW1.tex:1341-1345` `pose that $a\mid b$. Prove t`
6. [crates/compiler] `\hfill is not supported by this compiler version; unrestricted TeX math mode is not implemented` — render 7 / compiler 15 occurrences in 2 fixture(s) (cv, hw1); first span `hw1/HW1.tex:1257-1265` `re using.⏎⏎⏎\problem{1}{4}⏎Let $`
7. [crates/compiler] `\text is not supported in math mode` — render 11 / compiler 14 occurrences in 4 fixture(s) (article-twocolumn, hw1, lecture-notes, math-sheet); first span `hw1/HW1.tex:1984-1989` `{array}{ll}⏎\text{(a)} & \for`
8. [crates/compiler] `\vspace is not supported by this compiler version; unrestricted TeX math mode is not implemented` — render 1 / compiler 11 occurrences in 2 fixture(s) (cv, hw1); first span `hw1/HW1.tex:857-864` `er}⏎⏎\hrule⏎\vspace{0.6em}⏎⏎\te`
9. [crates/compiler] `\forall is not supported in math mode` — render 10 / compiler 10 occurrences in 1 fixture(s) (hw1); first span `hw1/HW1.tex:1997-2004` `text{(a)} & \forall m\,\exists `
10. [crates/compiler] `\qquad is not supported in math mode` — render 10 / compiler 10 occurrences in 2 fixture(s) (hw1, math-sheet); first span `hw1/HW1.tex:2459-2465` ` r)⏎        \qquad\text{and}\q`
11. [crates/compiler] `\ell is not supported in math mode` — render 9 / compiler 9 occurrences in 1 fixture(s) (lecture-notes); first span `lecture-notes/main.tex:1257-1261` `a$ and $c = \ell b$ for some`
12. [crates/compiler] `\begin is not supported in math mode` — render 7 / compiler 8 occurrences in 3 fixture(s) (article-twocolumn, hw1, math-sheet); first span `hw1/HW1.tex:1966-1972` `r false.⏎\[⏎\begin{array}{ll}⏎`
13. [crates/compiler] `\bfseries is not supported by this compiler version; unrestricted TeX math mode is not implemented` — render 2 / compiler 8 occurrences in 2 fixture(s) (cv, hw1); first span `hw1/HW1.tex:590-599` `⏎    {\Large\bfseries 21-128 and `
14. [crates/compiler] `\end is not supported in math mode` — render 7 / compiler 8 occurrences in 3 fixture(s) (article-twocolumn, hw1, math-sheet); first span `hw1/HW1.tex:2181-2185` `n\; D(m,n).⏎\end{array}⏎\]⏎F`
15. [crates/compiler] `\frac requires math mode` — render 8 / compiler 8 occurrences in 1 fixture(s) (math-sheet); first span `math-sheet/main.tex:421-426` `0}^{n} k &= \frac{n(n+1)}{2},`
16. [crates/compiler] `\gcd is not supported in math mode` — render 8 / compiler 8 occurrences in 1 fixture(s) (lecture-notes); first span `lecture-notes/main.tex:2718-2722` `both zero, $\gcd(a, b)$ is t`
17. [crates/compiler] `packages fontenc are recognised but not implemented` — render 6 / compiler 8 occurrences in 8 fixture(s) (article-twocolumn, cv, hw1, input-bibliography, lecture-notes, letter, math-sheet, unicode-accents); first span `hw1/HW1.tex:31-55` `]{article}⏎⏎\usepackage[T1]{fontenc}⏎\usepackage`
18. [crates/compiler] `packages geometry are recognised but not implemented` — render 6 / compiler 8 occurrences in 8 fixture(s) (article-twocolumn, cv, hw1, input-bibliography, lecture-notes, letter, math-sheet, unicode-accents); first span `hw1/HW1.tex:84-117` `]{inputenc}⏎\usepackage[margin=1in]{geometry}⏎\u`
19. [crates/compiler] `packages inputenc are recognised but not implemented` — render 6 / compiler 8 occurrences in 8 fixture(s) (article-twocolumn, cv, hw1, input-bibliography, lecture-notes, letter, math-sheet, unicode-accents); first span `hw1/HW1.tex:56-83` `1]{fontenc}⏎\usepackage[utf8]{inputenc}⏎\usepack`
20. [crates/compiler] `\normalfont is not supported by this compiler version; unrestricted TeX math mode is not implemented` — render 7 / compiler 7 occurrences in 1 fixture(s) (hw1); first span `hw1/HW1.tex:1257-1265` `re using.⏎⏎⏎\problem{1}{4}⏎Let $`
21. [crates/compiler] `\section requires a braced argument` — render 7 / compiler 7 occurrences in 1 fixture(s) (math-sheet); first span `math-sheet/main.tex:358-366` `yle{empty}⏎⏎\section*{Series and`
22. [crates/compiler] `\sum is not supported by this compiler version; unrestricted TeX math mode is not implemented` — render 7 / compiler 7 occurrences in 1 fixture(s) (math-sheet); first span `math-sheet/main.tex:401-405` `in{align}⏎  \sum_{k=0}^{n} k`
23. [crates/compiler] `\bigl is not supported in math mode` — render 6 / compiler 6 occurrences in 2 fixture(s) (hw1, math-sheet); first span `hw1/HW1.tex:2720-2725` ` \[⏎        \bigl(\forall x\i`
24. [crates/compiler] `\bigr is not supported in math mode` — render 6 / compiler 6 occurrences in 2 fixture(s) (hw1, math-sheet); first span `hw1/HW1.tex:2746-2751` `x\in S\;P(x)\bigr)⏎        \v`
25. [crates/compiler] `\date is not supported in the document preamble` — render 5 / compiler 6 occurrences in 6 fixture(s) (article-twocolumn, input-bibliography, lecture-notes, letter, math-sheet, unicode-accents); first span `input-bibliography/main.tex:204-209` `an Example}⏎\date{September 2`
26. [crates/compiler] `\exists is not supported in math mode` — render 6 / compiler 6 occurrences in 1 fixture(s) (hw1); first span `hw1/HW1.tex:2008-2015` ` \forall m\,\exists n\; D(m,n),`
27. [crates/compiler] `\le is not supported in math mode` — render 5 / compiler 6 occurrences in 3 fixture(s) (article-twocolumn, lecture-notes, math-sheet); first span `lecture-notes/main.tex:2013-2016` `and}\quad 0 \le r < b.⏎\]⏎\`
28. [crates/compiler] `\mathrm is not supported in math mode` — render 6 / compiler 6 occurrences in 1 fixture(s) (math-sheet); first span `math-sheet/main.tex:1027-1030` `tyle \int u \dd v = uv - \i`
29. [crates/compiler] `\newtheorem is not supported in the document preamble` — render 6 / compiler 6 occurrences in 1 fixture(s) (lecture-notes); first span `lecture-notes/main.tex:177-188` `enumerate}⏎⏎\newtheorem{theorem}{Th`
30. [crates/compiler] `\quad is not supported by this compiler version; unrestricted TeX math mode is not implemented` — render 2 / compiler 6 occurrences in 2 fixture(s) (cv, math-sheet); first span `math-sheet/main.tex:535-540` `rac{1}{1-x} \quad (|x|<1), &⏎`

## Durable checkpoint

- Task: mac-realworld-corpus (brief in parent scratchpad `prompt-mac-realworld-corpus.md`), rev 1.
- Branch `agent/mac-realworld-corpus/corpus`, worktree `.claude/worktrees/agent-a9e38bb939d90fda1`.
- Consumed parent SHA: `daa541e3` (origin/agent/mac-claude-a/mac-shell). Producer pins:
  compiler = this checkout's `crates/compiler`; `flashtex-render` = `origin/agent/mac-render-pipeline/unified` @ `9aaec57a`
  (git-archived into ignored `tools/real-world-corpus/target/render-pipeline-9aaec57a/`).
- Oracle: MacTeX 2026 `/usr/local/texlive/2026/bin/universal-darwin/pdflatex`, `SOURCE_DATE_EPOCH=0 FORCE_SOURCE_DATE=1`. Oracle only.
- Rasterizer: `pdftoppm` absent on this Mac; `/usr/bin/sips` present (the harness reports which it used).
- Committed: `cb5f03ca` (harness, fixtures, evidence, registration); this registration refresh follows as a second commit. Dirty files: none (scratch under ignored `tools/real-world-corpus/target/`).
- Next commands: none pending for this lane; rerun = `tools/real-world-corpus/run.sh`; tests = `python3 -m unittest discover -s tools/real-world-corpus -p 'test_*.py'`.
- Staffing/billing: shared Claude Max quota with parent; no purchases.
