# flashtex-tex-align: design and resume notes (KC-111, WIP, paused)

Status: **design only, no code yet.** Nothing here is verified against pdflatex.

## Overlap check (origin/main e1dddc47)

- `crates/compiler/src/tabular.rs` + `parser/tabular.rs` (Daniel, merged via
  `agent/daniel-parent/tabular`) is a command-level `tabular`/`tabular*` layout
  in f64 points: kernel `\@mkpream` column templates (`l c r p{} | @{}`),
  `\tabcolsep`/`\arrayrulewidth`/`\doublerulesep`, §801 span excess into the last
  spanned column, `\@arstrut` with `\arraystretch`, `\\[dim]`, `\hline`,
  `\cline`, booktabs rules, `[t]/[b]/[c]`, `tabular*` `\extracolsep{\fill}`.
- There is no `\halign`/`\valign` engine (no templates, `\omit`, `\span`,
  `\noalign`, `\tabskip` changes, periodic preambles, glue setting in sp).
- `render-pipeline/src/mathgrid.rs` places amsmath grid cells; not an engine.
- FT-030 (daniel-tables) owns `crates/paragraph-layout`, not tables.

So KC-111 proceeds in this new crate without touching theirs.

## Design

Depends on `crates/tex-boxes` (PR #93, merged into this branch) by path.
`BoxEngine`'s nest and save stack are private, so the crate drives it from outside:

- `align.rs` (engine-independent): the preamble list, generic over the token type
  (`u`/`v` templates, tabskip glue per boundary, `cur_loop` for `&&`), unset
  cells/rows, §801–804 width merging (span maps per column, null_flag
  nullification of the following tabskip), §806–810 setting rows and cells with
  web2c `zround` and `f64` glue ratios, and prototype packing via tex-boxes
  `hpack`/`vpackage`. Report text is post-processed so prototype children read
  `\unsetbox(...)`, as in §184.
- `interp.rs`: a small TeX machine used by the fixtures and as a reference driver.
  It has an input stack (file, backed-up, macro, u/v template, `\everycr`,
  `\everypar`) and `align_state` in `get_next`: §342 inserts the v template,
  §324 sets `align_state:=0` at the end of the u template, and §442 undoes brace
  counting in backtick constants. It also has `get_x_token` with `\endtemplate`
  becoming `endv`, and `scan_int`/`scan_dimen`/`scan_glue` with expansion and
  em/ex from cmr10 params. Macros and toks have their own save stack kept in
  lockstep with the engine's groups.
  Mapping onto `BoxEngine`:
  - Alignment group: `begin_group`.
  - Alignment list: `begin_box(VBox|HBox, SetBox scratch)`, with `prev_depth` inherited.
  - Each cell: `begin_box(HBox|VBox)`. At `fin_col`, `take_current_list` then `end_box`.
  - Rows: `append_to_vlist` placeholders, replaced at `fin_align`.
  - Result: put into a register and spliced with `unpackage`, then `aux` is restored.
  - Paragraph starts: the interpreter calls `new_graf` itself so it can push `\everypar`.
- `latex.rs`: Rust expanders that emit primitive token lists exactly as the
  macros expand. They cover:
  - Kernel `\@mkpream` classes 0–5 (latex.ltx 16632–16713) and `\@array`
    (16564), `\@tabularcr`/`\@arraycr` (16583–16602), `\multicolumn` (16603),
    `\hline`/`\cline`/`\vline` (16728–16748), `\@startpbox`/`\@endpbox` (16755).
  - The array.sty `\@testpach`/`\@classx`/`\@classz`/`\save@decl`/`\@classv..viii`,
    `insert@column` with toks slots, `\extrarowheight`, `m`/`b`/`p`, `\@finalstrut`
    and `\ar@align@mcell`.
  - booktabs `\toprule`/`\midrule`/`\bottomrule`/`\cmidrule`/`\addlinespace`,
    with the `\@lastruleclass` logic.
  - Brace tricks are modelled as `align_state` ±1 while scanning `*`/`[...]`.
- Oracle: `tests/oracle/generate.py` writes three pdflatex jobs: raw
  `\halign`/`\valign` in an article document, kernel tabular, and array+booktabs.
  Each job uses `\showbox`, `\wd`/`\ht`/`\dp`, and `\pdfsavepos` POS markers, as in
  tex-boxes, plus PARAM lines (cmr10 space/quad/xheight and booktabs
  dimens). Cells use rules/boxes/kerns and spaces (font glue), no characters.

## Resume steps

1. `git fetch origin && git checkout agent/kabir-claude/tex-align` (worktree-isolated).
2. Merge a newer `origin/agent/kabir-claude/tex-boxes-r2` if one exists.
3. Write `src/align.rs` first and unit-test §801–810 against tex.web examples.
4. Write `src/interp.rs` and the raw `\halign`/`\valign` fixtures, then run the
   generator (MacTeX) and replay them with `cargo test` (`CARGO_BUILD_JOBS=3`).
5. Add the kernel tabular layer, then array.sty, then booktabs; aim for ≥70
   fixtures in total.
6. Finish this file with the verified/believed status and the adoption plan for
   compiler, render-pipeline and tex-expansion (KC-101).

Reference: tex.web §§768–812 (TeX Live mirror `texk/web2c/tex.web`, part 37),
`latex.ltx` 16550–16758, `array.sty` v2.6n, `booktabs.sty` v1.61803398.
