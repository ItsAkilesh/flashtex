# Exact raster comparison

Owner: commander-raster. September 12, 2026. Product validation tool; reference
generation and original compiler implementation remain separate workstreams.

```sh
python3 tools/raster-compare/compare.py \
  --reference reference-page-1.png --candidate candidate-page-1.png \
  --provenance pair-page-1.json
python3 -m unittest discover -s tools/raster-compare -p 'test_*.py' -v
```

Pillow 12.1.0 was already available on the development machine; no installation,
renderer download, reference TeX invocation, paid inference or network request was
performed. `requirements.txt` pins the decoder used by the tests. Output also
records the actual decoder version. Input images and provenance are read only;
the CLI writes its JSON result to stdout.

Exit **0** means every decoded RGBA8 byte and both dimensions match. Exit **1**
means a pixel or dimension mismatch. Exit **2** means rejected inputs, unsupported
encoding/profile, invalid provenance, or a resource-bound failure. An error always
has `exact_equal: false`. Invalid hashes cannot produce a passing comparison even
when the two PNGs happen to look identical.

Supported input is single-frame, 8-bit-per-sample RGB or RGBA PNG. RGB becomes RGBA
by appending opaque alpha; no other color conversion is performed. Palette,
grayscale, 16-bit, color-key transparency, animation and nonidentity EXIF orientation
are rejected. Both sides must have identical ICC/gamma/sRGB/chromaticity metadata.
The comparator does not silently interpret or repair incompatible color profiles.

Acceptance is exact across the entire corresponding page. Different PNG compression
or ancillary text may yield different encoded file hashes yet identical decoded
pixels, which passes. Alpha changes fail, as do RGB changes beneath zero alpha.
There are no tolerance, alignment, cropping, scale, blur or threshold options.
Images with equal pixel hashes but different dimensions fail.

For equal dimensions, diagnostics report differing-pixel count, maximum channel
delta, per-channel counts and the exact global mismatch bounding box. Bounds are
`[left, top, right, bottom]`, origin at the upper-left, with right/bottom exclusive.
Each diagnostic region is a fixed tile containing changed pixels, with its own
tight bounding box and count; regions are not connected-component claims.
`--region-size` defaults to 64 pixels and `--max-regions` to 100. Regions are sorted
by tile row then column. Truncation is explicit, while total mismatch counts and
acceptance still cover the whole page. These options affect diagnostics only.

Different dimensions fail before overlap comparison. The result retains both page
bounds and the number of pixels absent from one image at the original origin;
overlap difference count and maximum channel delta remain null. It does not crop
to the common area and report a misleading pass.

Images are bounded to 25 million pixels each. Diagnostic grids exceeding 100,000
tiles are rejected; increasing region size reduces diagnostic memory without
changing exact acceptance. PNG decompression warnings are errors.

Pair provenance is a JSON object with these required fields (replace all digest
placeholders with actual lowercase SHA256 hex values):

```json
{
  "schema_version": 1,
  "case_id": "math-fractions",
  "profile": "pdflatex-cm-12pt-letter-144dpi",
  "source_sha256": "SOURCE_SHA256",
  "page_index": 0,
  "reference": {
    "source_sha256": "SOURCE_SHA256",
    "image_sha256": "REFERENCE_PNG_SHA256",
    "pixel_sha256": "REFERENCE_RGBA8_SHA256",
    "rasterizer_binary_sha256": "RASTERIZER_BINARY_SHA256",
    "rasterizer_version_and_build": "ACTUAL RASTERIZER NAME VERSION AND BUILD",
    "raster_argv": ["pdftoppm", "-r", "144", "-png", "-aa", "yes", "-aaVector", "yes", "-thinlinemode", "none", "INPUT.pdf", "OUTPUT_PREFIX"]
  },
  "candidate": {
    "source_sha256": "SOURCE_SHA256",
    "image_sha256": "CANDIDATE_PNG_SHA256",
    "pixel_sha256": "CANDIDATE_RGBA8_SHA256",
    "rasterizer_binary_sha256": "RASTERIZER_BINARY_SHA256",
    "rasterizer_version_and_build": "ACTUAL RASTERIZER NAME VERSION AND BUILD",
    "raster_argv": ["pdftoppm", "-r", "144", "-png", "-aa", "yes", "-aaVector", "yes", "-thinlinemode", "none", "INPUT.pdf", "OUTPUT_PREFIX"]
  }
}
```

`pixel_sha256` is optional but verified when supplied. It hashes decoded row-major
RGBA8 bytes, with dimensions checked separately. Per-side source hashes are
required and must match the shared source hash. Encoded PNG hashes are verified
against the exact bytes decoded. Rasterizer binary digest, version/build and
normalized argument template must match on both sides. Commands are recorded,
never executed by this tool. The output retains the supplied provenance and its
file hash, computed image/pixel hashes, dimensions and metadata.

Visual-corpus integration maps `reference.pages[].sha256` to `image_sha256` and
`pixel_sha256` directly; the manifest's one-based page `number` maps to zero-based
`page_index`. Map `reference.rasterizer.binary_sha256` to
`rasterizer_binary_sha256`, use the same actual rasterizer name/version/build
description on both sides, and map `reference.render.argv_template` to
`raster_argv`. The candidate producer must retain equivalent source and rasterizer
provenance. The visual manifest validator separately verifies source files, expected
page count/dimensions, fonts/packages and reference generation artifacts.

A pair comparison proves pixel equality for those two supplied PNGs only. Producer
statements about source/build provenance are not cryptographic proof that a renderer
actually used them. Synthetic tests or comparing an image to itself are comparator
checks, not evidence of original-compiler fidelity. Missing reference generation,
native rasterization and whole-project acceptance remain separate gates.
