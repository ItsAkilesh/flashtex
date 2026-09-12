#!/usr/bin/env bash
# Keystroke -> painted v2 frame, helper route AND direct route, before/after
# the page-reuse optimization, interleaved in the same load window.
#
# Drives the real FlashTeXMac (release) with TypingBench (tools/typing-bench
# conventions: typed-200.txt, FLASHTEX_NO_ACTIVATE=1, no UI scripting):
#   helper-v2  flashtex-preview-controller owning flashtex-render,
#              FLASHTEX_PREVIEW_V2=1 FLASHTEX_DISPLAY_CANDIDATES=1
#              (+ FLASHTEX_CONTROLLER_DIAGNOSTIC_TIMINGS=1: helper stage stderr)
#   direct-v2  flashtex-render as FLASHTEX_COMPILER, FLASHTEX_PREVIEW_V2=1
#              (negotiated display-list-v2 sibling, no helper)
# Every cell records `uptime` before/after and waits (bounded) for a 1-minute
# load below LOAD_LIMIT. tools/typing-bench/timeline.py gives the per-stage table.
#
# Usage: FLASHTEX_RENDER=<flashtex-render> FLASHTEX_PREVIEW_CONTROLLER=<helper> \
#        APP_AFTER=<FlashTeXMac release binary> [APP_BEFORE=<binary>] \
#        [SEEDS_DIR=<dir with p3.tex pmax.tex>] [SEEDS="p3 pmax"] [INTERVALS="30 0"] \
#        [ROUTES="helper-v2 direct-v2"] [LOAD_LIMIT=15] [QUIET_WAIT=2400] [OUT=<raw dir>] run.sh
set -euo pipefail
HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT="$(cd "$HERE/../../.." && pwd)"
MAC="$ROOT/apps/mac"
RENDER="${FLASHTEX_RENDER:?set FLASHTEX_RENDER to a built flashtex-render}"
HELPER="${FLASHTEX_PREVIEW_CONTROLLER:?set FLASHTEX_PREVIEW_CONTROLLER to a built helper}"
APP_AFTER="${APP_AFTER:?set APP_AFTER to the release FlashTeXMac of this tree}"
APP_BEFORE="${APP_BEFORE:-}"
LABEL_AFTER="${LABEL_AFTER:-after}"   # build label written into the raw JSON/filenames
LABEL_BEFORE="${LABEL_BEFORE:-before}"
SEEDS_DIR="${SEEDS_DIR:?set SEEDS_DIR (siblings.py output: p3.tex, pmax.tex)}"
SEEDS="${SEEDS:-p3 pmax}"
INTERVALS="${INTERVALS:-30 0}"
ROUTES="${ROUTES:-helper-v2 direct-v2}"
LOAD_LIMIT="${LOAD_LIMIT:-15}"
QUIET_WAIT="${QUIET_WAIT:-2400}"
UTC="$(date -u +%Y-%m-%dT%H%M%SZ)"
OUT="${OUT:-$HERE/raw-$UTC}"; mkdir -p "$OUT"
WORK="$MAC/build/preview-latency/$UTC"; mkdir -p "$WORK"
load1() { sysctl -n vm.loadavg | awk '{print $2}'; }
step() { echo "==> $*"; }

wait_quiet() {
  local waited=0 l other
  while :; do
    l="$(load1)"; other="$( (pgrep -x FlashTeXMac || true) | wc -l | tr -d ' ')"
    if awk -v l="$l" -v q="$LOAD_LIMIT" 'BEGIN { exit !(l < q) }' && (( other == 0 )); then return 0; fi
    if (( waited >= QUIET_WAIT )); then echo "    load $l / $other other FlashTeXMac after $QUIET_WAIT s; running anyway (load-affected)" >&2; return 0; fi
    (( waited % 120 == 0 )) && echo "    load $l (limit $LOAD_LIMIT), $other other FlashTeXMac; waiting (${waited}s of $QUIET_WAIT)"
    sleep 10; waited=$((waited + 10))
  done
}

