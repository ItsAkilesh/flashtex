#!/usr/bin/env bash
# Capture the ACTUAL native FlashTeXMac preview (SwiftUI Canvas) for each fixture
# and compiler build, without Accessibility: launch the app with the fixture body
# seeded and the compiler auto-attached, wait for the FLASHTEX_LOG status line,
# find the window with CGWindowList, `screencapture -l <id>`, detect the white
# page rectangle in the capture, resample it to the reference raster size, and
# write <out>/<fixture>/<compiler>/native-p1.{png,rgba,rgba.json} for diff.py.
#
# Requirements: Screen Recording permission for the terminal; the app built from
# origin/agent/mac-claude-a/mac-shell (`cd apps/mac && swift build`). The page is
# fitted to the preview pane width, so the capture is UPSAMPLED or DOWNSAMPLED to
# the 144-DPI raster size; the scale and the display's backing factor are recorded
# in capture.json and the report (on a 1x external display the page is ~434 px wide).
# Only the first page is captured (the pane shows page 1 at launch).
#
# Usage: capture_native.sh --out <dir> --app <FlashTeXMac binary> --repo <repo root>
#                          --rasterize <bin> --compiler <label>=<path> [--compiler ...]
#                          [--fixtures <dir>] [--dpi 144] [--fixture <name>]
set -euo pipefail
HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
OUT=""; APP=""; REPO=""; RASTERIZE=""; FIXTURES="$HERE/fixtures"; DPI=144; ONLY=""
COMPILERS=()
while [[ $# -gt 0 ]]; do
  case "$1" in
    --out) OUT="$2"; shift 2 ;;
    --app) APP="$2"; shift 2 ;;
    --repo) REPO="$2"; shift 2 ;;
    --rasterize) RASTERIZE="$2"; shift 2 ;;
    --compiler) COMPILERS+=("$2"); shift 2 ;;
    --fixtures) FIXTURES="$2"; shift 2 ;;
    --dpi) DPI="$2"; shift 2 ;;
    --fixture) ONLY="$2"; shift 2 ;;
    -h|--help) sed -n '2,18p' "$0"; exit 0 ;;
    *) echo "unknown argument: $1" >&2; exit 2 ;;
  esac
