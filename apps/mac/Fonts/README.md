# Bundled fonts

Latin Modern (OpenType CFF) from CTAN `fonts/lm` (Latin Modern 2.005 / LM Math
1.959), redistributed under the GUST Font License v1.0 (`GUST-FONT-LICENSE.TXT`).
These are the LaTeX default faces (Computer Modern design); the preview, the
CoreGraphics export, and `flashtex-pdf --default-face lm` use them so the app does
not depend on a TeX installation. `scripts/make-app.sh` copies this directory to
`Contents/Resources/Fonts`; `PreviewFonts` registers it (`FLASHTEX_LM_DIR` overrides).
Included: lmroman 7/10/12/17 masters (regular; 10/12 also bold/italic/bolditalic)
and `latinmodern-math.otf` (LM Math 1.959, 733,736 bytes, sha256
6075562b771f8b82f0c179e363389684f2dd09de30038269e2628e504bd7be0f — the file
MacTeX 2026 ships at texmf-dist/fonts/opentype/public/lm-math/). Nothing else in
the repository embeds proprietary fonts.

## Rooted TeX metrics (`texmf/`)

`texmf/fonts/tfm/public/lm/{ec-lmr10,ec-lmr12,rm-lmr12,rm-lmr8,rm-lmr6}.tfm` and
`texmf/doc/fonts/lm/GUST-FONT-LICENSE.TXT` are the official Latin Modern 2.004
metrics (GUST `lm2.004bas.zip`, archive sha256
97a725ea012d41367bf44fec1a2f4ccf4fe134c016715522133594e347115a7c) that
`flashtex-render` needs for the 10 pt / 12 pt regular text and 12 pt roman-math
fixtures, laid out exactly as a `texmf-dist` root so the producer can derive the
root and the license. They were copied byte-for-byte from MacTeX 2026
(`/usr/local/texlive/2026/texmf-dist`) only after their SHA-256 matched the
pinned manifest in `crates/rendering-core/docs/handoffs/native-assets/manifest.json`
(ec-lmr10 cd13479f…, ec-lmr12 29902112…, rm-lmr12 9d4e3d8e…, rm-lmr6 eb0bfdf8…,
rm-lmr8 80bcbfd8…, license 49ea6cb9…); `scripts/bundle-texmf.py check Fonts/texmf`
re-verifies them and `make-app.sh` refuses to package on any mismatch. Bold,
italic and the other design sizes have no metrics here yet.

`texmf/SUPPLEMENTARY-METRICS.json` pins 23 further text TFMs in the same
directory (`ec-lmr{5,6,7,8,9,17}`, `ec-lmbx{5,6,7,8,9,10,12}`,
`ec-lmri{7,8,9,10,12}`, `ec-lmbxi10`, `rm-lmr{5,7,9,10}`) for the other design
sizes and the bold/italic faces. They are not in the Commander's manifest: copied
from the same MacTeX 2026 tree (TeX Live `lm` rev 77682, catalogue 2.005,
MANIFEST 2.004) and byte-identical to the CTAN `lm.zip` copy on the build
machine, but not verified against the pinned 2.004 archive hash; see the
`provenance` block in that file. `make-app.sh` refuses packaging if any of them
drifts from the pinned hash.