run_cell() { # build route seed interval app
  local build="$1" route="$2" seed="$3" ms="$4" app="$5"
  local name="$build-$route-$seed-${ms}ms"
  local cell="$WORK/$name"
  mkdir -p "$cell"; cp "$SEEDS_DIR/$seed.tex" "$cell/$seed.tex"
  local log="$cell/app.log" json="$OUT/$name.json"
  # `-u`: the direct route must not see a helper in the environment (the app
  # auto-attaches to FLASHTEX_PREVIEW_CONTROLLER when set).
  local -a extra=()
  case "$route" in
    helper-v2) extra=(FLASHTEX_PREVIEW_CONTROLLER="$HELPER" FLASHTEX_COMPILER="$RENDER" FLASHTEX_CONTROLLER_LEDGER_ROOT="$cell/ledger"
                     FLASHTEX_PREVIEW_V2=1 FLASHTEX_DISPLAY_CANDIDATES=1 FLASHTEX_CONTROLLER_DIAGNOSTIC_TIMINGS=1) ;;
    # Helper compiler-frame cap raised to its 15 MiB maximum: with the 8 MiB
    # default the producer declines the 7.8 MB pmax sibling through this helper build.
    helper-v2cap15) extra=(FLASHTEX_PREVIEW_CONTROLLER="$HELPER" FLASHTEX_COMPILER="$RENDER" FLASHTEX_CONTROLLER_LEDGER_ROOT="$cell/ledger"
                     FLASHTEX_PREVIEW_V2=1 FLASHTEX_DISPLAY_CANDIDATES=1 FLASHTEX_CONTROLLER_DIAGNOSTIC_TIMINGS=1 FLASHTEX_CONTROLLER_MAX_FRAME_BYTES=15728640) ;;
    direct-v2) extra=(-u FLASHTEX_PREVIEW_CONTROLLER -u FLASHTEX_DISPLAY_CANDIDATES FLASHTEX_COMPILER="$RENDER" FLASHTEX_PREVIEW_V2=1) ;;
    *) echo "unknown route $route" >&2; return 1 ;;
  esac
  wait_quiet
  local before; before="$(load1)"
  step "run $name ($(uptime))"
  (
    cd "$MAC"
    env "${extra[@]}" FLASHTEX_REPO="$ROOT" FLASHTEX_AUTOATTACH=1 FLASHTEX_NO_ACTIVATE=1 \
      FLASHTEX_LM_DIR="$MAC/Fonts" FLASHTEX_FONT_DIRS="$MAC/Fonts" FLASHTEX_TFM_DIRS="$MAC/Fonts/texmf/fonts/tfm/public/lm" \
      FLASHTEX_SEED_FILE="$cell/$seed.tex" FLASHTEX_LOG="$log" \
      FLASHTEX_TYPING_BENCH="$ROOT/tools/typing-bench/typed-200.txt" FLASHTEX_TYPING_BENCH_MS="$ms" FLASHTEX_TYPING_BENCH_OUT="$json" \
      FLASHTEX_TYPING_BENCH_SETTLE_MS=60000 FLASHTEX_TYPING_BENCH_MAX_MS=120000 \
      "$app" >/dev/null 2>&1 &
    pid=$!
    for _ in $(seq 1 600); do kill -0 "$pid" 2>/dev/null || break; sleep 0.5; done
    if kill -0 "$pid" 2>/dev/null; then echo "    timed out after 300 s; killing pid $pid (launched by this script)" >&2; kill "$pid" 2>/dev/null || true; fi
    wait "$pid" 2>/dev/null || true
  )
  local after; after="$(load1)"
  cp "$log" "$OUT/$name.log"
  if [[ -f "$json" ]]; then
    python3 - "$json" "$build" "$route" "$seed" "$ms" "$before" "$after" "$LOAD_LIMIT" <<'PY'
import json, sys
path, build, route, seed, ms, before, after, limit = sys.argv[1:9]
d = json.load(open(path, encoding="utf-8"))
d.update(build=build, route=route, seed=seed, load_avg_before=float(before), load_avg_after=float(after), load_limit=float(limit))
d["load_affected"] = max(float(before), float(after)) > float(limit)
json.dump(d, open(path, "w", encoding="utf-8"), indent=2, sort_keys=True)
k = d["keystroke_to_paint_ms"]
print("    keystroke->paint p50 %s p95 %s p99 %s ms over %d painted of %d keystrokes (%d coalesced, %d paints without redraw); load %s -> %s%s" % (
    k.get("p50_ms"), k.get("p95_ms"), k.get("p99_ms"), d["painted"], d["keystrokes"], d["coalesced"], d["paints_without_redraw"], before, after,
    " LOAD-AFFECTED" if d["load_affected"] else ""))
PY
    python3 "$ROOT/tools/typing-bench/timeline.py" "$log" | sed 's/^/    /'
  else
    echo "    no summary written; last log lines:" >&2; tail -5 "$log" | sed 's/^/    /' >&2
  fi
}

for seed in $SEEDS; do
  for ms in $INTERVALS; do
    for route in $ROUTES; do
      [[ -n "$APP_BEFORE" ]] && run_cell "$LABEL_BEFORE" "$route" "$seed" "$ms" "$APP_BEFORE"
      run_cell "$LABEL_AFTER" "$route" "$seed" "$ms" "$APP_AFTER"
    done
  done
done
echo "raw results: $OUT"
