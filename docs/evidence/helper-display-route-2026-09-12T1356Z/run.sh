#!/usr/bin/env bash
# Keystroke -> painted v2 frame THROUGH THE HELPER display-candidate route
# (crates/preview-controller display-forwarding.md, ShellModel+DisplayCandidates.swift).
#
# Drives the real FlashTeXMac (release) with TypingBench (tools/typing-bench
# conventions: typed-200.txt, 30 ms and 0 ms intervals, FLASHTEX_NO_ACTIVATE=1)
# attached to the real `flashtex-preview-controller` owning the real
# `flashtex-render`, with FLASHTEX_PREVIEW_V2=1 + FLASHTEX_DISPLAY_CANDIDATES=1 so
# the recorded paint point is the v2 pane's bitmap blit of the validated
# candidate frame for the typed revision (PageV2View -> TypingBench). A v1
# control cell (same helper/producer, v1 pane, candidates OFF) is recorded for
# context. Every cell records `uptime` before/after; nothing here is an
# isolated-machine claim unless the recorded load says so.
#
# Requires a checkout that has the parent-retained hook lines applied
# (agent/mac-helper-display/route-applied or later integration).
# Usage: FLASHTEX_RENDER=<flashtex-render> [FLASHTEX_PREVIEW_CONTROLLER=<helper>] \
#        [INTERVALS="30 0"] [SEEDS="p3 p27"] run.sh
set -euo pipefail
HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT="$(cd "$HERE/../../.." && pwd)"
MAC="$ROOT/apps/mac"
INTERVALS="${INTERVALS:-30 0}"
SEEDS="${SEEDS:-p3 p27}"
RENDER="${FLASHTEX_RENDER:?set FLASHTEX_RENDER to a built flashtex-render}"
HELPER="${FLASHTEX_PREVIEW_CONTROLLER:-$ROOT/crates/preview-controller/target/release/flashtex-preview-controller}"
[[ -x "$RENDER" && -x "$HELPER" ]] || { echo "producer/helper not executable: $RENDER / $HELPER" >&2; exit 1; }
grep -q "displayCandidates" "$MAC/Sources/FlashTeXMac/ShellModel.swift" || { echo "checkout lacks the display-candidate hook lines (use agent/mac-helper-display/route-applied)" >&2; exit 1; }
UTC="$(date -u +%Y-%m-%dT%H%M%SZ)"
WORK="$MAC/build/helper-display-route/$UTC"; mkdir -p "$WORK"
RAW="$HERE/raw"; mkdir -p "$RAW"
load1() { sysctl -n vm.loadavg | awk '{print $2}'; }
step() { echo "==> $*"; }

step "building FlashTeXMac (release) at $(git -C "$ROOT" rev-parse --short HEAD) ($(git -C "$ROOT" rev-parse --abbrev-ref HEAD))"
swift build -c release --package-path "$MAC" 2>&1 | tail -1
APP="$MAC/.build/release/FlashTeXMac"

step "seeds (page counts probed through the producer)"
python3 "$HERE/seeds.py" "$ROOT" "$WORK" "$RENDER" "$MAC/Fonts" | tee "$WORK/seeds.txt"

