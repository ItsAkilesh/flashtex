# Extended TeX/LaTeX reference corpus

Owner: `mac-reference-corpus`, directly assigned by the user on 2026-09-12.
Status: 55 original projects, 48 positive reference PDFs (57 pages), seven expected
error cases, and fourteen incremental edit scenarios. No FlashTeX compatibility claim.
This is an independent extension of `tests/tex-corpus`, `tests/visual-corpus`, and
`fixtures/real-world`; it does not replace their manifests or their owners.

## Run it

From the repository root, with Python 3.9+:

```sh
python3 tools/extended-tex-corpus/reference.py --validate
python3 -m unittest discover -s tools/extended-tex-corpus -v
python3 tools/extended-tex-corpus/reference.py --output /tmp/extended-new-run --render
python3 tools/extended-tex-corpus/reference.py --output /tmp/extended-align --render --only math-align-points
python3 tools/extended-tex-corpus/reference.py --emit-request math-align-points
```

Generation requires installed MacTeX or TeX Live (`--texbin` selects its bin
folder), with Ghostscript for `--render`. A fresh output directory is required;
existing oracle artifacts are never overwritten by the runner. It does not
install packages, use network services, or enable shell escape. Each engine step
has a 60-second timeout; `--timeout` adjusts that bound. The complete batch takes
several minutes. LuaTeX uses a cache inside the selected output directory.

`--emit-request` produces a runtime-v1 compile request containing the exact source
bytes decoded as UTF-8, including `.sty`, `.cls`, bibliography and data inputs.
Send it to a trusted original compiler executable, retaining its response and
stderr. Runtime-v1 has no engine-selector or binary-asset field: engine profiles
are test metadata, and the PNG project is explicitly rejected by this adapter.
Use an existing project-files adapter for that project; do not omit its image or
claim that sending a filename constitutes asset support. A non-TeX source document
in the envelope does not prove the compiler can consume its package/data format.

Reference engines are development oracles only. None of these tools belongs in
FlashTeX's compilation or preview path.

## Artifacts and acceptance

- `manifest.json`: entry points, complete asset inventories, engine/workflow,
  features, authorship, and positive/negative expectations.
- `cases/<id>/`: small, editable, standalone project sources. All authored for this
  corpus; no third-party project source was copied. Repository license applies.
  Packages/fonts are resolved from the local TeX installation, not redistributed.
- `references/<id>/main.pdf`: positive reference output. Negative runs may leave a
  partial PDF in scratch, but it is deliberately not accepted as a golden PDF.
- `references/<id>/reference.json`: exact engine version and executable hash,
  commands, return codes, timing, source hashes, resolved input hashes, warnings,
  PDF digest, and rasterizer version/hash/arguments/page hashes.
- Logs and `.fls` files alongside each reference preserve package versions,
  diagnostics, resolved files and build evidence. Original generation paths in the
  recorder/provenance files are historical evidence, not portable lookup paths.
- `reference-index.json`: final source/PDF hashes and observed page counts for all
  55 cases. Integrity tests reject stale sources or PDFs.
- `edits.json`: fourteen exact UTF-8 byte replacements and their expected dependency
  effects. Feed baseline then edited project to an incremental compiler; compare
  the edited result against a clean build of identical edited bytes. Compare warm
  auxiliary-file workflows separately from clean auxiliary state.

For positive cases, assess complete page count and dimensions, text order,
math/diagram geometry, font selection/embedding, cross-reference resolution,
diagnostics, and all rendered pages. PDF byte equality and pixel equality are
separate gates. Use the SAME pinned rasterizer/settings for candidate and oracle;
96 DPI RGB Ghostscript settings are recorded here. Exact comparison allows no
translation, crop, rescale, blur, or tolerance adjustment. Text-only agreement is
insufficient for math, alignment, colors, or graphics. Use existing
`tools/raster-compare` tooling for full-resolution comparison after matching its
input/provenance requirements; these thumbnails do not replace that gate.

For negative cases, the reference diagnostic is a stable fragment from this
installed TeX version. Some engines emit a warning and return zero for a group
left open at end of document; expected-error classification checks the log, not
just exit status. FlashTeX may use different diagnostic wording and a different
explicit recovery policy. Require a correctly located diagnostic and validate
recovered output separately. Unsupported is an outstanding feature, even when
honestly diagnosed. Timeout/crash is never an expected-error pass.

