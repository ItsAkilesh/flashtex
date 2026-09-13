# What this compiler does not implement

Generated from the real compiler by `cargo test --test unsupported_inventory generate_unsupported_inventory -- --ignored --exact`.

Each entry is a required-but-outstanding item in the master plan's sense. Every one is reported explicitly by the compiler; none fails silently. Documents using these still compile and still produce readable output.

## \usepackage — package implementations

Any document relying on package-defined commands will report them as unsupported.

Input:

```text
\usepackage{amsmath}
Text.
```

Status: `recovered`

Diagnostics:

- `packages amsmath are recognised but not implemented` — recovery: continued without package-specific commands or formatting
## TikZ diagrams

The demo contract names a small TikZ diagram; it is not implemented.

Input:

```text
\begin{tikzpicture}\draw (0,0) -- (1,1);\end{tikzpicture}
```

Status: `recovered`

Diagnostics:

- `environment 'tikzpicture' is not implemented; its body is typeset as plain text` — recovery: typeset the body without the environment's formatting
- `\draw is not supported by this compiler version; unrestricted TeX math mode is not implemented` — recovery: skipped the command; any braced argument was typeset as plain text
## \includegraphics — image loading

Figures lay out and number, but no image is loaded or drawn.

Input:

```text
\begin{figure}\includegraphics{plot.png}\caption{P}\end{figure}
```

Status: `recovered`

Diagnostics:

- `\includegraphics is unsupported; image loading is not implemented` — recovery: omitted the image and continued
## longtable — multi-page tables

tabular is laid out, but longtable's page-breaking tables are typeset as plain text.

Input:

```text
\begin{longtable}{ll}a & b \\ c & d\end{longtable}
```

Status: `recovered`

Diagnostics:

- `environment 'longtable' is not implemented; its body is typeset as plain text` — recovery: typeset the body without the environment's formatting
## \cite and bibliographies

Citations do not resolve; .bib files are not read.

Input:

```text
Text \cite{knuth1984}.
```

Status: `recovered`

Diagnostics:

- `\cite is not supported by this compiler version; unrestricted TeX math mode is not implemented` — recovery: skipped the command; any braced argument was typeset as plain text
## \def, \let and category codes

Only \newcommand-style macros exist; raw TeX programmability does not.

Input:

```text
\def\x{y}\x
```

Status: `recovered`

Diagnostics:

- `\def is not supported by this compiler version; unrestricted TeX math mode is not implemented` — recovery: skipped the command; any braced argument was typeset as plain text
- `\x is not supported by this compiler version; unrestricted TeX math mode is not implemented` — recovery: skipped the command; any braced argument was typeset as plain text
- `\x is not supported by this compiler version; unrestricted TeX math mode is not implemented` — recovery: skipped the command; any braced argument was typeset as plain text
