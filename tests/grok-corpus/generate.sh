#!/usr/bin/env bash
# Regenerates the entire Grok handwriting-corpus (tests/grok-corpus/images/ and
# manifest.json) from the LaTeX sources under tests/grok-corpus/cases/.
#
# Requirements (all offline, no network, no API keys):
#   - pdflatex and xelatex (MacTeX / TeX Live), with amsmath, unicode-math,
#     fontspec, multicol and geometry available (standard in a full install).
#   - pdfcrop (ships with TeX Live).
#   - Ghostscript (`gs`) for PDF -> PNG rasterization.
#   - cargo (to build the corpus_augment degradation tool from crates/bridge).
#   - python3 (stdlib only) to assemble manifest.json.
#
# Usage:
#   tests/grok-corpus/generate.sh
#
# Every step is deterministic (fixed DPI/margins, seeded noise) so re-running
# this script reproduces the same corpus. LaTeX/PDF build artifacts are kept
# in tests/grok-corpus/build/ (git-ignored, safe to delete) and only the final
# PNG/JPEG images plus manifest.json are meant to be committed.
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
CORPUS_DIR="$SCRIPT_DIR"
CASES_DIR="$CORPUS_DIR/cases"
IMAGES_DIR="$CORPUS_DIR/images"
BUILD_DIR="$CORPUS_DIR/build"
BRIDGE_DIR="$ROOT/crates/bridge"

for tool in pdflatex xelatex pdfcrop gs cargo python3; do
  command -v "$tool" >/dev/null 2>&1 || {
    echo "error: required tool '$tool' not found on PATH" >&2
    exit 1
  }
done

rm -rf "$BUILD_DIR" "$IMAGES_DIR"
mkdir -p "$BUILD_DIR" "$IMAGES_DIR"

echo "==> building corpus_augment (release)"
cargo build --release --manifest-path "$BRIDGE_DIR/Cargo.toml" --bin corpus_augment
AUGMENT="$BRIDGE_DIR/target/release/corpus_augment"

# compile_case <case-id> <engine: pdflatex|xelatex>
compile_case() {
  local id="$1" engine="$2"
  cp "$CASES_DIR/$id.tex" "$BUILD_DIR/$id.tex"
  ( cd "$BUILD_DIR" && "$engine" -interaction=nonstopmode -halt-on-error "$id.tex" >"$id.compile.log" 2>&1 ) \
    || { echo "error: $engine failed for $id (see $BUILD_DIR/$id.compile.log)" >&2; exit 1; }
}

# crop_and_raster <case-id> <margin-pt> <dpi> <out-ext: png|jpg> [jpeg-quality]
crop_and_raster() {
  local id="$1" margin="$2" dpi="$3" ext="$4" quality="${5:-92}"
  pdfcrop --margins "$margin" "$BUILD_DIR/$id.pdf" "$BUILD_DIR/$id-crop.pdf" >"$BUILD_DIR/$id.crop.log" 2>&1
  gs -q -dNOPAUSE -dBATCH -sDEVICE=png16m -r"$dpi" -dTextAlphaBits=4 -dGraphicsAlphaBits=4 \
     -sOutputFile="$BUILD_DIR/$id.png" "$BUILD_DIR/$id-crop.pdf"
  if [ "$ext" = "jpg" ]; then
    "$AUGMENT" jpeg "$BUILD_DIR/$id.png" "$IMAGES_DIR/$id.jpg" "$quality"
  else
    cp "$BUILD_DIR/$id.png" "$IMAGES_DIR/$id.png"
  fi
}

echo "==> typesetting clean/structural/adversarial-prose cases"
compile_case clean-quadratic-formula pdflatex
crop_and_raster clean-quadratic-formula 40 200 png

compile_case clean-integral-unicode xelatex
crop_and_raster clean-integral-unicode 40 200 png

compile_case clean-multiline-derivation pdflatex
crop_and_raster clean-multiline-derivation 30 200 png

compile_case clean-matrix-determinant pdflatex
crop_and_raster clean-matrix-determinant 30 200 jpg 92

compile_case clean-nested-fraction-scripts pdflatex
crop_and_raster clean-nested-fraction-scripts 30 200 png

compile_case clean-mixed-prose-math pdflatex
crop_and_raster clean-mixed-prose-math 20 200 jpg 90

compile_case clean-multicolumn-two-equations pdflatex
crop_and_raster clean-multicolumn-two-equations 20 200 png

compile_case clean-two-problems-page pdflatex
crop_and_raster clean-two-problems-page 20 200 jpg 90

compile_case adversarial-prose-no-math pdflatex
crop_and_raster adversarial-prose-no-math 20 200 png

echo "==> synthesizing non-LaTeX adversarial cases"
"$AUGMENT" solid "$IMAGES_DIR/adversarial-blank-page.png" 1000 1300 255 255 255
# Deliberately modest resolution: random noise is incompressible (a PNG of it
# is close to raw width*height*3 bytes), and resolution adds nothing to what
# this adversarial case is testing.
"$AUGMENT" pure-noise "$IMAGES_DIR/adversarial-pure-noise.png" 300 400 20260912

echo "==> generating degraded-photo variants"
degrade_all() {
  local base="$1"
  local src="$IMAGES_DIR/$base.png"
  "$AUGMENT" rotate "$src" "$IMAGES_DIR/degraded-$base-rotate6.png" 6
  "$AUGMENT" rotate "$src" "$IMAGES_DIR/degraded-$base-rotate15.png" 15
  "$AUGMENT" perspective "$src" "$IMAGES_DIR/degraded-$base-perspective.png" 0.6
  "$AUGMENT" contrast "$src" "$IMAGES_DIR/degraded-$base-low-contrast.png" -55
  "$AUGMENT" lighting-gradient "$src" "$IMAGES_DIR/degraded-$base-lighting-gradient.png" 0.6
  "$AUGMENT" jpeg "$src" "$IMAGES_DIR/degraded-$base-jpeg-artifacts.jpg" 8
  "$AUGMENT" downscale-blur "$src" "$IMAGES_DIR/degraded-$base-downscale-blur.png" 0.28
  "$AUGMENT" noise "$src" "$IMAGES_DIR/degraded-$base-noise.png" 30 42
}
degrade_all clean-quadratic-formula
degrade_all clean-multiline-derivation

echo "==> writing manifest.json"
python3 "$CORPUS_DIR/build_manifest.py"

echo "==> done"
du -sh "$IMAGES_DIR" | awk '{print "images/ total size: " $1}'
find "$IMAGES_DIR" -type f | wc -l | awk '{print "image count: " $1}'
