# Visual oracle fixture specification

Owner: commander-corpus, fixture subtree only. Companion visual workers own the
comparison harness/reference generation separately. These eight original projects
exercise paragraph breaking, kerning/ligatures, font faces, fractions, scripts,
radicals, delimiters and three-page baseline placement. They require ordinary
LaTeX behavior that remains outstanding in the original Rust compiler.

`manifest.json` defines the source hashes, engine/font/DPI profile, expected page
count, and comparison logic. The ordinary reference profile is pdfLaTeX with
Computer Modern (12pt article), `amsmath` and `amssymb`. It intentionally does not
replace the document's default font with Times to make current output look closer.
Metric, face, size and glyph differences are meaningful failures.

**No reference PDF/image has been generated here.** The read-only
[tool inventory](tool-inventory.json) found Poppler 26.01.0, Ghostscript 10.06.0,
and ImageMagick 7.1.2-13, but no pdfLaTeX/XeLaTeX/LuaLaTeX/Tectonic/latexmk in PATH.
This is a fixture specification with integrity checks, not validated TeX syntax,
visual acceptance evidence, or proof that all required fonts are installed.

Run fixture checks from repository root:

```sh
python3 tests/visual-corpus/fixtures/validate.py
```

## Reference generation on a capable machine

Copy one fixture into an isolated output directory and run a real reference engine
there with network/shell execution disabled:

```sh
pdflatex -interaction=nonstopmode -halt-on-error -no-shell-escape -recorder CASE.tex
pdflatex -interaction=nonstopmode -halt-on-error -no-shell-escape -recorder CASE.tex
pdftoppm -r 144 -png -aa yes -aaVector yes -thinlinemode none CASE.pdf reference
```

These are test-only commands. The product must compile source with the original
Rust implementation, never dispatch it to this reference engine. Inspect the log
and actual output before accepting a reference. Record the exact engine version,
LaTeX format, package versions (`\listfiles`), resolved inputs (`.fls`), all font
and metric file hashes, and the rasterizer build/arguments. `pdffonts` can inspect
PDF font names/embedding; it does not replace hashing the source font/metric files.
Absent required fonts/packages is a blocked oracle, not permission to substitute
another reference font or synthesize an expected image.

The compiler candidate receives the identical source bytes and declared font/page
profile. Preserve raw candidate display list, diagnostics, PDF, compiler SHA, build
command, and font inputs. Render candidate and reference with the **same pinned
rasterizer binary and settings**, ideally on the same machine. Each Letter page
must be 1224×1584 pixels at 144 DPI. Compare decoded opaque RGB pixels, not PNG
file bytes or PDF metadata; dates/object numbering/compression need not match.

## Pixel equality and useful diagnosis

The primary gate requires identical page count/dimensions and zero differing
pixels/channels on each entire corresponding page. No automatic translation,
crop, rescale, blur or threshold relaxation is permitted in the passing gate.
Store difference heatmaps and bounding boxes to diagnose geometry/glyph errors;
nonzero tolerances can be reported separately but cannot be called pixel-perfect.
If environment provenance differs, report an incomparable run and regenerate
under a common pinned environment before blaming the compiler.

A candidate that reports unsupported commands remains unsupported even if blank
or partial pages happen to match a cropped region. A generated oracle must first
be inspected for correct page count, nonempty expected content and successful
compilation. `expected_page_count` is a declared target until actual generation
confirms it; accidental overfull content/additional pages must be investigated.

The multipage case uses explicit page breaks to isolate repeated origins and
baseline placement from automatic page-break policy. The paragraph case uses
fixed 240pt and 180pt measures to expose actual line breaking. The kerning case
includes explicit zero-kern controls, so ligatures and natural spacing are tested
against real reference output rather than guessed glyph coordinates.

Export dark-mode behavior and source-navigation provenance remain separate gates:
white background is the reference export policy; screenshots of a dark preview do
not replace export comparison. Pixel equality on these eight projects would still
not establish universal LaTeX compatibility.
