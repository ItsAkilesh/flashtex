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
#                           [--exact <label>=<flashtex-render>:<flashtex-pdf-exact> ...]
#                           [--font-dir <dir> ...] [--reference <reference work dir>]
#                           [--dpi 144] [--fixtures <dir>] [--embed-font auto]
#
# --exact labels take the EXACT route instead of compile_result -> flashtex-pdf:
#   flashtex-render --secnumdepth 0 --v2 display_list.json   (rendering-v2 envelope; the
#   runtime-v1 compile_result still goes to stdout, so the preview-equivalent raster exists)
#   flashtex-pdf-exact from-v2 display_list.json --out flashtex.pdf [--font-dir ...]
#   (glyphs by original GID at exact tick origins, GID-preserving CFF subsets of the Latin
#   Modern OTFs found by content hash). Its stdout/stderr notes (including the sha256
#   deviation: the producer's font hash is SHA-256(bytes || face_index)) go to build.json.
#   When --reference is given and <reference>/<fixture>/pdflatex-lm/main.pdf exists,
#   `flashtex-pdf-exact classify REF OUT` runs and its categories go to build.json too.
#
# Output layout: <out>/<fixture>/<label>/compile_result.json|flashtex.pdf|
#                export-p<N>.png|.rgba|.rgba.json|preview-p<N>.png|.rgba|.rgba.json|
#                build.json|words.json|rules.json
set -euo pipefail
HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
OUT=""; RASTERIZE=""; PDFBIN=""; DPI=144; FIXTURES="$HERE/fixtures"; EMBED=""; REFERENCE=""
COMPILERS=(); EXACT=(); FONT_DIRS=()
while [[ $# -gt 0 ]]; do
  case "$1" in
    --out) OUT="$2"; shift 2 ;;
    --rasterize) RASTERIZE="$2"; shift 2 ;;
    --pdf-bin) PDFBIN="$2"; shift 2 ;;
    --compiler) COMPILERS+=("$2"); shift 2 ;;
    --dpi) DPI="$2"; shift 2 ;;
    --fixtures) FIXTURES="$2"; shift 2 ;;
    --embed-font) EMBED="$2"; shift 2 ;;
    --exact) EXACT+=("$2"); shift 2 ;;
    --font-dir) FONT_DIRS+=("$2"); shift 2 ;;
    --reference) REFERENCE="$2"; shift 2 ;;
    -h|--help) sed -n '2,28p' "$0"; exit 0 ;;
    *) echo "unknown argument: $1" >&2; exit 2 ;;
  esac
