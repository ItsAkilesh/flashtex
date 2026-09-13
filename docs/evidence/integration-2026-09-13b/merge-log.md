# Integration candidate `agent/kabir-claude/integration-2026-09-13b` — merge log

Integrator: kabir-claude integration subagent (Claude Opus 5), mac-m5pro-kabir.
Base: `origin/main` `bffd168a` (contains Commander's integration k, `c40d2a6c`).
Every PR head branch is merged with `git merge --no-ff`; conflicts and their
resolutions are recorded per merge. Nothing here is pushed to `main`.

## Skipped (already on main)

- Merged on GitHub: #124, #126, #128, #130, #132, #134, #136, #139, #140, #141.
- Head commit already an ancestor of main (landed via integration k): #125, #127, #129 (also #119, #121).
- Superseded / contained: #142 (in #153), #143 (in #152), #146 (in #157), #123 and #117 (in #135).

## Baseline on main `bffd168a` (before any merge)

- HW1: 3 pages, 0 errors, 0 overfull. HW2: 3 pages, 0 errors, 0 overfull.
- Words with dx > 0.1pt vs pdflatex (PR #171 `measure.sh` method): HW1 p1/p2/p3 4/2/12 (of 224/174/144), HW2 11/48/41 (of 176/255/96).
- FT-065 perf_bench (load ~3.9): HW1 full p50 2.86 ms, type-paragraph 1.30 ms, type-inline-math 1.13 ms; 500KB full 2484 ms, type-paragraph 124 ms.

## Stage 1 — compiler and library crates

### #131 hyperref-coverage (compiler) @ 3b131b88

- Textual conflicts: `src/lib.rs` (`pub mod expansion` vs `pub mod hyperref`: both kept); `src/parser.rs` (`Parsed.expansions` vs `Parsed.hyperref` field and initialiser: both kept; `use_package`'s `fontenc` block vs `hyperref` block: both kept in sequence); generated `supported/coverage.md`, `supported/supported-latex.json`, `docs/user/compiler.md` (took main's side; regenerated once at the end of stage 1).
- Build breaks against the post-expansion parser (as the Commander reported): `parser/hyperlinks.rs` read `self.macros` (removed: definitions run in `crate::expansion`) and spliced `self.t` (now `Rc<Vec<ExpandedToken>>`, possibly lent from the expansion cache with an index-based undo log).
  - `requeue_group`: rewinds the cursor to the braced group and turns arguments consumed after it into `Comment` tokens through `token_mut` (length-preserving, undo-logged) instead of splicing.
  - `defined_names` (`\autoref` names): scans the documents' sources for argument-free `\renewcommand`/`\newcommand`/`\providecommand{\...name}{..}` and `\def\...name{..}`, last definition winning.
- Checks: `cargo test --release --lib --test hyperref_oracle`: lib 258 passed; hyperref oracle 2 passed (all cases).

### #137 box-commands (compiler) @ 54ff53d8

- Textual conflicts: `src/incremental.rs` span arms (main's `Logo`/`Rule`/`Kern` vs `Box`/`SetLength`/`LengthGlue`: both); `src/layout.rs` Core 14 arms (same variants: both, with the brace between them restored); `src/parser.rs` (`mod hyperlinks` vs `mod boxes`, #131's hyperref fields vs `saved_boxes`/`lengths`, `\hypersetup` vs `\newsavebox`/`\newlength` preamble dispatch: all both); `src/supported.rs` (`thebibliography` formatting + new `minipage`/`lrbox`); generated inventory (took main).
- Build breaks against the post-expansion parser (as the Commander reported), fixed in `parser/boxes.rs` / `parser.rs`:
  - `box_blocks` now wraps the nested token list in `Rc`, parks the outer undo log and cache-lend flag for the nested parse, and no longer pushes/pops `macro_scopes` (removed; macro scoping is the expansion pass's). Unclosed braces are truncated like the style/alignment stacks.
  - The optional-argument remainder word is written through `token_mut` instead of indexing the `Rc` stream.
  - `long_required_group` added over main's existing `required_group_bounded(.., true)` (text copied verbatim from #153, so that merge sees an identical addition).
- Semantic overlap with main's expansion pass (#130): tex-expansion implements `\newlength`/`\settowidth`/`\settoheight`/`\settodepth` as engine primitives, so `box_commands` failed with engine diagnostics ("Missing number, treated as zero.", "Illegal unit of measure") and `supported_latex` reported `duplicate Text newlength`. Resolution: `HOST_PRELUDE` `\let`s the four to `\flashtexundefined` (as it already does for `\setlength`), so the parser's structured `Inline::SetLength`/`LengthGlue` (consumed by #138) handle them; the four engine-side inventory entries are removed. No compiler test or HW/corpus fixture used the engine versions.
- Checks: `cargo test --release --no-fail-fast` in crates/compiler: all binaries pass except `supported_latex::generated_artifacts_are_current` (stale generated docs, regenerated at the end of stage 1).

### #153 footnotes-2-compiler (compiler, includes #142) @ 3f227bba

- Textual conflicts, all in `src/parser.rs`:
  - Float caption (hunk 0): kept main's `table`/`figure` counters and `caption_hyperref`; the printed number now comes from #153's `within_chapter` (`Figure 1.1:` in report/book).
  - Hunk 1: #153's side re-added four pre-expansion macro functions (`declare_math_operator`, `define_macro`, `expand_macro`, `expand_macro_word`) that main removed with the expansion pass. Dropped them; kept #153's new `chapter` and `within_chapter`. `chapter` also resets `table_counter` (report.cls `\@addtoreset{table}{chapter}`; #153 predates that counter).
  - Equation counter and label (hunks 2-3): kept main's `set_current_anchor`/`label_inline`, with `within_chapter` numbers; the anchor is `equation.<chapter>.<n>` in chapter classes.
  - `long_required_group` was already added verbatim during #137, so there is no duplicate.
- Interaction with #137: `minipage` is now handled by `box_environment` (an `Inline::Box`), so #153's `\begin{minipage}` path (mpfootnote reset, `env_stack` lookup) was never reached (`minipage_footnotes_use_alph_mpfootnote…` got `["1","6"]`).
  - Fix: `box_environment` resets `mpfootnote_counter` and pushes the environment on `env_stack` around the nested parse.
  - `tests/footnote_counters.rs`'s `notes` helper now descends into `Inline::Box` content.
- Checks: `cargo test --release --no-fail-fast` in crates/compiler: all pass except `supported_latex::generated_artifacts_are_current` (stale generated docs; regenerated at the end of stage 1).

### #144 verbatim-fidelity-compiler (compiler) @ 439dc1e6

- Textual conflicts, `src/parser.rs`:
  - Command dispatch: #144's listings configuration arms (`\lstset`, `\lstdefinestyle`, `\lstloadlanguages`, `\lstinputlisting`) vs main's kernel text-symbol/logo/kern/`\rule` arms: both kept.
  - `inlines_from_tokens`: kept #144's `Verb` arm (adds latex.ltx's "`\verb` illegal in argument" diagnostic), followed by main's text-symbol and logo arms.
  - Generated `docs/user/compiler.md`: took main.
- Break against the expansion pass (as the Commander reported): `expansion::prepare`'s verbatim gate only knew `\verb`.
  - A document using only `\lstinline` skipped blanking entirely.
  - Blanking hard-coded `\verb`'s 5-byte name, so the engine would have read `\lsti`.
  - Fix: the gate also checks `\lstinline`; blanking starts after the actual control-word length; the converter maps `\lstinline` back to its lexer `Verb` token, as for `\verb`.
- Checks: `cargo test --release --no-fail-fast` in crates/compiler: all pass except `supported_latex::generated_artifacts_are_current` (regenerated at the end of stage 1).

### #147 list-structure-compiler (compiler) @ a86fad2e

- Textual conflicts, `src/parser.rs`:
  - Modules: `hyperlinks` (#131), `boxes` (#137), `lists` (#147): all kept.
  - `\item` dispatch: took #147's `item_label_argument` + `begin_item`, which replaces main's tuple-based marker code.
  - `\begin{itemize|enumerate|description}`: took #147's `open_list`; kept main's (#131) `list_label_state` push, whose pop at `\end` merged cleanly.
  - Generated inventory files: took main.
- Semantic fixes:
  - #131's hyperref hook moved into `begin_item`. Only counted enumerate items call `enumerate_item_hyperref` (`\refstepcounter{enum<i>}`, `Item.<n>` anchor); `\item[..]` does not. The templated reference text is used when enumitem `label=`/`label*=`/shortlabels are in force.
  - `enumerate_item_hyperref` reads `OpenList::counter` (#147's `\c@enum<i>`, which honours `start=`/`resume`) instead of destructuring the old `list_stack` tuple.
  - #137's `box_paragraphs` matches `Block::Styled { style, content, .. }`, since #147 added `lists`/`line_break_before`.
- Not done here (out of scope): #147's PR body notes CI's `apps/mac/scripts/sync-supported-latex.sh` step. The `apps/mac` supported-latex sync is needed when this lands.
- Checks: `cargo test --release --no-fail-fast` in crates/compiler: all pass (including `hyperref_oracle` and `list_structure`) except `supported_latex::generated_artifacts_are_current` (regenerated at the end of stage 1).

### #149 theorem-fidelity-compiler (compiler) @ ce7797fd

- Textual conflicts in `src/parser.rs`:
  - Imports: main's `text_builtins` + #149's extended `theorems` import.
  - `Parsed` fields and initialisers: `expansions`, `hyperref`, `theorems`, `qed_marks`, all kept.
  - `\chapter` tail vs #149-side pre-expansion macro code (misaligned by git): took main.
  - `\end{figure|table}`: main's flush, then #149's separate `close_theorem` branch.
  - `begin_theorem` vs `new_theorem_style` (misaligned): took #149's rewrite (`TheoremRecord`s, `counters.step`), with #131's hyperref anchor re-added (`<counter>.<number>`).
  - `thebibliography` `widest_label`: main's `OpenList` version.
- Textual conflicts in `src/supported.rs`: kept the listings (#144) and theorem (#149) commands; main's environment descriptions with #149's `proof` text; main's amssymb check.
- Post-expansion breaks:
  - `begin_proof` read `\proofname` from `self.macros`, and `qed_symbol` checked it for `\qedsymbol`. Both now use `P::user_definitions()`, the source scan from #131's `defined_names`, generalised to all names; `defined_names` filters it.
  - `renewed_proofname_is_the_default_heading` got "LaTeX Error: Command \proofname undefined." from the engine. amsthm is never loaded there, and #149's parser-side `amsthm_default` went with the macro table. `HOST_PRELUDE` now defines amsthm's `\proofname` (`Proof`) and `\qedsymbol` (U+220E, the glyph the parser paints) for every document.
- Checks: `cargo test --release --no-fail-fast` in crates/compiler: all pass (`amsthm` included) except `supported_latex::generated_artifacts_are_current`.

### #150 xcolor-support (compiler) @ dc79f2e7

- Textual conflicts:
  - `src/lib.rs`: kept `boxes` alongside `color` and `color_names`.
  - `src/parser.rs` modules: kept `hyperlinks`/`boxes`/`lists` plus `colors`.
  - `Inline` enum: interleaved, because git shared one `}` between different items. The result is #137's `Box`/`SetLength`/`LengthGlue` variants, then #150's `ColorBox` variant, then main's `ReferenceForm` enum, then #150's `ColorBox` struct.
  - `src/incremental.rs` span shifting: kept both arms.
- Integration fixes (no pre-expansion API use in #150's new `parser/colors.rs`/`color.rs`):
  - #150's `box_inlines` (one argument) collided with #137's `box_inlines` (three arguments). Renamed #150's to `color_box_inlines`.
  - #149's `theorems.rs` `ITALIC` const lacked #150's new `TextStyle::color` field; it now sets `color: None`.
  - `xcolor_parse::colorbox_and_fcolorbox_capture_fill_frame_and_fboxsep` got `fboxsep_pt` 3.0 instead of 5.0. #137's `set_box_length` claims `\setlength{\fboxsep}` (it emits `Inline::SetLength` for the typesetter) and returns before #150's `fboxsep_pt` arm. `set_length` now records `\fboxsep`/`\fboxrule` before delegating, so both consumers see the value.
- Checks: `cargo test --release --no-fail-fast` in crates/compiler: all pass except `supported_latex::generated_artifacts_are_current`.

### #155 equation-numbering (compiler) @ 7cec0c3f

- Semantic overlap with #153. #153 numbered equations, figures and tables per chapter with hand-kept `u32` counters and `within_chapter`. #155 moves `equation`/`figure`/`table` onto `xref::Counters` (`step`/`the`, `define_body_counters`, `\numberwithin`/`\counterwithin`/`\counterwithout`, `subequations`). Resolved to #155's single counter model:
  - `Counters::report()` (from #153) calls `define_body_counters()` and puts `equation`/`figure`/`table` within `chapter`, so `\chapter`'s `counters.step("chapter")` resets them, as report.cls `\@addtoreset` does.
  - Removed the `equation_counter`/`figure_counter`/`table_counter` fields, their resets in `chapter()`, and `within_chapter`.
  - Known difference: a report/book equation before the first `\chapter` prints `0.1` where #153 printed `1`. `\ifnum\c@chapter>\z@` is not modelled by `xref::Piece`; no test or fixture has one.
- Textual conflicts:
  - `src/expansion.rs` `HOST_PRELUDE`: amsthm defaults (#149) plus the `\counterwithin`/`\counterwithout` pass-through.
  - `src/parser.rs`:
    - fields: `subequations` added;
    - dispatch: `\hypersetup`/boxes plus the counter commands;
    - float captions: `Counters` numbering with #131's `caption_hyperref`;
    - `thebibliography`: `OpenList`, then the `subequations` begin/end branches (kept after main's NoHyper/`list_label_state` code);
    - `equation`: `Counters` plus #131's `equation.<n>` anchor;
    - multirow: #155;
    - labels: `label_inline` with a `Counters` fallback.
  - `src/supported.rs`: counter commands plus main's hyperref `url`/`href` text.
  - `src/xref.rs`: `report()` plus `define_body_counters()`.
  - Generated files: took main.
- Further integration fixes:
  - `Counters::set_within` (#149) wrote `reset_by = Some(..)` and `.prefixed`. It is rewritten on #155's representation (`reset_by: Vec`, `the` pieces), with the parent-chain cycle check kept.
  - #149's `"numberwithin" => self.number_within(span)` arm shadowed #155's `counter_numbering`, which handles `[style]`. Removed #149's arm and function.
  - #155's special case for theorem counters (setting `TheoremDef.within_section`, a field #149 removed) was deleted; theorem counters are `Counters` entries since #149.
  - `supported_latex` reported `duplicate Text numberwithin`: #155 added it to `BUILT_INS`, and #149 listed it in `TEXT_EXTRA_ARMS`. Removed it from `TEXT_EXTRA_ARMS` and #149's described entry; #155's entry is kept.
  - Removed the unused `HashMap` import left in `parser/boxes.rs` (#137 fix).
- Checks: `cargo test --release --no-fail-fast` in crates/compiler: all pass (`equation_numbering` 3/3, `footnote_counters` 7/7, `amsthm`, `hyperref_oracle`, `xcolor_parse`, `box_commands`) except `supported_latex::generated_artifacts_are_current`.

### #159 algorithmic-compiler (compiler) @ 1c78cd73

- Textual conflicts in `src/parser.rs`:
  - modules: `mod algorithmic` added;
  - initialisers and fields: main's `subequations` plus #159's `algorithm_counter`; #159's `equation_counter`/`figure_counter` were dropped, since #155 moved those to `xref::Counters`;
  - preamble dispatch: hyperref/boxes/counter commands plus `\algnewcommand`/`\algrenewcommand`/`\algsetup`/`\floatname`;
  - `\caption`: #159's algorithm branch first, then main's figure/table check;
  - float `\begin`/`\end`: `figure | table | algorithm | algorithm*`. #159's `|| self.theorems.contains_key(..)` in `\end` was dropped, because main closes theorems in the following branch (#149);
  - package matches: `listings` plus the algorithm packages.
- Textual conflicts in `src/supported.rs`: `TEXT_EXTRA_ARMS` keeps both the theorem and the algorithm arms; #159's `\caption` text and algorithm commands, plus main's `\item [label]` entry (#147).
- Generated files: took main.
- Integration fix: `parser/algorithmic.rs` built `Block::Styled` without #147's `lists`/`line_break_before`; it now sets `lists: self.list_frames.clone()` and `line_break_before: None`. #159's `expansion.rs` change (keyword macros declared as host commands) merged cleanly, and #159's new files use no pre-expansion APIs.
- Checks: `cargo test --release --no-fail-fast` in crates/compiler: all pass except `supported_latex::generated_artifacts_are_current`.

### #163 siunitx (compiler) @ c93e58ca

- Textual conflicts:
  - `scripts/canonical_latex.py` and `supported/canonical-latex.tsv`: both new canonical sets kept, `xcolor` (main, #150) then `siunitx`; source-header lines and rows in the same order.
  - `src/supported.rs`:
    - text commands: main's listings and theorem commands, plus the siunitx commands;
    - packages: git shared one tuple's `(`, so the result is main's `hyperref` tuple, then #163's `siunitx` tuple, then main's `PSEUDOCODE_PACKAGES` table (#159);
    - `CANONICAL_SETS`: `... "tikz", "xcolor", "siunitx"`.
  - `src/parser.rs`:
    - `BUILT_INS`: both.
    - Preamble dispatch: main's arms; #163's `\sisetup`/`\DeclareSIUnit` before the preamble guard; siunitx commands after `\chapter`.
    - `\usepackage`: #163 reads raw options with braces kept, while main keeps `options_span`. After main's hyperref option block comes #163's `siunitx::load_package`.
    - `inlines_from_tokens`: #150's colour arm and #163's siunitx arm share one skip index (`skip_to`).
    - Helpers: main's `token_source` plus #163's `siunitx_bracket_at`/`siunitx_group_at`.
    - Package matches: both.
  - Generated inventory files: took main.
- Integration fix: #163's two `Inline::Math` initialisers (siunitx in running text and in style/heading arguments) lacked #150's `color`/`color_ranges`. They now take the running text colour and no inner colour ranges; the formula is built from argument text, not source math tokens. #163's `siunitx.rs` uses no pre-expansion APIs.
- Checks: `cargo test --release --no-fail-fast` in crates/compiler: all pass except `supported_latex::generated_artifacts_are_current`.

### #164 multicol-compiler (compiler) @ ff034588

- Textual conflicts in `src/parser.rs`:
  - Initialisers: main's hyperref/box fields plus `twocolumn_option`.
  - Fields: #164's `twocolumn_option` placed inside `struct P`, before its closing brace. Git had shared that brace with main's `OpenList` struct.
  - `\usepackage`: main's colour-package loop (#150), then #164's multicol twocolumn warning.
  - `\begin`/`\end`: main's bibliography `OpenList`, subequations and float branches, plus #164's `multicols`/`multicols*` branches. #164's stale tuple-style bibliography push was dropped.
  - Package matches: both.
- Textual conflicts in `src/supported.rs`:
  - Environments: `minipage`/`lrbox` plus `multicols`/`multicols*`.
  - Packages: the `multicol` tuple after `siunitx`, before `PSEUDOCODE_PACKAGES`.
- Generated files: took main.
- Checks: `cargo test --release --no-fail-fast` in crates/compiler: all pass except `supported_latex::generated_artifacts_are_current`.

### #166 citations-bibliography (compiler) @ 40b4070f

Merged on nixos-pc-kabir (Linux, TeX Live 2025) after the session handoff.

- Textual conflicts in `src/bib.rs`: main's `resolve` read `BibItem.label`, a
  field #166 replaces with `kernel_label`/`natbib_num`. Took #166's side (the
  struct definition itself merged cleanly to #166's, so main's accessor no
  longer compiled).
- Textual conflicts in `src/parser.rs`:
  - `Parsed` fields and initialisers: main's `hyperref`/`theorems`/`qed_marks`
    plus #166's `citations`/`cite_style`/`cite_numbers`; both kept.
  - Preamble dispatch: main's hyperref/box/counter/pseudocode/siunitx arms plus
    #166's `\citestyle`/`\bibpunct`/`\setcitestyle`. Git had shared the brace
    closing `\DeclareSIUnit` with the one closing `\setcitestyle`; the missing
    brace was restored, or #166's arms would have nested inside `\DeclareSIUnit`.
  - `\usepackage`: main's colour-package loop and multicol twocolumn warning
    plus #166's natbib option block. The same shared-brace misalignment; the
    brace closing the multicol `if` was restored.
  - Supported-package list: main's listings/pseudocode/siunitx/multicol arms
    plus #166's `natbib`; both kept.
  - Generated `supported/supported-latex.json` and `docs/user/compiler.md`:
    took main's side (regenerated once at the end of stage 1).
- Semantic overlap, `\cite` (hunk 3, badly misaligned): git aligned main's old
  inline `\cite` body against #166's `\bibliographystyle` arm. #166 routes every
  citation command through `P::citation` → `bib::Citer::cite`, so main's inline
  body is dead; took #166's `\bibliographystyle` text. That dropped #131's
  hyperref integration, which is re-added inside `citation()`: each key that
  `bibliography.resolve` finds records a `LinkKind::Cite` link to `cite.<key>`.
  Verified by `hyperref_oracle` case `29-cite-links`.
- Semantic overlap, `\bibitem` (hunk 4): main applied `bib::label_bracket` to the
  label and set #147's `pending_item` template; #166's `Bibliography::list_label`
  already brackets (and returns an empty string for a natbib author-year list),
  so main's call would have double-bracketed. Kept #166's label and re-added
  #147's `pending_item = ItemLabel::Template { text: label.clone() }`, which is
  what paints the marker.
- Checks: `cargo test --release --no-fail-fast` in crates/compiler: all pass
  (`hyperref_oracle`, `list_structure`, `footnote_counters`, `amsthm` included)
  except `supported_latex::generated_artifacts_are_current` (stale generated
  docs; regenerated at the end of stage 1).

### #169 inline-graphics-compiler (compiler) @ 5e425dcf

- Textual conflicts:
  - `src/lib.rs`: `pub mod graphics` before `pub mod hyperref` (alphabetical).
  - `src/incremental.rs`: the span-shifting arms (main's `Box`/`SetLength`/
    `LengthGlue`/`ColorBox` vs `Graphic`/`Transform`) and the `span_of` arms;
    both kept. Git had shared the brace closing the `ColorBox` arm, which was
    restored.
  - `src/layout.rs`: `visit_inline_references` (main's `Box` arm plus #169's
    `Transform` arm) and the Core 14 emitter (main's `ColorBox` plus #169's
    `Graphic`/`Transform` "this layout draws no images" diagnostics); both kept.
  - `src/parser.rs`:
    - `Inline` enum: git shared one brace between the end of the enum and the
      end of #150's `ColorBox` struct, so #169's variants were placed first and
      main's `Box`/`SetLength`/`LengthGlue`/`ColorBox` variants after them; the
      shared brace then closes `ColorBox` as before.
    - Preamble dispatch: main's arms plus #169's `\graphicspath`; both kept.
    - Helper functions: git aligned the tail of #163's `siunitx_group_at`
      against #169's new `bracket_inner` (both end in the same
      `_ => {} } } None }` shape). The tail was duplicated so each function
      keeps its own.
  - Generated `supported/coverage.md`, `supported/supported-latex.json` and
    `docs/user/compiler.md`: took main's side (regenerated at the end of stage 1).
- Main's `\includegraphics` stub (an "unsupported" diagnostic) was replaced by
  #169's `include_graphics` without conflict; only one dispatch arm remains.
  #169's new `graphics.rs` uses no pre-expansion parser APIs.
- Checks: `cargo test --release --no-fail-fast` in crates/compiler: all pass
  except `supported_latex::generated_artifacts_are_current`.

## PAUSED 2026-09-13 (session handoff)

Stage 1 is partly done; see the draft PR description for resume notes.
