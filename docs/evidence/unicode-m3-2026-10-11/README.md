# Unicode mode's M3 gate, measured (2026-10-11)

Lane UNICODE-M3 (engineer under the Commander `kabir-claude`). The gate is the
M3 row of [`docs/design/modes/PROPOSAL.md`](../../design/modes/PROPOSAL.md)
("Unicode in the product"). Retirement stages S6/S7 wait on it. Every
criterion that GitHub-hosted runners can measure was run there by
[`.github/workflows/unicode-m3.yml`](../../../.github/workflows/unicode-m3.yml).
The harnesses are in [`tools/unicode-m3/`](../../../tools/unicode-m3/README.md).

- **Oracle:** TeX Live 2026's `xelatex`/`xetex`, XeTeX 3.141592653-2.6-0.999998.
  This is the scheme-full `texlive/texlive@sha256:a7ae4dfa…` image that
  nightly.yml pins and the no-TeX-Live bundle is packed from. It is never in
  the product path.
- **Runners:** ubuntu-latest (Linux x86-64; `perf` cannot count there, so
  instruction counts come from callgrind) and macos-15 (Apple M1, virtualised,
  with no TeX Live).
- **Runs:** [38178866539](https://github.com/flash-tex/flashtex/actions/runs/38178866539)
  for every criterion, and [38185041540](https://github.com/flash-tex/flashtex/actions/runs/38185041540) for P-T1 (with the panic's frames) and the export after the colour fix. Its no-TeX-Live and Core Graphics totals equal the first run's.

**Verdict: M3 is NOT MET.** Of the gate's seven criteria:
- two are met: the `[fonts]` fixtures, and Classic unchanged so far as hosted
  runners can show it;
- four are not met: Unicode corpus P-T1, the export gate, the no-TeX-Live run,
  and the §1.2 edit and reopen rows;
- one cannot run as defined: Unicode mode has no incremental engine to sweep.

The §1.2 wall-time rows also wait for the reference Mac. Below, **verified**
means a number measured in the runs above. **Belief** means an inference that
was not measured.

## Checklist

| # | M3 criterion | Verdict | Numbers (verified unless marked) |
|---|---|---|---|
| 1 | Unicode corpus (fontspec, unicode-math, polyglossia, xeCJK; Q2's corpus) at P-T1 100 % | **not met** | **46 of 49** equal (P-T1 log, exit status, XDV): fontspec 15/15, `[fonts]` 6/6, unicode-math 7/7, xeCJK 16/16 (thuthesis included), polyglossia 2/5. Failing: u03 (Hebrew) and u04 (Arabic), XDV differs (no bidi, ranked 1); `polyglossia` (the manual), the port panics (ranked 2). 224.7 M log lines compared |
| 2 | Export gate per Q2 (structural; 2×/3× pixels) | **not met** | 23 of 49 cases pass structurally. Another 19 differ only in `ToUnicode`; 15 of them because FlashTeX writes a `ToUnicode` where xdvipdfmx writes none (CID-keyed CJK fonts, by design, PLAN.md §4B). 3 differ for real: u03, u04 (no bidi) and u11 (beamer's links); xeCJK-example-mathblock's colour was a fourth, fixed here. 4 are cases the oracle itself errors on (2 with no comparable PDF). 204,703 glyphs compared; positions within 0.0003 bp of TeX's XDV wherever glyphs pair. Core Graphics: 47 pairs, 1 at 0 px; 187,677 px in all at 2×, 432,795 px at 3×. That is a floor, not 0, as the 2026-10-09 ruling expects |
| 3 | §1.2 latency on xelatex bench documents | **not met** (edit and reopen rows); preamble row **pending the reference Mac** | Every Unicode compile is a full run. Start-up to the format loaded is 848 M instructions. A full run is 1,504 M (article) to 30,554 M (300 pages); xelatex takes 1,514 M to 14,253 M. Host on ubuntu-latest, first page: article 144 ms, unicode-math 663 ms, edit = preamble = reopen. Belief: 848 M instructions take far more than 16 ms (edit) or 100 ms (reopen) on any M-series core |
| 4 | Incremental soundness sweeps on Unicode | **cannot run as defined** (no incremental Unicode engine, INCREMENTAL.md §5) | What does exist was swept: the host's reuse between compiles (hot spares, `.aux` passes). 53 documents, 318 compiles; host A (spares) equals host B (cold) in pages, fonts, forms and images, frame for frame, in 318 of 318 |
| 5 | A no-TeX-Live run of the corpus from the bundle | **not met** | macos-15, sandboxed, bundle `dc72dde1…` (HELLO: no TeX Live, resolver `bundle`). 27 of 50 compile cleanly with a PDF. Each of those, except the macOS-only l004, is **byte-identical** to the Linux TeX Live run's PDF (26 of 26). The other 23: 10 need packages the bundle lacks (`bidi`, `thuthesis`, `zhnumber`, `l3draw`, `showexpl`, `typogrid`, `unicode-math.dtx`). 13 end with status `error` and fonts missing: fonts the bundle lacks, or TeX Gyre named by family, which only TeX Live's fontconfig set-up finds here (f01–f04, f06, u08, fontspec-example). fontspec-example and unimath-symbols fail with TeX Live too |
| 6 | `[fonts]` fixtures: P-T1 + XDV against xelatex on the effective source; precedence (document font commands win) | **met** | 6 of 6 fixtures: P-T1 log, exit status and XDV equal to xelatex on the same first line, which comes from FlashTeX's own generator (`fonts_line`). Expected and forbidden fonts hold in both engines: the role applies, `\setmainfont` and `\setmathfont` in the document win, a local `\fontspec` group wins inside it, and a refused name is not applied. All 6 are also structurally equal in the PDF. `tests/host.rs` with TeX Live: 7 of 7 pass (the `[fonts]` precedence test is one) |
| 7 | Classic lockstep, P-T1/P-T2 fixtures, latency, `flashtex-host` size and start-up unchanged | **met** for what hosted runners show | The U3 merges (#1732, #1734, #1762) change no file of Classic's engine: only `crates/flashtex-xetex`, the bundle lock and contents, packaging, docs, and `scripts/gate.sh`'s list of what builds flashtex-xetex (each merge diffed against its first parent). `flashtex-host` built at each U3 merge and at its first parent: size equal for each pair (10,043,160 / 10,043,160; 10,043,160 / 10,043,160; 10,798,384 / 10,798,384 bytes); SHA-256 equal for all three pairs in run 38173318789. Start-up to HELLO: 311.9 M against 312.0 M, 312.1 M against 312.1 M, and 302.3 M against 302.4 M instructions. ci.yml after #1762's merge (run 38028759045): engine parity lockstep, P-T fixtures 1/2 and 2/2, and engine tests all pass. Classic's T7 latency needs the reference Mac: **pending** |

## 1. P-T1 + XDV (criterion 1)

`tools/xetex-lockstep/latex.py --cases-dir` (run 38185041540). Each
engine builds its own `xelatex.fmt`. The two format logs differ in one known
line, the string pool's count (97,840 against 97,825 strings; latex.py
explains it). Each case then runs untraced, then traced. 46 of the 49 Linux
cases (l004 needs macOS's fonts) are compared under `\tracingall`. Three long
documents (polyglossia, thuthesis-example, unimath-symbols) are compared by
box dumps at every `\shipout` only.

| family | cases | equal | not equal |
|---|---:|---:|---|
| fontspec (l001–l010 less l004, d01–d05, u08, u11, u12, fontspec-example) | 15 | 15 | - |
| `[fonts]` fixtures (f01–f06) | 6 | 6 | - |
| unicode-math (l007–l009, u09, u10, unimath-example, unimath-symbols) | 7 | 7 | - |
| xeCJK (u05–u07, the 12 xeCJK examples, thuthesis-example) | 16 | 16 | - |
| polyglossia (u01–u04, the polyglossia manual) | 5 | 2 | u03, u04, polyglossia |
| **all** | **49** | **46** | |

Six cases end in a TeX error in xelatex too. Four of them end in the same
error in both engines, so they count as equal:
- fontspec-example and xeCJK-example-fallback: fontspec cannot find a font;
- unimath-symbols: `unicode-math.dtx` is not in the tree;
- xeCJK-example-listings.

The other two count as failures: u04 (`Missing number`, and the bidi
difference) and polyglossia (the missing Syriac font, then the panic).

## 2. Export (criterion 2)

`tools/xetex-pdfparity/run.py --cases-dir … --no-visual` on Linux, then
`tools/unicode-m3/visual.py` on macos-15 over the same PDFs (Core Graphics):

| class | cases |
|---|---|
| structurally equal | 23 of 49: d01–d05, f01–f06, l001–l003, l005, l006, l009, l010, u01, u02, u10, u12, unimath-example |
| `ToUnicode` only, xdvipdfmx writes none (CID-keyed CJK fonts) | 15: u05–u07, thuthesis-example, 11 xeCJK examples (mathblock included, after the colour fix) |
| `ToUnicode` only, code-point choice (two code points, one glyph) | l007, l008, u08 (small capitals read as PUA) |
| `ToUnicode` of `.notdef` | u09 (FlashTeX U+FFFF, xdvipdfmx none) |
| differs | 3: u03 and u04 (right-to-left), u11 (beamer's navigation links). xeCJK-example-mathblock differed in 128 glyph colours until this lane's fix (run 38178866539 against 38185041540) |
| the oracle errors in this environment | fontspec-example, unimath-symbols (equal otherwise); xeCJK-example-fallback (xelatex writes no PDF); polyglossia (FlashTeX panics, damaged PDF); xeCJK-example-listings is also counted above (`ToUnicode` only, including CMTT9's visible space, which xelatex maps and FlashTeX does not) |

Positions: once glyphs pair, the PDF's glyph origins are within 0.000252 bp
of TeX's XDV positions (0.001 bp is the tolerance). The XDV check now puts
the page origin at the paper-size special's unrounded height, as dvipdfmx
does. Before that change, beamer's 96 mm page showed a spurious 0.004 bp.

## 3. Latency (criterion 3)

Run 38178866539, ubuntu-latest. Instructions are user-space, counted by
callgrind; each is the best of 3 runs with a warm `.aux`. Wall times are
from wait4 on a shared x86 runner, so they are **not** the reference
hardware's.

| document | FlashTeX, PDF (M instr) | ms | FlashTeX `-no-pdf` | xelatex `-no-pdf` | xelatex + xdvipdfmx |
|---|---:|---:|---:|---:|---:|
| b1 article + amsmath + graphicx | 1,504 | 156 | 1,307 | 1,514 | 2,473 |
| b2 fontspec, TeX Gyre Termes by name | 3,320 | 440 | 3,284 | 2,528 | 3,448 |
| b3 unicode-math + amsmath | 7,001 | 656 | 6,900 | 5,289 | 6,220 |
| b4 fontspec, 11 pages | 4,565 | 502 | 3,671 | 3,109 | 4,146 |
| b5 fontspec, about 300 pages | 30,554 | 4,160 | 15,425 | 11,241 | 14,253 |
| start-up to the format loaded | 848 | 72 | | 1,087 | |

The host (`flashtex-host-unicode`, dl3-client), time to the first page in ms:

| document | open | reopen | 5 edits in a paragraph | preamble edit |
|---|---:|---:|---|---:|
| b1 | 145 | 144 | 142–145 | 144 |
| b2 | 446 | 446 | 444–447 | 449 |
| b3 | 663 | 662 | 662–667 | 664 |
| b4 | 287 | 290 | 285–293 | 288 |
| b5 | 296 | 293 | 292–295 | 294 |

Against the §1.2 targets:
- **Edit, ≤ 16 ms (host ≤ 11 ms):** not met. Every keystroke is a full run;
  the spare saves only the process start.
- **Reopen, ≤ 100 ms to the first page:** not met.
- **Preamble edit, ≤ 400 ms to the first page:** met on this runner for b1,
  b4 and b5; not met for b2 (446 ms) and b3 (664 ms). Pending the reference
  Mac.

Two further observations:
- **FlashTeX's own PDF writing costs more than xdvipdfmx at length:** b5
  costs 15.1 G instructions beyond `-no-pdf`, where xdvipdfmx adds 3.0 G.
- **`flashtex-host-unicode` itself:** 8,589,016 bytes (7,197,256 stripped),
  HELLO in 38 ms median, 644 M instructions.

## 4. Soundness (criterion 4)

`tools/unicode-m3/host_sweep.py`: the whole corpus and the five bench
documents. Each gets five scripted edits and a revert, compiled by host A
(spares on) and by host B (spares off, every compile cold), each in its own
copy of the project. Result: 318 of 318 compiles are equal. Of these, 40
compile to a TeX error in both hosts (the cases the oracle errors on). This
does not replace the sweep the gate asks for: that needs the incremental
Unicode engine of INCREMENTAL.md §5.

## 5. No TeX Live (criterion 5)

`tools/unicode-m3/notex_corpus.py --mode notex` on macos-15 ran the host
with an empty environment, the lock's bundle and the user's consent, and
sandbox-exec denying TeX Live's paths. The macOS PDFs are compared with the
Linux run (`--mode texlive`) and with xelatex's PDFs. All 26 clean cases that
both runs have are byte-identical across platforms and resolvers.

Missing from the bundle:
- **packages:** `bidi` (polyglossia's right-to-left languages), `thuthesis`,
  `zhnumber` (ctex's `\zhnumber`, xeCJK's CJKfntef example), `l3draw`,
  `showexpl`, `typogrid`, and `unicode-math.dtx` (a source file).
- **fonts:** Harano Aji, Fira Math, New Computer Modern, UnBatang.
- **TeX Gyre by family name:** the bundle's fonts are not in the font index.
  Belief, not measured: xelatex on a Mac without those fonts installed fails
  the same way, because Core Text does not see TeX Live's fonts either.

## 6. `[fonts]` (criterion 6)

The details are in criterion 6's row of the checklist above.

## Ranked failures, with repros

Each repro is a case of the corpus: `python3 tools/unicode-m3/corpus.py
assemble --out C --fonts-line BIN --only NAME`, then
`python3 tools/xetex-lockstep/latex.py --engine flashtex-xetex --cases-dir C`
or `tools/xetex-pdfparity/run.py --engine … --cases-dir C --no-visual`.

1. **Right-to-left text is laid out left to right (no ICU bidi).**
   `crates/flashtex-xetex/src/native/mod.rs` `set_native_metrics` warns and
   lays out LTR: ICU is not vendored, so `ubidi` is absent. P-T1 logs are
   equal, but the XDV differs from byte 570 (u03, Hebrew) and from byte 192
   (u04, Arabic). In the PDF, 100 of 154 Hebrew glyphs are unpaired.
   Repros: u03-polyglossia-hebrew, u04-polyglossia-arabic. Size: a lane of its
   own (vendor TeX Live 2026's ICU, port `measure_native_node`'s bidi branch
   and `\XeTeXlinebreaklocale`).
2. **Engine panic on polyglossia's manual.**
   `flashtex-engine/src/ix.rs:44` (index out of bounds) in both passes, after
   the fontspec error for the missing Syriac font that xelatex also reports.
   xelatex exits 1 and writes 106 pages; the port exits 101 with a damaged
   PDF. The message is `index out of bounds: the len is 5000000 but the index is 18446744073441116161`: `mem[null]`, null being min_halfword −0x0FFFFFFF. The frames are `Globals::main_control` ← `tex_body` ← `driver::run`, which fits the native-word merge loop of `main_control` (`main_ppp` walking `link`s, xetex.web's "collapse native words") running off the list. Belief: the earlier divergence is the right-to-left text that is laid out left to right (the bidi warning precedes the panic). Repro: case `polyglossia`.
   **Root cause (lane M3-PANIC, 2026-10-12), not bidi:** ICU's line breaker was a stub (`linebreak_next` returned `UBRK_DONE` at once), so under `\XeTeXlinebreaklocale` `do_locale_linebreaks` appended nothing. A word merged with the native word before it then left `tail` at the flushed node, and the walk at `body_7.rs:6738` ran off the list (located with `#[track_caller]`, run 38201493482). polyglossia's Chinese sets the locale `zh` (`\textchinese{\chinesenumeral{753}}`). Fixed by `crates/flashtex-xetex/src/native/linebreak.rs`: ICU 78.2's own line-break tables (`third_party/icu`) run by a port of ICU's rule-based break iterator, ICU4X's dictionaries for Thai, Lao, Khmer and Myanmar. With it the manual runs to its end in both passes (exit 1 and 85 pages, as xelatex; the untraced log equal but for the string pool's count). Its box dumps (130,657 lines) still differ in 19 lines: box heights, depths and widths around right-to-left text (Amiri Arabic inside `\beginR`, laid out left to right: failure 1). Belief: all 19 are failure 1.
3. **beamer's navigation-symbol links are missing.** 63 links in xelatex's
   PDF are absent from FlashTeX's (3 pages), and FlashTeX writes 3 links
   xelatex has not. They are pgf-drawn icons inside `\hyperlink`. Repro:
   u11-beamer-fontspec.
4. **The bundle lacks what the corpus reads** (criterion 5's list).
   Owner: BUNDLE-PUBLISH (`tools/bundle/tl2026/extra.txt`).
5. **`ToUnicode` for `.notdef` is U+FFFF where xelatex writes none** (Fira
   Math, u09). It is marked as measured in `out/tounicode.rs`, so the rule
   needs a second measurement.
6. **Small-capital glyphs read as PUA code points** (TeX Gyre Termes, u08).
   xelatex reads them as the letter; the same class as the known l007/l008
   differences.
7. **Fixed in this lane:** a coloured font's RGB (fontspec `Color=`) is now
   written with 3 decimals, as dvipdfmx writes it (xeCJK-example-mathblock:
   128 glyphs, 0.1412 against .141). `crates/flashtex-xetex/src/out/page.rs`.

Not FlashTeX failures, but corpus cases whose oracle errors in this
environment (left in, because the engines must still agree):
- fontspec-example: a fontspec error, a font it names is not found here;
- unimath-symbols: needs `unicode-math.dtx`;
- xeCJK-example-fallback and xeCJK-example-listings: missing fonts and
  packages;
- u04 also has a "Missing number" error in both engines.
