# Vendored sibling crates (pinned mirrors)

These directories are byte-for-byte copies (`git archive <sha>:crates/<name>`)
of sibling crates at the revisions the pipeline is built against. Most are
task-branch tips that are **ahead of `main`**; they exist only so
`crates/render-pipeline` builds standalone from a `git archive` of this
directory alone (which is how the visual-oracle harness builds it). They are
read-only here: no edits, no fixes; requested API changes go to the owning
agent through `docs/proposals/rendering-abi.md` and are listed in the
render-pipeline README. Each directory carries a `PIN` file with the full
commit SHA it was exported from. Directory names are the plain crate names
because the siblings depend on each other by `../<name>` path.

| Directory | Branch | Commit | Owner |
| --- | --- | --- | --- |
| `compiler` | `main` | `3d3d5ae3` (was `87df3e4a`: +preamble `\setlength{\parskip}`/`\parindent` and the class body size read into `Parsed::parskip_pt`/`class_size_pt` (`4b11df28`), `\tiny..\Huge` as no-ops (`738714a9`), text-mode `\quad`/`\qquad` as `Inline::TextGlue` and `\bigskip`/`\medskip`/`\smallskip` as `Block::VSpace`, `\hfill`/`\hfil` emitted inside titles and macro bodies (`Inline::HFill`), single math letters hinted Times-Italic in the compiler's own v1 layout (`391de812`; the pipeline sets math from Latin Modern Math and ignores that hint). The pipeline maps `TextGlue` to `Item::Quad` and no longer scans heading gaps for `\hfill`; it keeps its own `\parskip`/`\parindent`/class-size source scan because those feed the pipeline's typesetter, not the compiler's `LayoutConstraints`, and the pipeline's `em` is the class `\normalsize` (10.95pt at 11pt) where the compiler's is the integer option. The `font-engine` pin below still satisfies it) | compiler lead |
| `font-resources` | `main` | `d5440b0` (crates/font-resources last changed by `5c89501`; shared TFM reader `tfm.rs`/`tfm_run.rs`, `required_tfm.rs`) | commander-corpus |
| `project-files` | `main` | `d5440b0` (last changed by `d92db37`; `ProjectRoot` for the rooted TFM reads) | project-files owner |
| `font-engine` | `agent/mac-font-engine/tex-fonts` | `f418238` (main's only later font-engine change, `1ff6abc0` Core14 Symbol U+2223 `afm_char` mapping, adds no API the `79986817` compiler pin needs; not carried here) | mac-font-engine |
| `paragraph-layout` | `agent/mac-paragraph-layout/linebreak` | `70209e2` | mac-paragraph-layout |
| `math-layout` | `agent/mac-math-layout/math-boxes` | `db90047` | mac-math-layout |
| `pdf` | `agent/mac-pdf/pdf-output` | `4bd8c2e` | mac-pdf |
| `document-style` | `agent/mac-document-style/style-model` (= main) | `bfc980d` | mac-document-style |

`font-engine` enables its `paragraph`/`math`/`pdf` adapter features by
default, which is why those three siblings must be present under their plain
names next to it.

`font-resources` brings `serde`, `serde_json` and `sha2` from the registry
(the first external crates in this build); everything else is path-only.

**Delete this directory and point `Cargo.toml` path dependencies at
`../<name>` once the siblings are integrated on `main`** (`main` currently
carries older revisions of font-engine, paragraph-layout, math-layout and
pdf than the pins above). Nothing binary is vendored; Latin Modern's OTFs
and TFMs are read from the local TeX Live installation at run time, never
committed; the 12 pt TFM set is digest-bound to the official Latin Modern
2.004 release (`REQUIRED_TFMS` in `src/fonts.rs`).
