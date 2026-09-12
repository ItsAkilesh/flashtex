#!/usr/bin/env bash
# Render every fixture with each available reference TeX engine (oracle only;
# FlashTeX never invokes these), then rasterize with the shared CoreGraphics
# rasterizer at a fixed DPI.
#
# Usage: render_reference.sh --out <dir> --rasterize <bin> [--dpi 144]
#                            [--texbin /Library/TeX/texbin] [--fixtures <dir>]
#                            [--engine pdflatex --engine xelatex ...]
#                            [--reference-from <evidence dir> ...]
#
# Reproducible bytes: every engine runs with SOURCE_DATE_EPOCH=0 FORCE_SOURCE_DATE=1 (pdfTeX,
# XeTeX/xdvipdfmx and LuaTeX then write fixed CreationDate/ModDate and, for pdfTeX/xdvipdfmx,
# a deterministic trailer /ID); lualatex additionally gets `\pdfvariable trailerid{...}` in front
# of \documentclass because LuaTeX's /ID is otherwise random. With this, rendering the same
# fixture twice gives byte-identical PDFs (verified on this machine for all three engines), so
# the established-engine byte pin in harness/oracle-profile.json is a real pin of oracle
# output, not of a normalised copy. The environment and the extra line are recorded in
# engine.json and in the pin.
#
# Missing engines: when an engine binary is not installed, each fixture/variant is
# looked up (newest --reference-from dir first) in <dir>/references/<fixture>/<label>/
# {main.pdf,engine.json}; the stored PDF is reused ONLY when its recorded fixture
# SHA-256 equals the current fixture and its recorded preamble equals the preamble
# this script would use. engine.json then carries `reused_from` (run id, original
# engine version/path/flags) and the raster/word boxes are re-derived from the PDF
# with the shared rasterizer. Otherwise engine.json says `available:false` with a
# `reason` ("reference unavailable ...") and the fixture is reported, not failed.
#
# Output layout: <out>/<fixture>/<engine>/main.tex|main.pdf|main.log|
#                page-p<N>.png|.rgba|.rgba.json|raster.json|engine.json|words.json
# plus <out>/engines.json with versions/availability.
#
# Preambles (everything before \begin{document} in the fixture is replaced).
# Two font variants per engine, written to <fixture>/<engine> and <fixture>/<engine>-lm:
#   times (matches the current FlashTeX compiler's Times metrics):
#     pdflatex : 12pt article, T1 fontenc, times (URW Nimbus Roman clone via
#                psnfss), geometry margin=1in, parindent 0, secnumdepth 0, empty
#                pagestyle -- the native-validation oracle "B" preamble.
#     xelatex/lualatex : same, but fontspec \setmainfont{Times New Roman}.
#   lm (LaTeX's real default look, Latin Modern = Computer Modern outlines; the
#       primary target once the Latin-Modern-metrics render pipeline exists):
#     pdflatex : T1 fontenc + lmodern instead of times.
#     xelatex/lualatex : fontspec Latin Modern Roman loaded by explicit path from
#                the TeX Live tree (lmroman12-*.otf), so no font lookup is involved.
set -euo pipefail
HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
OUT=""; RASTERIZE=""; DPI=144; TEXBIN=/Library/TeX/texbin; FIXTURES="$HERE/fixtures"
ENGINES=(); REF_FROM=()
while [[ $# -gt 0 ]]; do
  case "$1" in
    --out) OUT="$2"; shift 2 ;;
    --rasterize) RASTERIZE="$2"; shift 2 ;;
    --dpi) DPI="$2"; shift 2 ;;
    --texbin) TEXBIN="$2"; shift 2 ;;
    --fixtures) FIXTURES="$2"; shift 2 ;;
    --engine) ENGINES+=("$2"); shift 2 ;;
    --reference-from) REF_FROM+=("$2"); shift 2 ;;
    -h|--help) sed -n '2,30p' "$0"; exit 0 ;;
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
LM_DIR="$(dirname "$("$TEXBIN/kpsewhich" -var-value TEXMFDIST 2>/dev/null || echo /usr/local/texlive/2026basic/texmf-dist)")/texmf-dist/fonts/opentype/public/lm/"
[[ -d "$LM_DIR" ]] || LM_DIR="/usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm/"
PRE_PDFLATEX_LM='\documentclass[12pt]{article}
\usepackage[T1]{fontenc}
\usepackage{lmodern}
\usepackage[margin=1in]{geometry}
\setlength{\parindent}{0pt}
\setcounter{secnumdepth}{0}
\pagestyle{empty}
'
PRE_FONTSPEC_LM="\\documentclass[12pt]{article}
\\usepackage{fontspec}
\\setmainfont{lmroman12-regular.otf}[Path=$LM_DIR,BoldFont=lmroman12-bold.otf,ItalicFont=lmroman12-italic.otf,BoldItalicFont=lmroman10-bolditalic.otf]
\\usepackage[margin=1in]{geometry}
\\setlength{\\parindent}{0pt}
\\setcounter{secnumdepth}{0}
\\pagestyle{empty}
"
FLAGS=(-interaction=batchmode -halt-on-error -file-line-error)
ENGINE_ENV=(SOURCE_DATE_EPOCH=0 FORCE_SOURCE_DATE=1)
LUA_TRAILER='\pdfvariable trailerid{[<00000000000000000000000000000000> <00000000000000000000000000000000>]}
'

