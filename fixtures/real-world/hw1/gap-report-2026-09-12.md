# HW1.tex — real-world gap report (2026-09-12)

User-provided target (mac-m1max-a): `HW1.tex` must render like `HW1-reference.pdf` (pdfLaTeX, 3 pages). The user's note: this is *a very small subset* of what FlashTeX must eventually support.

Producer: `flashtex-render` (agent/mac-render-pipeline/unified 9aaec57a, compiler pin 745f327); the direct `flashtex-compiler` (main) reports the identical set.

Status `recovered`, 3 pages laid out (reference: 3).

| count | diagnostic |
|---|---|
| 12 | `\in is not supported in math mode` |
| 11 | `\mathbb is not supported in math mode` |
| 10 | `\forall is not supported in math mode` |
| 7 | `\subsection requires a braced argument` |
| 7 | `\hfill is not supported by this compiler version; unrestricted TeX math mode is not implemented` |
| 7 | `\normalfont is not supported by this compiler version; unrestricted TeX math mode is not implemented` |
| 6 | `\exists is not supported in math mode` |
| 5 | `\text is not supported in math mode` |
| 5 | `\qquad is not supported in math mode` |
| 5 | `\bigl is not supported in math mode` |
| 5 | `\bigr is not supported in math mode` |
| 4 | `\mid is not supported in math mode` |
| 4 | `\vee is not supported in math mode` |
| 3 | `\Rightarrow is not supported in math mode` |
| 2 | `\setlength is not supported in the document preamble` |
| 2 | `\bfseries is not supported by this compiler version; unrestricted TeX math mode is not implemented` |
| 2 | `\quad is not supported in math mode` |
| 2 | `\setminus is not supported in math mode` |
| 1 | `packages fontenc are recognised but not implemented` |
| 1 | `packages inputenc are recognised but not implemented` |
| 1 | `packages geometry are recognised but not implemented` |
| 1 | `packages amsmath, amssymb, amsthm are recognised but not implemented` |
| 1 | `packages enumitem are recognised but not implemented` |
| 1 | `packages microtype are recognised but not implemented` |
| 1 | `\parindent is not supported in the document preamble` |
| 1 | `\parskip is not supported in the document preamble` |
| 1 | `\setlist is not supported in the document preamble` |
| 1 | `\pagestyle is not supported by this compiler version; unrestricted TeX math mode is not implemented` |
| 1 | `environment 'center' is not implemented; its body is typeset as plain text` |
| 1 | `\Large is not supported by this compiler version; unrestricted TeX math mode is not implemented` |
| 1 | `\LARGE is not supported by this compiler version; unrestricted TeX math mode is not implemented` |
| 1 | `\hrule is not supported by this compiler version; unrestricted TeX math mode is not implemented` |
| 1 | `\vspace is not supported by this compiler version; unrestricted TeX math mode is not implemented` |
| 1 | `\begin is not supported in math mode` |
| 1 | `\end is not supported in math mode` |
| 1 | `\Longrightarrow is not supported in math mode` |
| 1 | `\newpage is not supported by this compiler version; unrestricted TeX math mode is not implemented` |
| 1 | `environment 'quote' is not implemented; its body is typeset as plain text` |
| 1 | `overfull line: 4.76pt too wide (no hyphenation available)` |
| 1 | `overfull line: 14.68pt too wide (no hyphenation available)` |
| 1 | `no math glyph for '&'; empty box used` |
| 1 | `display is 454.63pt wider than the text width` |
| 1 | `display is 189.55pt wider than the text width` |
| 1 | `display is 19.04pt wider than the text width` |
| 1 | `overfull line: 3.80pt too wide (no hyphenation available)` |
| 1 | `overfull line: 26.36pt too wide (no hyphenation available)` |
| 1 | `lmmi10: glyphs drawn from latinmodern-math (one 10pt design); no optical-size OpenType outline resource exists for this family, so the outlines are not the reference's lmmi10 design` |
| 1 | `lmmi8: glyphs drawn from latinmodern-math (one 10pt design); no optical-size OpenType outline resource exists for this family, so the outlines are not the reference's lmmi8 design` |
| 1 | `lmsy10: glyphs drawn from latinmodern-math (one 10pt design); no optical-size OpenType outline resource exists for this family, so the outlines are not the reference's lmsy10 design` |

## Constructs used by this one document

Preamble: `\documentclass[11pt]{article}`, `fontenc[T1]`, `inputenc[utf8]`, `geometry[margin=1in]`, `amsmath`, `amssymb`, `amsthm`, `enumitem[shortlabels]` + `\setlist`, `microtype`, `\setlength{\parindent}`/`\parskip`, `\newcommand` with 0 and 2 arguments (`\problem{#1}{#2}` expanding to `\subsection*{… \hfill \normalfont[…]}`).
Body: `\pagestyle{empty}`, `center` env with `{\Large\bfseries …}\\[3pt]`, `\textbf`, `\hrule`, `\vspace`, `\subsection*`, `enumerate[(a)]` with `\item`, inline math `$a,b\in\Z_{>0}$`, display math `\[ … \]` with `\mid`, `\sqrt`, `array{ll}` with `\text{}` and `\\[2pt]`, `\forall`/`\exists`/`\exists!`, `\Rightarrow`/`\Longrightarrow`, `\vee`, `\bigl(`/`\bigr)`, `\quad`/`\qquad`, `\mathbb`, `\setminus`, `\emph`, `\newpage`, `quote` env, `---` and ``` ``…'' ``` quotes, `\\` line breaks.

## Ownership (per coordination/authority.json)
Parser/expansion diagnostics come from `crates/compiler` (Commander/main); math typesetting from `crates/math-layout` (Daniel); page/paragraph typesetting and the OpenType/TFM producer from `crates/render-pipeline` (mac-claude-a). The Mac shell already displays every diagnostic above with source spans and keeps the recovered layout.