done
[[ -n "$OUT" && -n "$RASTERIZE" && -n "$PDFBIN" && ${#COMPILERS[@]} -gt 0 ]] || {
  echo "--out, --rasterize, --pdf-bin and at least one --compiler are required" >&2; exit 2; }
mkdir -p "$OUT"
FONT_ARGS=(); for fd in "${FONT_DIRS[@]}"; do FONT_ARGS+=(--font-dir "$fd"); done

# --- exact route
for tex in "$FIXTURES"/*.tex; do
  name="$(basename "$tex" .tex)"
  for spec in "${EXACT[@]}"; do
    label="${spec%%=*}"; rest="${spec#*=}"; rbin="${rest%%:*}"; xbin="${rest#*:}"
    dir="$OUT/$name/$label"
    mkdir -p "$dir"
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
    "$rbin" --secnumdepth 0 --v2 "$dir/display_list.json" < "$dir/request.jsonl" > "$dir/compile_result.json" 2> "$dir/compiler.stderr"
    crc=$?
    "$xbin" from-v2 "$dir/display_list.json" --out "$dir/flashtex.pdf" "${FONT_ARGS[@]}" > "$dir/from-v2.stdout" 2> "$dir/pdf.stderr"
    prc=$?
    cls=""; clsrc=""
    if [[ -n "$REFERENCE" && -f "$REFERENCE/$name/pdflatex-lm/main.pdf" && -f "$dir/flashtex.pdf" ]]; then
      "$xbin" classify "$REFERENCE/$name/pdflatex-lm/main.pdf" "$dir/flashtex.pdf" > "$dir/classify.txt" 2>&1
      clsrc=$?; cls="$dir/classify.txt"
    fi
    set -e
    if [[ $prc -eq 0 && -f "$dir/flashtex.pdf" ]]; then
      "$RASTERIZE" pdf "$dir/flashtex.pdf" "$dir/export" --dpi "$DPI" > "$dir/export-raster.json"
      "$RASTERIZE" word-boxes "$dir/flashtex.pdf" > "$dir/words.json"
    fi
    if [[ $crc -eq 0 ]]; then
      "$RASTERIZE" preview "$dir/compile_result.json" "$dir/preview" --dpi "$DPI" > "$dir/preview-raster.json" || true
    fi
    python3 - "$dir" "$label" "$rbin" "$xbin" "$crc" "$prc" "$cls" "$clsrc" "${FONT_DIRS[*]:-}" <<'PY'
import json, os, re, sys
d, label, rbin, xbin, crc, prc, cls, clsrc, fdirs = sys.argv[1:10]
def rd(f):
    p = os.path.join(d, f)
    return open(p, errors="replace").read() if os.path.exists(p) else ""
err = rd("pdf.stderr")
info = {"compiler": label, "route": "exact", "compiler_path": rbin, "compile_exit": int(crc), "pdf_exit": int(prc),
        "pdf_bin": xbin, "pdf_flags": ["from-v2"] + (["--font-dir"] + fdirs.split() if fdirs else []),
        "render_flags": ["--secnumdepth", "0", "--v2"],
        "pdf_stderr": err.strip()[:1500], "from_v2_stdout": rd("from-v2.stdout").strip()[:800],
        "from_v2_summary": next((l.split("note: ", 1)[1].split(" -> ")[0] for l in err.splitlines() if "page(s)" in l and "glyph run" in l), None),
        "deviations": [l.split("note: ", 1)[-1] for l in err.splitlines() if "deviation" in l],
        "fonts_embedded": [l.split("note: ", 1)[-1] for l in err.splitlines() if "glyph(s)," in l and "program" in l]}
if cls:
    txt = rd("classify.txt")
    info["classify_exit"] = int(clsrc) if clsrc else None
    cats = next((l.split("categories:", 1)[1].strip() for l in txt.splitlines() if l.startswith("categories:")), None)
    info["classify_categories"] = [c.strip() for c in cats.split(",")] if cats else ([] if "categories:" in txt else None)
    info["classify_same"] = [l for l in txt.splitlines() if l.startswith("[same]")]
    info["classify_reference"] = "pdflatex-lm"
try:
    env = json.load(open(os.path.join(d, "compile_result.json"), encoding="utf-8"))
    pl = env["payload"]
    info["status"] = pl.get("status")
    info["diagnostics"] = [{"severity": x.get("severity"), "message": x.get("message")} for x in pl.get("diagnostics", [])][:25]
    info["diagnostic_count"] = len(pl.get("diagnostics", []))
    info["pages"] = [{"number": p["number"], "width_pt": p["width_pt"], "height_pt": p["height_pt"], "items": len(p["items"])} for p in pl["pages"]]
except Exception as exc:
    info["status"] = f"unparseable: {exc}"
try:
    dl = json.load(open(os.path.join(d, "display_list.json"), encoding="utf-8"))
    pay = dl.get("payload", dl)
    info["display_list"] = {"schema": dl.get("schema") or pay.get("schema"), "pages": len(pay.get("pages", [])) if isinstance(pay.get("pages"), list) else None,
                            "bytes": os.path.getsize(os.path.join(d, "display_list.json"))}
except Exception as exc:
    info["display_list"] = {"error": str(exc)[:200]}
pdf = os.path.join(d, "flashtex.pdf")
if os.path.exists(pdf):
    import zlib
    data = open(pdf, "rb").read()
    rects = []
    for m in re.finditer(rb"stream\r?\n(.*?)\r?\nendstream", data, re.S):
        raw = m.group(1)
        try: raw = zlib.decompress(raw)
        except Exception: pass
        rects += [tuple(float(v) for v in r) for r in re.findall(rb"([-\d.]+) ([-\d.]+) ([-\d.]+) ([-\d.]+) re\s*f", raw)]
    info["pdf_re_f_rects"] = rects[:50]; info["pdf_re_f_count"] = len(rects)
json.dump(info, open(os.path.join(d, "build.json"), "w"), indent=2)
PY
    echo "-- $name/$label (exact route): render exit $crc, from-v2 exit $prc, classify ${clsrc:-skipped}"
  done
done

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