# Engine availability and versions, recorded honestly.
REF_FROM_ARG="$(IFS=:; echo "${REF_FROM[*]:-}")"
python3 - "$OUT/engines.json" "$TEXBIN" "$REF_FROM_ARG" "${ENGINES[@]}" <<'PY'
import json, os, subprocess, sys
out, texbin, ref_from, engines = sys.argv[1], sys.argv[2], sys.argv[3], sys.argv[4:]
stores = []
for d in [x for x in ref_from.split(":") if x]:
    m = os.path.join(d, "references", "manifest.json")
    if os.path.exists(m):
        j = json.load(open(m))
        stores.append({"dir": d, "run": j.get("run"), "engines": j.get("engines", {}), "entries": len(j.get("entries", {}))})
info = {}
for e in engines:
    path = os.path.join(texbin, e)
    row = {"path": path, "available": os.access(path, os.X_OK)}
    if row["available"]:
        v = subprocess.run([path, "--version"], capture_output=True, text=True).stdout.splitlines()
        row["version"] = v[0] if v else "?"
    else:
        row["reason"] = "engine binary not installed at " + path
        src = next((s for s in stores if s["engines"].get(e, {}).get("available")), None)
        if src:
            row["reused_from"] = {"run": src["run"], "dir": src["dir"], "version": src["engines"][e].get("version"),
                                  "path": src["engines"][e].get("path")}
            row["version"] = src["engines"][e].get("version")  # the version that produced the reused PDFs
    info[e] = row
info["_reference_stores"] = stores
kp = os.path.join(texbin, "kpsewhich")
pk = {}
if os.access(kp, os.X_OK):
    for sty in ["times.sty", "lmodern.sty", "fontspec.sty", "geometry.sty", "t1enc.def"]:
        r = subprocess.run([kp, sty], capture_output=True, text=True).stdout.strip()
        pk[sty] = r or None
    r = subprocess.run([kp, "-var-value", "SELFAUTOPARENT"], capture_output=True, text=True).stdout.strip()
    pk["texlive_root"] = r
    pk["texbin_realpath"] = os.path.realpath(texbin)
    tl = os.path.join(texbin, "tlmgr")
    if os.access(tl, os.X_OK):
        v = subprocess.run([tl, "--version"], capture_output=True, text=True).stdout.splitlines()
        pk["tlmgr"] = next((l for l in v if "tlmgr" in l or "revision" in l), v[0] if v else "?")
    pk["distribution"] = ("BasicTeX (TeX Live " + r.rsplit("/", 1)[-1] + ")" if r.endswith("basic")
                          else "MacTeX / TeX Live full (" + r + ")" if "/texlive/" in r else "TeX Live at " + r)
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
  for variant in times lm; do
    bin="$TEXBIN/$engine"
    label="$engine"; [[ "$variant" == "lm" ]] && label="$engine-lm"
    dir="$OUT/$name/$label"
    mkdir -p "$dir"
    case "$engine-$variant" in
      pdflatex-times) pre="$PRE_PDFLATEX" ;;
      pdflatex-lm) pre="$PRE_PDFLATEX_LM" ;;
      *-times) pre="$PRE_FONTSPEC" ;;
      *) pre="$PRE_FONTSPEC_LM" ;;
    esac
    [[ "$engine" == lualatex ]] && pre="$LUA_TRAILER$pre"
    if [[ ! -x "$bin" ]]; then
      # Reuse a stored reference PDF (newest store first) when fixture SHA and preamble match.
      reused="$(python3 - "$tex" "$dir" "$engine" "$variant" "$label" "$pre" "$REF_FROM_ARG" <<'PY'
