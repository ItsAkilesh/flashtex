# Vendored sibling crates (pinned mirrors)

These directories are byte-for-byte copies (`git archive <sha> crates/<name>`)
of sibling task branches that are **not yet merged to `main`**. They exist only
so `crates/render-pipeline` builds standalone on its own branch. They are
read-only here: no edits, no fixes; requested API changes go to the owning
agent through `docs/proposals/rendering-abi.md` and are listed in the
render-pipeline README.

| Directory | Branch | Commit | Owner |
| --- | --- | --- | --- |
| `font-engine@d0c64cf` | `agent/mac-font-engine/tex-fonts` | `d0c64cf` | mac-font-engine |
| `paragraph-layout@7af5c05` | `agent/mac-paragraph-layout/linebreak` | `7af5c05` | mac-paragraph-layout |
| `math-layout@2b7c7d5` | `agent/mac-math-layout/math-boxes` | `2b7c7d5` | mac-math-layout |
| `document-style@bfc980d` | `agent/mac-document-style/style-model` | `bfc980d` | mac-document-style |
| `pdf@52b3711` | `agent/mac-pdf/pdf-output` | `52b3711` | mac-pdf |

The assignment named font-engine `c25b516`; `d0c64cf` is that branch's tip at
vendoring time (adds GPOS MarkToBase, no API removals) and is what the
pipeline is built against.

**Delete this directory and point `Cargo.toml` path dependencies at
`../<name>` once the siblings are integrated on `main`.** Each pinned copy
keeps its own README with provenance and licence notes. Nothing binary is
vendored; Latin Modern is read from the local TeX Live installation at run
time, never committed.