done
[[ -n "$OUT" && -x "$APP" && -n "$REPO" && -x "$RASTERIZE" && ${#COMPILERS[@]} -gt 0 ]] || {
  echo "--out, --app, --repo, --rasterize and --compiler are required" >&2; exit 2; }
mkdir -p "$OUT"
APP_NAME="$(basename "$APP")"

# Note: the window opens wherever macOS places it (this build ignores a pre-set
# "NSWindow Frame" default), so the page is captured at whatever fit-to-width
# scale the pane gives on that display and then resampled; the factor is recorded.
PW=$(python3 -c "print(int(round(612*$DPI/72)))"); PH=$(python3 -c "print(int(round(792*$DPI/72)))")
SCREEN="$(system_profiler SPDisplaysDataType 2>/dev/null | grep -E 'Resolution|UI Looks like' | head -2 | tr -s ' ' | tr '\n' ';')"
RESULTS="["

wait_for() {  # file pattern timeout_s
  local waited=0
  while (( waited < $3 * 2 )); do
    [[ -f "$1" ]] && grep -qE "$2" "$1" 2>/dev/null && return 0
    sleep 0.5; waited=$((waited + 1))
  done
  return 1
}

for tex in "$FIXTURES"/*.tex; do
  name="$(basename "$tex" .tex)"
  [[ -z "$ONLY" || "$name" == "$ONLY" ]] || continue
  for spec in "${COMPILERS[@]}"; do
    label="${spec%%=*}"; bin="${spec#*=}"
    dir="$OUT/$name/$label"; mkdir -p "$dir"
    python3 - "$tex" "$dir/seed.tex" <<'PY'
import re, sys
src = open(sys.argv[1], encoding="utf-8").read()
m = re.search(r"^[ \t]*\\begin\{document\}", src, re.M)
open(sys.argv[2], "w", encoding="utf-8").write(src[m.start():] if m else src)
PY
    set +e
    for attempt in 1 2; do
      log="$dir/app.log"; : > "$log"
      pkill -x "$APP_NAME" >/dev/null 2>&1
      sleep 0.5
      FLASHTEX_REPO="$REPO" FLASHTEX_COMPILER="$bin" FLASHTEX_AUTOATTACH=1 FLASHTEX_SEED_FILE="$dir/seed.tex" \
        FLASHTEX_LOG="$log" "$APP" > "$dir/app.stdout" 2>&1 &
      app_pid=$!
      status="timeout"
      if wait_for "$log" 'status: revision [0-9]+: (ok|recovered|failed)' 30; then
        status="$(grep -E 'status: revision [0-9]+:' "$log" | tail -1 | cut -f2-)"
      fi
      sleep 1.5   # let SwiftUI lay out and draw the page
      win="$("$RASTERIZE" window-id "$APP_NAME" 2>/dev/null || echo '[]')"
      echo "$win" > "$dir/window.json"
      wid="$(python3 -c 'import json,sys; w=[x for x in json.load(sys.stdin) if x["layer"]==0]; print(w[0]["id"] if w else "")' <<<"$win" 2>/dev/null)"
      capture_ok=0
      if [[ -n "$wid" ]]; then
        screencapture -x -o -l "$wid" "$dir/window.png" && capture_ok=1
      fi
      kill "$app_pid" >/dev/null 2>&1
      wait "$app_pid" 2>/dev/null
      page="{}"
      if [[ $capture_ok -eq 1 ]]; then
        page="$("$RASTERIZE" find-page "$dir/window.png" 2>/dev/null || echo '{}')"
        echo "$page" > "$dir/page-rect.json"
        rect="$(python3 -c 'import json,sys; p=json.load(sys.stdin); print(" ".join(str(p[k]) for k in ("x","y","w","h")) if p.get("found") else "")' <<<"$page" 2>/dev/null)"
        if [[ -n "$rect" ]]; then
          # shellcheck disable=SC2086
          "$RASTERIZE" crop-scale "$dir/window.png" "$dir/native" --page $rect --width "$PW" --height "$PH" --mask-caption > "$dir/crop.json"
        fi
      fi
      [[ -f "$dir/native-p1.rgba" ]] && break
      echo "   retry $attempt for $name/$label (no raster)"
    done
    set -e
    entry="$(python3 - "$name" "$label" "$status" "$wid" "$capture_ok" "$dir" <<'PY'
import json, os, sys
name, label, status, wid, cap, d = sys.argv[1:7]
e = {"fixture": name, "compiler": label, "app_status": status, "window_id": wid, "captured": cap == "1"}
for f in ("page-rect.json", "native-p1.rgba.json", "window.json"):
    p = os.path.join(d, f)
    if os.path.exists(p):
        try: e[{"page-rect.json": "page_rect", "native-p1.rgba.json": "crop", "window.json": "window"}[f]] = json.load(open(p))
        except Exception as exc: e[f] = f"unparseable: {exc}"
e["native_raster"] = os.path.exists(os.path.join(d, "native-p1.rgba"))
print(json.dumps(e))
PY
)"
    RESULTS+="$entry,"
    echo "-- $name/$label: app '$status', window '$wid', captured=$capture_ok, raster=$(test -f "$dir/native-p1.rgba" && echo yes || echo no)"
  done
done
RESULTS="${RESULTS%,}]"
python3 - "$OUT/capture.json" "$APP" "$SCREEN" "$DPI" "$RESULTS" <<'PY'
import json, subprocess, sys
out, app, screen, dpi, results = sys.argv[1:6]
entries = json.loads(results)
scales = [e["crop"]["scale_x"] for e in entries if isinstance(e.get("crop"), dict) and "scale_x" in e["crop"]]
backing = sorted({round(e["page_rect"]["image_px"][0] / e["window"][0]["bounds"]["Width"], 2)
                  for e in entries if isinstance(e.get("page_rect"), dict) and e.get("window") and "image_px" in e["page_rect"]})
json.dump({"method": "screencapture -x -o -l <CGWindowList id> of the running FlashTeXMac window; page rectangle "
                     "detected as the largest white region bounded by the pane background; resampled with CoreGraphics high-quality "
                     "interpolation to the 144-DPI raster size; the 'page N' caption region (bottom-right of the page) is masked white "
                     "on the capture only",
           "app": app, "display": screen, "dpi": float(dpi), "backing_scale_factors": backing,
           "resample_scale_min": min(scales) if scales else None, "resample_scale_max": max(scales) if scales else None,
           "entries": entries}, open(out, "w"), indent=1)
print(f"capture.json: {len(entries)} entries, resample scale {min(scales) if scales else None}..{max(scales) if scales else None}")
PY
