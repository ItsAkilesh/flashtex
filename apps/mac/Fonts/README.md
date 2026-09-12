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