import hashlib, json, os, re, shutil, sys
tex, d, engine, variant, label, pre, ref_from = sys.argv[1:8]
fx = os.path.basename(tex)[:-4]
fx_sha = hashlib.sha256(open(tex, "rb").read()).hexdigest()
tried = []
for store in [x for x in ref_from.split(":") if x]:
    src = os.path.join(store, "references", fx, label)
    ej, pdf = os.path.join(src, "engine.json"), os.path.join(src, "main.pdf")
    if not (os.path.exists(ej) and os.path.exists(pdf)):
        tried.append(f"{store}: no stored reference"); continue
    e = json.load(open(ej))
    if e.get("fixture_sha256") != fx_sha:
        tried.append(f"{store}: fixture changed (stored {str(e.get('fixture_sha256'))[:12]}, now {fx_sha[:12]})"); continue
    norm = lambda t: re.sub(r"Path=[^,\]]+", "Path=<lm-dir>", t or "")  # same OTF files, TeX Live tree may move
    if norm(e.get("preamble")) != norm(pre):
        tried.append(f"{store}: preamble differs"); continue
    if e.get("exit") != 0:
        tried.append(f"{store}: stored render exited {e.get('exit')}"); continue
    if hashlib.sha256(open(pdf, "rb").read()).hexdigest() != e.get("pdf_sha256"):
        tried.append(f"{store}: stored PDF SHA-256 mismatch"); continue
    shutil.copy2(pdf, os.path.join(d, "main.pdf"))
    shutil.copy2(ej, os.path.join(d, "stored-engine.json"))
    out = {"engine": label, "variant": variant, "available": True, "reused": True,
           "reused_from": {"run": e.get("rendered_in_run"), "dir": store, "engine_version": e.get("engine_version"),
                           "path": e.get("path"), "flags": e.get("flags"), "seconds": e.get("seconds")},
           "path": None, "exit": 0, "seconds": 0.0, "flags": e.get("flags"), "env": e.get("env"), "preamble": pre,
           "errors": e.get("errors", []), "warnings": e.get("warnings", []), "fontspec_fonts": e.get("fontspec_fonts", []),
           "fixture_sha256": fx_sha, "pdf_sha256": e.get("pdf_sha256"), "pdf": True,
           "note": "engine binary not installed this run; PDF reused from run %s (%s) after fixture SHA-256 and preamble check" % (e.get("rendered_in_run"), e.get("engine_version"))}
    json.dump(out, open(os.path.join(d, "engine.json"), "w"), indent=2)
    print("reused " + str(e.get("rendered_in_run")))
    sys.exit(0)
out = {"engine": label, "variant": variant, "available": False, "reused": False, "fixture_sha256": fx_sha, "preamble": pre,
       "reason": "reference unavailable: engine binary not installed and no matching stored reference (" + ("; ".join(tried) or "no --reference-from store") + ")"}
json.dump(out, open(os.path.join(d, "engine.json"), "w"), indent=2)
print("unavailable")
PY
)"
      if [[ "$reused" == reused* ]]; then
        "$RASTERIZE" pdf "$dir/main.pdf" "$dir/page" --dpi "$DPI" > "$dir/raster.json"
        "$RASTERIZE" word-boxes "$dir/main.pdf" > "$dir/words.json"
        echo "-- $name/$label: engine not installed; $reused (raster re-derived)"
      else
        echo "-- $name/$label: reference unavailable (engine not installed, no stored reference)"
      fi
      continue
    fi
    python3 - "$tex" "$dir/main.tex" "$pre" <<'PY'
import re, sys
src = open(sys.argv[1], encoding="utf-8").read()
m = re.search(r"^[ \t]*\\begin\{document\}", src, re.M)
body = src[m.start():] if m else src
open(sys.argv[2], "w", encoding="utf-8").write(sys.argv[3] + body)
PY
    t0=$(python3 -c 'import time;print(time.time())')
    set +e
    ( cd "$dir" && env "${ENGINE_ENV[@]}" "$bin" "${FLAGS[@]}" main.tex >/dev/null 2>&1 )
    rc=$?
    set -e
    secs=$(python3 -c "import time;print(round(time.time()-$t0,3))")
    if [[ $rc -eq 0 && -f "$dir/main.pdf" ]]; then
      "$RASTERIZE" pdf "$dir/main.pdf" "$dir/page" --dpi "$DPI" > "$dir/raster.json"
      "$RASTERIZE" word-boxes "$dir/main.pdf" > "$dir/words.json"
    fi
    python3 - "$dir" "$label" "$bin" "$rc" "$secs" "${FLAGS[*]}" "$pre" "$variant" "$tex" "${ENGINE_ENV[*]}" <<'PY'
import json, re, sys, os
d, engine, path, rc, secs, flags, pre, variant = sys.argv[1:9]
engine_env = sys.argv[10].split()
log = open(os.path.join(d, "main.log"), encoding="latin-1").read() if os.path.exists(os.path.join(d, "main.log")) else ""
fonts = sorted(set(re.findall(r"[\w/.+\-]+\.(?:pfb|otf|ttf|ttc|pfa)", log.replace("\n", ""))))[:40]  # font files the engine reported loading
errors = [l for l in log.splitlines() if l.startswith("!")][:5]
warns = [l for l in log.splitlines() if "Warning" in l][:10]
fontspec_font = re.findall(r"Font\s+'([^']+)'\s+\(([^)]+)\)", log)
import hashlib
pdfp = os.path.join(d, "main.pdf")
json.dump({"engine": engine, "variant": variant, "available": True, "reused": False, "path": path, "exit": int(rc), "seconds": float(secs),
           "flags": flags.split(), "env": engine_env, "preamble": pre, "errors": errors, "warnings": warns,
           "fonts_in_log": fonts,
           "fontspec_fonts": fontspec_font[:6], "pdf": os.path.exists(pdfp),
           "fixture_sha256": hashlib.sha256(open(sys.argv[9], "rb").read()).hexdigest(),
           "pdf_sha256": hashlib.sha256(open(pdfp, "rb").read()).hexdigest() if os.path.exists(pdfp) else None},
          open(os.path.join(d, "engine.json"), "w"), indent=2)
PY
    echo "-- $name/$label: exit $rc (${secs}s)"
  done
  done
done