The original 51 positive pages were inspected in contact sheets, with full-size spot checks
of alignment, delimiter sizing and clipping. Book verso pages are intentionally
blank; narrow-paragraph/discretionary underfull warnings are intentional probes.
The `epstopdf` shell-disabled warning in the units/chemistry case does not invoke
conversion. Initial font/cache failures and accidental graphic/link overflow were
fixed and rerun; only matching final source hashes are accepted in the index.

Six added HW1 probes were individually inspected at 96 DPI. They isolate
blackboard alphabets and macro expansion, arrays/cases, delimiter/kern widths,
title size/weight scoping, paragraph registers, and heading hfill/quote widths.
The hfill fixture's paragraph boundary was corrected and rebuilt before accepting
its PDF. [The recorded local compiler baseline](../../docs/evidence/hw1-reference-probes-20260912/README.md)
contains recovered results and 173 diagnostics; these references add requirements,
not a support claim. Run any one with `--only hw1-array-cases`, for example.

## Coverage map

| Area | Cases / distinguishing checks |
| --- | --- |
| Math atoms and fonts | Binary/relation spacing, Greek variants, calligraphic/blackboard/fraktur/bold alphabets, punctuation and logic |
| Math construction | Nested fractions, binomials, four styles, scripts/primes, limits/substacks, sums/products/integrals, radicals |
| Delimiters and accents | Automatic/manual sizing, null and middle delimiters, wide accents, braces, phantom/smash, overset/underset |
| Math environments | Matrix variants, cases, smallmatrix, array preambles and row spacing; align/alignat/flalign, split/gather/multline/aligned/intertext |
| Advanced math packages | Mathtools paired delimiters, dcases, prescripts, arrows, right-aligned matrices, subequations and section numbering |
| TeX execution | Delimited and optional arguments, edef/noexpand/expandafter/csname, local/global scope, registers/arithmetic, conditionals, token registers, loops/aftergroup |
| Tokenization | Active characters, changed escape category code, internal command names, comments and verbatim special characters |
| Plain TeX | halign/noalign/eqalign, box registers and dimensions; engine explicitly pdfTeX plain format |
| Text and layout | Accents, ligatures/kerning, font faces/sizes, discretionary breaks, italic correction, stretch orders, leaders/rules/boxes, minipage/parbox |
| Tables and documents | Multirow/multicolumn, partial rules, wrapping cells, booktabs, three-page longtable, lists/theorems/proofs, floats/captions/footnotes, TOC/LOF |
| Graphics | Color models, transforms, picture primitives, included PNG quadrants, crop/rotation/distortion |
| TikZ/PGFPlots | Bézier paths, arcs, grids, scopes, styles/nodes/fit/positioning/loops, arrow tips, clipping/patterns/shading/transparency, functions and file data |
| Packages | siunitx numeric columns/units, mhchem reactions/isotopes, listings, microtype and balanced columns, hyperref links/bookmarks/PDF strings |
| Real project structure | Nested input, local package/class, book chapters/includeonly/aux state, BibTeX/plain and biblatex/Biber author-year |
| Unicode profiles | LuaLaTeX and XeLaTeX, installed Libertinus text fonts and Latin Modern Math, Greek/Cyrillic text and Unicode math |
| Invalid syntax | Unknown command, open group, misplaced &, missing input, double superscript, mismatched environment, missing right delimiter |

## Next coverage gates

This finite suite does NOT cover “all math” or arbitrary TeX programs. Remaining
areas include math font family tables and every symbol, extensible-arrow variants,
RTL/bidi/CJK and combining-mark shaping, amscd/tikz-cd, circuitikz/chemfig,
algorithm packages, beamer overlays, glossaries/index generation, SVG/EPS/PDF image
assets, custom output routines/inserts, page-builder penalties, e-TeX expression
and marks semantics, engine-specific primitives, large real licensed projects,
and generated stress/property cases with minimization. Keep these outstanding;
never interpret absence from this corpus as lack of product requirement.
