# Diagram and algorithm reference oracles

Three original one-page MacTeX references broaden the extended corpus from 55 to
58 projects: `amscd` commutative diagrams, `tikz-cd` diagrams, and `algorithm2e`.
All use the declared three-pass `pdflatex` workflow with shell escape disabled,
and all final logs are warning-free. Each final page was rasterized at 96 DPI RGB
Ghostscript settings and visually inspected for arrow direction/labels, matrix
spacing, line intersections, algorithm rules, nested vertical scopes, line
numbers, math and resolved references.

The initial algorithm source accidentally selected a small-caps italic Computer
Modern shape that the T1 encoding lacks; its reference emitted a font warning and
was discarded. The accepted `Locate` function heading removes that incidental
font dependency. This makes a missing-glyph/font-fallback regression a real test
failure instead of reference noise.

Sources, PDFs, logs, FLS dependency recordings, raw engine output and hashes are
under `tests/extended-tex-corpus/references/`. `reference-index.json` pins the
new manifest/source/PDF hashes. These are TeX oracle requirements, not claims that
FlashTeX currently supports these packages or achieves visual parity.

Recreate the final set with:

```sh
python3 tools/extended-tex-corpus/reference.py \
  --only diagrams-amscd --only diagrams-tikzcd --only packages-algorithm2e \
  --output /private/tmp/flashtex-diagram-algorithm-oracles --render
python3 -m unittest discover -s tools/extended-tex-corpus -v
```

No external provider calls, paid tooling, application changes, main/control
writes, or worktree changes outside this corpus branch were made.
