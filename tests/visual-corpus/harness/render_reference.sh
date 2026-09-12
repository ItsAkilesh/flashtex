#!/usr/bin/env bash
# Render every fixture with each available reference TeX engine (oracle only;
# FlashTeX never invokes these), then rasterize with the shared CoreGraphics
# rasterizer at a fixed DPI.
#
# Usage: render_reference.sh --out <dir> --rasterize <bin> [--dpi 144]
#                            [--texbin /Library/TeX/texbin] [--fixtures <dir>]
#                            [--engine pdflatex --engine xelatex ...]
#
# Output layout: <out>/<fixture>/<engine>/main.tex|main.pdf|main.log|
#                page-p<N>.png|.rgba|.rgba.json|raster.json|engine.json|words.json
# plus <out>/engines.json with versions/availability.
#
# Preambles (everything before \begin{document} in the fixture is replaced):
#   pdflatex : 12pt article, T1 fontenc, times (URW Nimbus Roman clone via
#              psnfss), geometry margin=1in, parindent 0, secnumdepth 0, empty
#              pagestyle -- the native-validation oracle "B" preamble.
#   xelatex/lualatex : same, but fontspec with \setmainfont{Times New Roman}
#              (the macOS system TrueType face) instead of T1 + times.
set -euo pipefail
HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
OUT=""; RASTERIZE=""; DPI=144; TEXBIN=/Library/TeX/texbin; FIXTURES="$HERE/fixtures"
ENGINES=()
while [[ $# -gt 0 ]]; do
  case "$1" in
    --out) OUT="$2"; shift 2 ;;
    --rasterize) RASTERIZE="$2"; shift 2 ;;
    --dpi) DPI="$2"; shift 2 ;;
    --texbin) TEXBIN="$2"; shift 2 ;;
    --fixtures) FIXTURES="$2"; shift 2 ;;
    --engine) ENGINES+=("$2"); shift 2 ;;
    -h|--help) sed -n '2,20p' "$0"; exit 0 ;;
    *) echo "unknown argument: $1" >&2; exit 2 ;;
  esac
done
[[ -n "$OUT" && -n "$RASTERIZE" ]] || { echo "--out and --rasterize are required" >&2; exit 2; }
[[ ${#ENGINES[@]} -gt 0 ]] || ENGINES=(pdflatex xelatex lualatex)
mkdir -p "$OUT"

PRE_PDFLATEX='\documentclass[12pt]{article}
\usepackage[T1]{fontenc}
\usepackage{times}
\usepackage[margin=1in]{geometry}
\setlength{\parindent}{0pt}
\setcounter{secnumdepth}{0}
\pagestyle{empty}
'
PRE_FONTSPEC='\documentclass[12pt]{article}
\usepackage{fontspec}
\setmainfont{Times New Roman}
\usepackage[margin=1in]{geometry}
\setlength{\parindent}{0pt}
\setcounter{secnumdepth}{0}
\pagestyle{empty}
'
FLAGS=(-interaction=batchmode -halt-on-error -file-line-error)

# Engine availability and versions, recorded honestly.
python3 - "$OUT/engines.json" "$TEXBIN" "${ENGINES[@]}" <<'PY'
import json, os, subprocess, sys
out, texbin, engines = sys.argv[1], sys.argv[2], sys.argv[3:]
info = {}
for e in engines:
    path = os.path.join(texbin, e)
    row = {"path": path, "available": os.access(path, os.X_OK)}
    if row["available"]:
        v = subprocess.run([path, "--version"], capture_output=True, text=True).stdout.splitlines()
        row["version"] = v[0] if v else "?"
    info[e] = row
kp = os.path.join(texbin, "kpsewhich")
pk = {}
if os.access(kp, os.X_OK):
    for sty in ["times.sty", "fontspec.sty", "geometry.sty", "t1enc.def", "utopia.sty"]:
        r = subprocess.run([kp, sty], capture_output=True, text=True).stdout.strip()
        pk[sty] = r or None
    r = subprocess.run([kp, "-var-value", "SELFAUTOPARENT"], capture_output=True, text=True).stdout.strip()
    pk["texlive_root"] = r
info["_packages"] = pk
fonts = {}
for f in ["/System/Library/Fonts/Supplemental/Times New Roman.ttf", "/System/Library/Fonts/Times.ttc"]:
    fonts[f] = os.path.exists(f)
info["_system_fonts"] = fonts
json.dump(info, open(out, "w"), indent=2)
PY

for tex in "$FIXTURES"/*.tex; do
  name="$(basename "$tex" .tex)"
  for engine in "${ENGINES[@]}"; do
    bin="$TEXBIN/$engine"
    dir="$OUT/$name/$engine"
    mkdir -p "$dir"
    if [[ ! -x "$bin" ]]; then
      echo "{\"engine\":\"$engine\",\"available\":false}" > "$dir/engine.json"
      echo "-- $name/$engine: not available"; continue
    fi
    case "$engine" in
      pdflatex) pre="$PRE_PDFLATEX" ;;
      *) pre="$PRE_FONTSPEC" ;;
    esac
    python3 - "$tex" "$dir/main.tex" "$pre" <<'PY'
import re, sys
src = open(sys.argv[1], encoding="utf-8").read()
m = re.search(r"^[ \t]*\\begin\{document\}", src, re.M)
body = src[m.start():] if m else src
open(sys.argv[2], "w", encoding="utf-8").write(sys.argv[3] + body)
PY
    t0=$(python3 -c 'import time;print(time.time())')
    set +e
    ( cd "$dir" && "$bin" "${FLAGS[@]}" main.tex >/dev/null 2>&1 )
    rc=$?
    set -e
    secs=$(python3 -c "import time;print(round(time.time()-$t0,3))")
    if [[ $rc -eq 0 && -f "$dir/main.pdf" ]]; then
      "$RASTERIZE" pdf "$dir/main.pdf" "$dir/page" --dpi "$DPI" > "$dir/raster.json"
      "$RASTERIZE" word-boxes "$dir/main.pdf" > "$dir/words.json"
    fi
    python3 - "$dir" "$engine" "$bin" "$rc" "$secs" "${FLAGS[*]}" "$pre" <<'PY'
import json, re, sys, os
d, engine, path, rc, secs, flags, pre = sys.argv[1:8]
log = open(os.path.join(d, "main.log"), encoding="latin-1").read() if os.path.exists(os.path.join(d, "main.log")) else ""
fonts = sorted(set(re.findall(r"(?:Font|font)\s+([A-Za-z0-9\-]+(?:/[A-Za-z0-9\-\[\]]+)?)", log)))[:20]
errors = [l for l in log.splitlines() if l.startswith("!")][:5]
warns = [l for l in log.splitlines() if "Warning" in l][:10]
fontspec_font = re.findall(r"Font\s+'([^']+)'\s+\(([^)]+)\)", log)
json.dump({"engine": engine, "available": True, "path": path, "exit": int(rc), "seconds": float(secs),
           "flags": flags.split(), "preamble": pre, "errors": errors, "warnings": warns,
           "fontspec_fonts": fontspec_font[:6], "pdf": os.path.exists(os.path.join(d, "main.pdf"))},
          open(os.path.join(d, "engine.json"), "w"), indent=2)
PY
    echo "-- $name/$engine: exit $rc (${secs}s)"
  done
done