run_cell() { # route seed interval
  local route="$1" seed="$2" ms="$3"
  local name="$route-$seed-${ms}ms" cell="$WORK/$route-$seed-${ms}ms"
  mkdir -p "$cell"; cp "$WORK/$seed.tex" "$cell/$seed.tex"
  local log="$cell/app.log" json="$RAW/$name.json"
  local -a extra=(FLASHTEX_PREVIEW_CONTROLLER="$HELPER" FLASHTEX_COMPILER="$RENDER" FLASHTEX_CONTROLLER_LEDGER_ROOT="$cell/ledger")
  case "$route" in
    helper-v2) extra+=(FLASHTEX_PREVIEW_V2=1 FLASHTEX_DISPLAY_CANDIDATES=1) ;;
    helper-v1) ;;
  esac
  local before; before="$(load1)"
  step "run $name (1-min load $before; $(uptime))"
  (
    cd "$MAC"
    env FLASHTEX_REPO="$ROOT" FLASHTEX_AUTOATTACH=1 FLASHTEX_NO_ACTIVATE=1 \
      FLASHTEX_LM_DIR="$MAC/Fonts" FLASHTEX_FONT_DIRS="$MAC/Fonts" \
      FLASHTEX_SEED_FILE="$cell/$seed.tex" FLASHTEX_LOG="$log" \
      FLASHTEX_TYPING_BENCH="$ROOT/tools/typing-bench/typed-200.txt" FLASHTEX_TYPING_BENCH_MS="$ms" FLASHTEX_TYPING_BENCH_OUT="$json" \
      FLASHTEX_TYPING_BENCH_SETTLE_MS=60000 FLASHTEX_TYPING_BENCH_MAX_MS=120000 \
      "${extra[@]}" "$APP" >/dev/null 2>&1 &
    pid=$!
    for _ in $(seq 1 600); do kill -0 "$pid" 2>/dev/null || break; sleep 0.5; done
    if kill -0 "$pid" 2>/dev/null; then echo "    timed out after 300 s; killing pid $pid" >&2; kill "$pid" 2>/dev/null || true; fi
    wait "$pid" 2>/dev/null || true
  )
  local after; after="$(load1)"
  if [[ -f "$json" ]]; then
    python3 - "$json" "$route" "$seed" "$ms" "$before" "$after" "$log" <<'PY'
import json, re, sys
path, route, seed, ms, before, after, log = sys.argv[1:8]
d = json.load(open(path, encoding="utf-8"))
d["route"] = route; d["seed"] = seed
d["load_avg_before"] = float(before); d["load_avg_after"] = float(after)
d["load_note"] = "under shared load, not an isolated result" if max(float(before), float(after)) > 8 else "load < 8 during the cell"
text = open(log, encoding="utf-8", errors="replace").read()
c = lambda pat: len(re.findall(pat, text))
d["candidate_counters"] = {
    "v1_previews_applied": c(r"status: revision \d+: "),
    "candidates_admitted": c(r"display-candidate: admitted "),
    "candidates_painted": c(r"display-candidate: painted "),
    "candidates_refused": c(r"display-candidate: refused "),
    "candidates_invalid": c(r"display-candidate: invalid "),
    "candidates_dropped_at_paint": c(r"display-candidate: dropped "),
    "declined_display_list_v2": c(r"declined layout capabilities.*display-list-v2"),
}
m = re.findall(r"display-candidate: validated \S+ in ([0-9.]+) ms", text)
if m:
    v = sorted(float(x) for x in m)
    d["candidate_validation_ms"] = {"count": len(v), "p50": v[len(v)//2], "p95": v[min(len(v)-1, int(len(v)*0.95))], "max": v[-1]}
json.dump(d, open(path, "w", encoding="utf-8"), indent=2, sort_keys=True)
k = d["keystroke_to_paint_ms"]
print("    keystroke->paint p50 %s p95 %s p99 %s ms over %d painted of %d keystrokes (%d coalesced); %s; %s" % (
    k.get("p50_ms"), k.get("p95_ms"), k.get("p99_ms"), d["painted"], d["keystrokes"], d["coalesced"], d["candidate_counters"], d["load_note"]))
PY
  else
    echo "    no summary written; last log lines:" >&2; tail -5 "$log" | sed 's/^/    /' >&2
  fi
}

for seed in $SEEDS; do
  for ms in $INTERVALS; do run_cell helper-v2 "$seed" "$ms"; done
done
# v1 control on the first seed only (same helper + producer, v1 pane).
first="${SEEDS%% *}"
for ms in $INTERVALS; do run_cell helper-v1 "$first" "$ms"; done

step "writing $HERE/summary.md"
python3 "$HERE/summarize.py" "$HERE/summary.md" "$RAW" "$ROOT" "$UTC" "$HELPER" "$RENDER" "$WORK/seeds.txt"
