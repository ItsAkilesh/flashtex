# Font route study (lane `daniel-font-route-study`)

Read-only research at `origin/main` c7ff3f74, 2026-09-12 ~19:45Z. No code,
font files, or pins were changed. Local TeX Live 2026 was used only to locate
files and read metrics/licences (no downloads).

## Partial \mathbb notes for daniel-parent-b (handed over, not a recommendation)

Scope moved to `daniel-parent-b` mid-study. These are findings, not a ruling.

**Correct the premise first.** There are two producers, and both share the
parser in `crates/compiler` (render-pipeline vendors it), which is why HW1's
11 `\mathbb is not supported in math mode` diagnostics are identical on both
(`fixtures/real-world/hw1/gap-report-2026-09-12.md`). The diagnostic comes from
`crates/compiler/src/math.rs` `command_atom` (fallback arm, ~line 341), not
from the export guard.

- `flashtex-compiler`: layout on font-engine Core14 AFM metrics
  (`crates/compiler/src/layout.rs` `face`/`math_font`), export through
  `crates/pdf` runtime-v1 with base-14 `Times-Roman` + `Symbol` that are **not
  embedded** (`crates/pdf/src/lib.rs` header). The guard
  `every_math_symbol_has_a_decided_export_outcome` iterates `COMMAND_GLYPHS`
  only; the separate compile-time warning in `crates/compiler/src/protocol.rs`
  (~line 644, `export::unrepresentable`) would fire for U+211D etc.
- `flashtex-render` (`crates/render-pipeline`): TFM metrics (`lmmi/lmsy/lmex`
  via math-layout `CmMathMetrics`, `rm-lmr*` read from TFM) and painting from
  `latinmodern-math.otf` by original GID (`src/mathtex.rs`
  `TexMathMetrics::glyph`/`otf_gid`); exact export via
  `flashtex-pdf-exact from-v2` (fonts matched by sha256, GID-preserving CFF
  subset in `crates/pdf/src/cff.rs`). `TexMathMetrics::glyph` returns `None`
  for any char with no cm slot, so a parsed U+211D would be a missing glyph.

**Reference fonts.** `fixtures/real-world/hw1/reference-mactex2026.pdf` embeds
`QSLEFF+MSBM10` (Type 1 subset) for the double-struck letters, CMMI/CMSY/CMEX/CMR
for math, and cm-super `SFRM1095`/`SFBX*`/`SFTI1095` for text.

**Facts per option.**

| | (a) AMS msbm10 Type 1 | (b/c) Latin Modern Math U+2115/2124/211A/211D | (b') STIX Two Math |
|---|---|---|---|
| Already in repo | No | **Yes**: `apps/mac/Fonts/latinmodern-math.otf` sha256 `6075562b…` = font-engine manifest `lm.math`; bundled by `make-app.sh` | No (TeX Live has it, 838 KB) |
| Glyphs present | N Z Q R in msbm10.pfb (34 694 B) | GIDs 3506/3518/3509/3510, plus U+1D538 block | GIDs 1230/1238/1233/1236 |
| Advance (em) N/Z/Q/R | 0.722/0.666/0.777/0.722 (msbm10.afm) | 0.722/0.667/**0.667/0.639** | 0.692/0.648/0.751/0.729 |
| Licence | SIL OFL 1.1, © AMS; `msbm5–10` are Reserved Font Names (`doc/fonts/amsfonts/OFL.txt` l.136–141). OFL-FAQ 1.1: embedded subsets in documents need no licence text | GUST Font License (already shipped with text `GUST-FONT-LICENSE.TXT`) | OFL 1.1 |
| font-engine support | **None** for Type 1 programs (README "Not implemented") | CFF OpenType load, cmap 12, MATH: yes; CFF subsetting no (engine), yes in `crates/pdf/src/cff.rs` | same as LM Math |
| crates/pdf support | `type1.rs` parse+subset, `exact.rs` `FontProgram::Type1` exist | runtime-v1 `embed.rs` embeds the whole CFF table (no subset; large); exact route subsets | same |
| Mac preview | Preview draws display-list glyphs by GID from bundled OTFs; a `.pfb` has no GIDs and would need a new resource kind (verify whether CoreText can still load Type 1 on current macOS; `crates/font-resources` has Type 1 outline decoding) | No new resource; face already loaded | new pinned file |
| Bundle (#36) | New pfb + `msbm5/7/10.tfm` into the pinned texmf tree, pins, OFL text; `make-app.sh` copies only pinned `*.otf` to `Fonts/` and refuses unpinned files | None (already bundled). Note `make-app.sh` on main already bundles a pinned texmf tree for GH36 though the issue is still open | new pin + OFL |

**Likely code paths** (not sized precisely): parser arm in
`crates/compiler/src/math.rs` `command_atom` (letter argument → Unicode
double-struck char; refuse non-letters) and re-vendoring into
`crates/render-pipeline/vendor/compiler`; render route
`crates/render-pipeline/src/mathtex.rs` `glyph`/`otf_gid` (a double-struck
family: metrics from msbm TFM for (a) or from LM Math hmtx + `src/cff.rs`
bounds for (b)); compiler route `crates/compiler/src/export.rs` (an embedded
outcome instead of Symbol-only), `layout.rs` `face` (a non-Core14 face), and
`crates/pdf` runtime-v1 embedding (whole-table CFF today).

**Falsifiers noticed.** (b) visual parity with the MSBM10 reference fails for Q
and R (0.110/0.083 em narrower, different design), which can move HW1 line
breaks; whether that is acceptable is the decision. (a) fails if the Mac preview
cannot draw a Type 1 program, or if the v2 display list's GID/sha256 font model
cannot carry a name-keyed Type 1 face without a schema change. For the compiler
route, any option fails the "no silent substitute" rule unless the protocol.rs
warning and export table gain a real embedded outcome; do not relax the guard.
