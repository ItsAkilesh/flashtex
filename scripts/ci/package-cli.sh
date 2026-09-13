#!/usr/bin/env bash
# Packages the command-line helpers as flashtex-cli-<version>-<platform>.tar.gz:
#
#   flashtex-cli-<version>-<platform>/
#     README.md
#     bin/flashtex-render        (+ flashtex-compiler, flashtex-pdf,
#                                 flashtex-pdf-exact when they were built)
#     bin/Fonts/                 pinned Latin Modern OTFs + GUST licence
#     bin/texmf/                 rooted TFM metrics + licence (LM 2.004)
#
# flashtex-render discovers `<exe>/Fonts` and `<exe>/texmf` on its own
# (crates/render-pipeline/src/fonts.rs, Discovery), so the tarball needs no
# host TeX installation and no --font-dir. Binaries that do not exist are
# listed as missing in README.md instead of failing the packaging, so a
# platform where a crate does not build still ships what it has.
#
# Usage: scripts/ci/package-cli.sh <version> <platform> <out-dir>
#          [--bin <path>]... [--fonts-dir <dir>] [--texmf-root <dir>]
#   <version>    e.g. 0.2.0 (a leading v is dropped)
#   <platform>   e.g. macos-arm64, linux-x86_64
#   <out-dir>    where the .tar.gz is written (created)
#   --bin        a built binary to include (default: the four helpers from
#                crates/*/target/release when present)
#   --fonts-dir  flat directory of .otf faces + GUST-FONT-LICENSE.TXT
#                (default: apps/mac/Fonts, the pinned vendored set)
#   --texmf-root rooted texmf tree with fonts/tfm/public/lm and
#                doc/fonts/lm (default: apps/mac/Fonts/texmf)
# Prints the tarball path on stdout.
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"

VERSION="${1:?version}"; PLATFORM="${2:?platform}"; OUT_DIR="${3:?out-dir}"
shift 3
VERSION="${VERSION#v}"
BINS=()
FONTS_DIR="$REPO_ROOT/apps/mac/Fonts"
TEXMF_ROOT="$REPO_ROOT/apps/mac/Fonts/texmf"
while [[ $# -gt 0 ]]; do
  case "$1" in
    --bin) BINS+=("${2:?--bin needs a path}"); shift 2 ;;
    --fonts-dir) FONTS_DIR="${2:?}"; shift 2 ;;
    --texmf-root) TEXMF_ROOT="${2:?}"; shift 2 ;;
    -h|--help) sed -n '2,27p' "${BASH_SOURCE[0]}"; exit 0 ;;
    *) echo "package-cli.sh: unknown argument: $1" >&2; exit 2 ;;
  esac
done
die() { echo "package-cli.sh: $*" >&2; exit 1; }

if [[ ${#BINS[@]} -eq 0 ]]; then
  for p in render-pipeline/flashtex-render compiler/flashtex-compiler pdf/flashtex-pdf pdf/flashtex-pdf-exact; do
    BINS+=("$REPO_ROOT/crates/${p%%/*}/target/release/${p##*/}")
  done
fi
[[ -d "$FONTS_DIR" ]] || die "fonts dir not found: $FONTS_DIR"
[[ -d "$TEXMF_ROOT/fonts/tfm/public/lm" ]] || die "texmf root has no fonts/tfm/public/lm: $TEXMF_ROOT"
[[ -f "$TEXMF_ROOT/doc/fonts/lm/GUST-FONT-LICENSE.TXT" ]] || die "texmf root has no GUST licence: $TEXMF_ROOT"

NAME="flashtex-cli-$VERSION-$PLATFORM"
STAGE="$(mktemp -d "${TMPDIR:-/tmp}/flashtex-cli.XXXXXX")"
trap 'rm -rf "$STAGE"' EXIT
ROOT="$STAGE/$NAME"
mkdir -p "$ROOT/bin/Fonts" "$ROOT/bin/texmf"

INCLUDED=(); MISSING=()
for b in "${BINS[@]}"; do
  if [[ -x "$b" ]]; then
    cp "$b" "$ROOT/bin/"
    INCLUDED+=("$(basename "$b")")
  else
    MISSING+=("$(basename "$b")")
  fi
done
[[ ${#INCLUDED[@]} -gt 0 ]] || die "no binary to package (looked for: ${BINS[*]})"
[[ " ${INCLUDED[*]} " == *" flashtex-render "* ]] || echo "package-cli.sh: warning: flashtex-render is not among the binaries" >&2

# Flat faces + licence (the layout Discovery calls "flat"): only the pinned
# .otf files and the licence, never a stray file from the directory.
cp "$FONTS_DIR"/*.otf "$ROOT/bin/Fonts/"
cp "$FONTS_DIR"/GUST-FONT-LICENSE.TXT "$ROOT/bin/Fonts/"
[[ -f "$FONTS_DIR/SUPPLEMENTARY-FACES.json" ]] && cp "$FONTS_DIR/SUPPLEMENTARY-FACES.json" "$ROOT/bin/Fonts/"
# Rooted metrics tree, as vendored (TFMs + licence + pin manifest).
cp -R "$TEXMF_ROOT/." "$ROOT/bin/texmf/"

{
  echo "# FlashTeX command-line tools $VERSION ($PLATFORM)"
  echo
  echo "Built from https://github.com/flash-tex/flashtex (release v$VERSION)."
  echo
  echo "## Contents"
  echo
  for b in "${INCLUDED[@]}"; do echo "- \`bin/$b\`"; done
  for b in "${MISSING[@]}"; do echo "- \`bin/$b\` — not built for $PLATFORM in this release"; done
  echo "- \`bin/Fonts/\` — Latin Modern OpenType faces (GUST Font License, see GUST-FONT-LICENSE.TXT)"
  echo "- \`bin/texmf/\` — the pinned Latin Modern 2.004 TFM metrics flashtex-render lays text out with"
  echo
  echo "## Usage"
  echo
  echo '```'
  echo "bin/flashtex-render --tex main.tex --pdf main.pdf     # single file, diagnostics on stderr"
  echo "bin/flashtex-render --tex main.tex --v2 main.json     # rendering-v2 display list"
  echo "bin/flashtex-render < requests.jsonl                  # runtime-v1 JSON Lines worker"
  echo "bin/flashtex-render --help"
  echo '```'
  echo
  echo "Exit status of \`--tex\`: 0 when the document rendered (ok/recovered), 1 when it failed, 2 when the file cannot be read."
  echo
  echo "The fonts and metrics are found relative to the executable (\`bin/Fonts\`, \`bin/texmf\`);"
  echo "keep the directory layout when moving the tools, or point \`FLASHTEX_FONT_DIRS\` /"
  echo "\`FLASHTEX_TFM_DIRS\` (colon separated) at your own copies. No TeX installation is required."
} > "$ROOT/README.md"

mkdir -p "$OUT_DIR"
OUT="$(cd "$OUT_DIR" && pwd)/$NAME.tar.gz"
tar -C "$STAGE" -czf "$OUT" "$NAME"
echo "$OUT"
