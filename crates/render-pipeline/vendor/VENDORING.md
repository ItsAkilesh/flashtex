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
| `compiler` | `agent/kabir-claude/compiler-perf` | `59074193` (FT-065: `origin/main` `6d2f5628` crates/compiler plus Daniel's compile-latency squash — in-place incremental block shifting, ASCII fast paths in `export`/`protocol`, `json` integer/hundredths/unescaped-string writers; also carries main's `7a695a79` LM-math export change. The pipeline uses `parser`, `json` and the protocol envelopes; `perf_bench` reply digests are identical to the `42557b09` pin on HW1/HW2/500 KB. Was `42557b09`, which was `b38e1884`: every `\item` paragraph is now `Block::ListItem { level, label, content, extra_gap_before_pt, extra_gap_after_pt }` (`1f9bd33c`/`6c07a4f1` list-indent: the compiler's *own* layout hangs items at `\leftmargin<i>` sums and draws the label; the pipeline takes `level`/`label` and sets the geometry itself — `ListGeom` in `adapter.rs`: article's `\leftmargin<i>` or enumitem `leftmargin=*`/`<dimen>` read from `\setlist`/`\begin{..}[..]`, the label box `\hskip-\labelwidth-\labelsep \hbox to\labelwidth{\hfil label}\hskip\labelsep` on the first line, `\parskip=\parsep` for item paragraphs); the compiler's `extra_gap_*` are **not** applied: they land on whichever paragraph the next `\item`/`\end` flushes (wrong block when an item holds a display) and evaluate `em` at 12pt, so the pipeline derives `\topsep`/`\itemsep`/`\partopsep` (class `\@list<i>` + `\setlist` overrides at the class size) with `\addvspace` semantics (`Block::Paragraph::addvspace_before`). `Inline::Text`/`Inline::Math` gained `space_before` (`5a18d274`: no invented inter-word space at math/macro-splice boundaries; the lexer now swallows whitespace after a control word) — the pipeline reads interword gaps from the source bytes (`gap_has_space`, which already skips post-control-word blanks) and ignores the flag, so nothing is doubled or lost; page-1 word x unchanged. The `font-engine` pin below still satisfies it) | compiler lead |
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
