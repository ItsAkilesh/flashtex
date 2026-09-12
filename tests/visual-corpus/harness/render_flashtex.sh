#!/usr/bin/env bash
# Compile every fixture (preamble stripped: body from \begin{document}) with each
# FlashTeX compiler build, write the runtime-v1 compile_result, convert it to a
# PDF with flashtex-pdf (--verify), rasterize that PDF with the shared
# CoreGraphics rasterizer ("export" side), and also produce the
# "preview-equivalent" raster directly from the compile_result (CoreText
# Times-Roman at the reported x/baseline; U+2500 runs as rules).
#
# Usage: render_flashtex.sh --out <dir> --rasterize <bin> --pdf-bin <flashtex-pdf>
#                           --compiler <label>=<path> [--compiler ...]
#                           [--dpi 144] [--fixtures <dir>] [--embed-font auto]
#
# Output layout: <out>/<fixture>/<label>/compile_result.json|flashtex.pdf|
#                export-p<N>.png|.rgba|.rgba.json|preview-p<N>.png|.rgba|.rgba.json|
#                build.json|words.json|rules.json
set -euo pipefail
HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
OUT=""; RASTERIZE=""; PDFBIN=""; DPI=144; FIXTURES="$HERE/fixtures"; EMBED=""
COMPILERS=()
while [[ $# -gt 0 ]]; do
  case "$1" in
    --out) OUT="$2"; shift 2 ;;
    --rasterize) RASTERIZE="$2"; shift 2 ;;
    --pdf-bin) PDFBIN="$2"; shift 2 ;;
    --compiler) COMPILERS+=("$2"); shift 2 ;;
    --dpi) DPI="$2"; shift 2 ;;
    --fixtures) FIXTURES="$2"; shift 2 ;;
    --embed-font) EMBED="$2"; shift 2 ;;
    -h|--help) sed -n '2,16p' "$0"; exit 0 ;;
    *) echo "unknown argument: $1" >&2; exit 2 ;;
  esac
done
[[ -n "$OUT" && -n "$RASTERIZE" && -n "$PDFBIN" && ${#COMPILERS[@]} -gt 0 ]] || {
  echo "--out, --rasterize, --pdf-bin and at least one --compiler are required" >&2; exit 2; }
mkdir -p "$OUT"

for tex in "$FIXTURES"/*.tex; do
  name="$(basename "$tex" .tex)"
  for spec in "${COMPILERS[@]}"; do
    label="${spec%%=*}"; bin="${spec#*=}"
    dir="$OUT/$name/$label"
    mkdir -p "$dir"
    # Request envelope: body only, as a single runtime-v1 compile line.
    python3 - "$tex" "$dir/request.jsonl" "$name" <<'PY'
import json, re, sys
src = open(sys.argv[1], encoding="utf-8").read()
m = re.search(r"^[ \t]*\\begin\{document\}", src, re.M)
body = src[m.start():] if m else src
req = {"protocol_version": 1, "id": "visual-" + sys.argv[3], "type": "compile",
       "payload": {"project_id": "visual-corpus", "revision": 1, "entry_path": "main.tex",
                   "documents": [{"path": "main.tex", "text": body}]}}
open(sys.argv[2], "w", encoding="utf-8").write(json.dumps(req, ensure_ascii=False) + "\n")
PY
    set +e
    "$bin" < "$dir/request.jsonl" > "$dir/compile_result.json" 2> "$dir/compiler.stderr"
    crc=$?
    embed_args=()
    [[ -n "$EMBED" ]] && embed_args=(--embed-font "$EMBED")
    "$PDFBIN" "$dir/compile_result.json" --out "$dir/flashtex.pdf" --verify "${embed_args[@]}" 2> "$dir/pdf.stderr"
    prc=$?
    set -e
    if [[ $prc -eq 0 && -f "$dir/flashtex.pdf" ]]; then
      "$RASTERIZE" pdf "$dir/flashtex.pdf" "$dir/export" --dpi "$DPI" > "$dir/export-raster.json"
      "$RASTERIZE" word-boxes "$dir/flashtex.pdf" > "$dir/words.json"
    fi
    if [[ $crc -eq 0 ]]; then
      "$RASTERIZE" preview "$dir/compile_result.json" "$dir/preview" --dpi "$DPI" > "$dir/preview-raster.json" || true
    fi
    python3 - "$dir" "$label" "$bin" "$crc" "$prc" "$PDFBIN" "$EMBED" <<'PY'
import json, re, sys, os
d, label, cbin, crc, prc, pdfbin, embed = sys.argv[1:8]
info = {"compiler": label, "compiler_path": cbin, "compile_exit": int(crc), "pdf_exit": int(prc),
        "pdf_bin": pdfbin, "pdf_flags": ["--verify"] + (["--embed-font", embed] if embed else []),
        "pdf_stderr": open(os.path.join(d, "pdf.stderr"), errors="replace").read().strip()[:800]}
try:
    env = json.load(open(os.path.join(d, "compile_result.json"), encoding="utf-8"))
    pl = env["payload"]
    info["status"] = pl.get("status")
    info["diagnostics"] = [{"severity": x.get("severity"), "message": x.get("message")} for x in pl.get("diagnostics", [])][:25]
    info["diagnostic_count"] = len(pl.get("diagnostics", []))
    info["pages"] = [{"number": p["number"], "width_pt": p["width_pt"], "height_pt": p["height_pt"], "items": len(p["items"])} for p in pl["pages"]]
    # Rule geometry straight from the compile_result (the U+2500 convention).
    rules = []
    for p in pl["pages"]:
        for it in p["items"]:
            t = it.get("text", "")
            if t and set(t) == {"─"}:
                s = it["font_size_pt"]
                rules.append({"page": p["number"], "x_pt": it["x_pt"], "y_pt": it["baseline_y_pt"] - 0.0857 * s,
                              "width_pt": len(t) * 0.5 * s, "height_pt": 0.0857 * s})
    json.dump(rules, open(os.path.join(d, "rules.json"), "w"), indent=1)
    info["rule_items"] = len(rules)
except Exception as exc:
    info["status"] = f"unparseable: {exc}"
# `re f` rectangles actually written into the FlashTeX PDF content streams.
pdf = os.path.join(d, "flashtex.pdf")
if os.path.exists(pdf):
    import zlib
    data = open(pdf, "rb").read()
    rects = []
    for m in re.finditer(rb"stream\r?\n(.*?)\r?\nendstream", data, re.S):
        raw = m.group(1)
        try:
            raw = zlib.decompress(raw)
        except Exception:
            pass
        rects += [tuple(float(v) for v in r) for r in re.findall(rb"([-\d.]+) ([-\d.]+) ([-\d.]+) ([-\d.]+) re\s*f", raw)]
    info["pdf_re_f_rects"] = rects[:50]
    info["pdf_re_f_count"] = len(rects)
json.dump(info, open(os.path.join(d, "build.json"), "w"), indent=2)
PY
    echo "-- $name/$label: compile exit $crc, pdf exit $prc"
  done
done
