#!/usr/bin/env bash
# Measures keystroke -> paint latency of the FlashTeX Mac shell by typing a
# fixed script into the real editor (TypingBench.swift, FLASHTEX_TYPING_BENCH)
# for three seed documents, two typing intervals, and every available producer
# (main's flashtex-compiler; flashtex-render from the render-pipeline branch
# when it builds). Writes docs/evidence/typing-bench-<UTC>.md plus the raw JSON
# summaries next to it.
#
# Usage: tools/typing-bench/run.sh [--intervals "30 0"] [--seeds "demo body60k fixture"]
#                                  [--producers "compiler render"]
#                                  [--no-render] [--out <evidence.md>]
# Env:   FLASHTEX_RENDER=<path>  use an already built flashtex-render
#        FLASHTEX_RENDER_REF=<git ref>  branch to build it from
#                                        (default origin/agent/mac-render-pipeline/unified)
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
MAC="$ROOT/apps/mac"
INTERVALS="30 0"
SEEDS="demo body60k fixture"
PRODUCERS="compiler render"
OUT=""
WANT_RENDER=1
UTC="$(date -u +%Y-%m-%dT%H%M%SZ)"

while [[ $# -gt 0 ]]; do
  case "$1" in
    --intervals) INTERVALS="$2"; shift 2 ;;
    --seeds) SEEDS="$2"; shift 2 ;;
    --producers) PRODUCERS="$2"; shift 2 ;;
    --out) OUT="$2"; shift 2 ;;
    --no-render) WANT_RENDER=0; shift ;;
    -h|--help) sed -n '2,14p' "${BASH_SOURCE[0]}"; exit 0 ;;
    *) echo "run.sh: unknown argument $1" >&2; exit 1 ;;
  esac
done

WORK="$MAC/build/typing-bench/$UTC"
mkdir -p "$WORK"
[[ -n "$OUT" ]] || OUT="$ROOT/docs/evidence/typing-bench-$UTC.md"
RAW_DIR="$(dirname "$OUT")/typing-bench-$UTC"
mkdir -p "$RAW_DIR"

step() { echo "==> $*"; }

# --- 1. Build the app (release) ------------------------------------------
step "building FlashTeXMac (release)"
swift build -c release --package-path "$MAC" 2>&1 | tail -1
APP_BIN="$MAC/.build/release/FlashTeXMac"

# --- 2. Producers ---------------------------------------------------------
declare -a PRODUCER_NAMES=() PRODUCER_PATHS=() PRODUCER_NOTES=()
COMPILER="$ROOT/crates/compiler/target/release/flashtex-compiler"
if [[ " $PRODUCERS " == *" compiler "* ]]; then
  if [[ ! -x "$COMPILER" ]]; then
    step "building flashtex-compiler (release)"
    cargo build --release --manifest-path "$ROOT/crates/compiler/Cargo.toml" 2>&1 | tail -1
  fi
  PRODUCER_NAMES+=("compiler"); PRODUCER_PATHS+=("$COMPILER")
  PRODUCER_NOTES+=("crates/compiler @ $(git -C "$ROOT" rev-parse --short HEAD) (this checkout)")
fi
if [[ " $PRODUCERS " == *" render "* && $WANT_RENDER == 1 ]]; then
  RENDER="${FLASHTEX_RENDER:-$ROOT/crates/render-pipeline/target/release/flashtex-render}"
  RENDER_NOTE="crates/render-pipeline/target/release (this checkout)"
  if [[ ! -x "$RENDER" ]]; then
    REF="${FLASHTEX_RENDER_REF:-origin/agent/mac-render-pipeline/unified}"
    SCRATCH="$MAC/build/typing-bench/render-pipeline"
    step "building flashtex-render from $REF in $SCRATCH"
    if git -C "$ROOT" rev-parse --verify -q "$REF" >/dev/null; then
      mkdir -p "$SCRATCH"
      if git -C "$ROOT" archive "$REF" crates/render-pipeline | tar -x -C "$SCRATCH" \
         && cargo build --release --manifest-path "$SCRATCH/crates/render-pipeline/Cargo.toml" --bin flashtex-render 2>&1 | tail -1; then
        RENDER="$SCRATCH/crates/render-pipeline/target/release/flashtex-render"
        RENDER_NOTE="$REF @ $(git -C "$ROOT" rev-parse --short "$REF") (scratch build, vendored siblings)"
      else
        echo "    flashtex-render build FAILED; skipping that producer" >&2
        RENDER=""
        RENDER_NOTE="build failed from $REF"
      fi
    else
      echo "    $REF not found (git fetch origin); skipping flashtex-render" >&2
      RENDER=""; RENDER_NOTE="$REF not fetched"
    fi
  fi
  if [[ -n "$RENDER" && -x "$RENDER" ]]; then
    PRODUCER_NAMES+=("render"); PRODUCER_PATHS+=("$RENDER"); PRODUCER_NOTES+=("$RENDER_NOTE")
  else
    PRODUCER_NOTES+=("render: $RENDER_NOTE")
  fi
fi

# --- 3. Seeds -------------------------------------------------------------
TYPED="$SCRIPT_DIR/typed-200.txt"
python3 - "$ROOT" "$WORK" <<'PY'
import json, sys, os
root, work = sys.argv[1], sys.argv[2]
demo = open(os.path.join(root, "apps/mac/Samples/demo.tex"), encoding="utf-8").read()
open(os.path.join(work, "demo.tex"), "w", encoding="utf-8").write(demo)
# 60 KB body: the demo's paragraphs repeated inside one document.
head, body = demo.split("\\begin{document}\n", 1)
body = body.rsplit("\\end{document}", 1)[0]
out = head + "\\begin{document}\n"
while len(out.encode("utf-8")) < 60 * 1024:
    out += body
out += "\\end{document}\n"
open(os.path.join(work, "body60k.tex"), "w", encoding="utf-8").write(out)
req = json.load(open(os.path.join(root, "protocol/fixtures/compile-request.json")))
docs = req["payload"]["documents"]
text = next(d["text"] for d in docs if d["path"] == req["payload"]["entry_path"])
open(os.path.join(work, "fixture.tex"), "w", encoding="utf-8").write(text)
for n in ("demo", "body60k", "fixture"):
    print("    seed %-8s %6d bytes" % (n, os.path.getsize(os.path.join(work, n + ".tex"))))
PY

# --- 4. Runs --------------------------------------------------------------
run_one() { # producer_name producer_path seed interval -> writes $RAW_DIR/<name>.json
  local pname="$1" ppath="$2" seed="$3" ms="$4"
  local name="$pname-$seed-${ms}ms" log="$WORK/$pname-$seed-${ms}ms.log" json="$RAW_DIR/$pname-$seed-${ms}ms.json"
  rm -f "$log" "$json"
  step "run $name"
  (
    cd "$MAC"
    FLASHTEX_REPO="$ROOT" FLASHTEX_AUTOATTACH=1 FLASHTEX_NO_ACTIVATE=1 \
    FLASHTEX_COMPILER="$ppath" FLASHTEX_LM_DIR="$MAC/Fonts" FLASHTEX_FONT_DIRS="$MAC/Fonts" \
    FLASHTEX_SEED_FILE="$WORK/$seed.tex" FLASHTEX_LOG="$log" \
    FLASHTEX_TYPING_BENCH="$TYPED" FLASHTEX_TYPING_BENCH_MS="$ms" FLASHTEX_TYPING_BENCH_OUT="$json" \
    FLASHTEX_TYPING_BENCH_SETTLE_MS=60000 FLASHTEX_TYPING_BENCH_MAX_MS=120000 \
    "$APP_BIN" >/dev/null 2>&1 &
    pid=$!
    for _ in $(seq 1 600); do kill -0 "$pid" 2>/dev/null || break; sleep 0.5; done
    if kill -0 "$pid" 2>/dev/null; then echo "    timed out after 300 s; killing" >&2; kill "$pid" 2>/dev/null || true; fi
    wait "$pid" 2>/dev/null || true
  )
  if [[ -f "$json" ]]; then
    grep -F "bench: done" "$log" | sed 's/^/    /' || true
  else
    echo "    no summary written (see $log)" >&2
    grep -E "status:|bench:" "$log" | tail -3 | sed 's/^/    /' || true
  fi
}

for i in "${!PRODUCER_NAMES[@]}"; do
  for seed in $SEEDS; do
    for ms in $INTERVALS; do
      run_one "${PRODUCER_NAMES[$i]}" "${PRODUCER_PATHS[$i]}" "$seed" "$ms"
    done
  done
done

# --- 5. Evidence document --------------------------------------------------
step "writing $OUT"
python3 - "$OUT" "$RAW_DIR" "$ROOT" "$UTC" "$TYPED" "$(printf '%s\n' "${PRODUCER_NOTES[@]}")" <<'PY'
import json, os, subprocess, sys, glob
out, raw, root, utc, typed_path, notes = sys.argv[1:7]
def sh(*a):
    try: return subprocess.check_output(a, text=True).strip()
    except Exception as e: return "unavailable (%s)" % e
sha = sh("git", "-C", root, "rev-parse", "--short", "HEAD")
branch = sh("git", "-C", root, "rev-parse", "--abbrev-ref", "HEAD")
hw = sh("sysctl", "-n", "machdep.cpu.brand_string")
osv = sh("sw_vers", "-productVersion")
runs = []
for f in sorted(glob.glob(os.path.join(raw, "*.json"))):
    d = json.load(open(f)); d["_file"] = os.path.basename(f); runs.append(d)
def ms(v): return "—" if v is None else ("%.0f" % v if v >= 10 else "%.1f" % v)
lines = []
lines.append("# Typing bench: keystroke → paint latency (%s)" % utc)
lines.append("")
lines.append("Branch `%s` @ `%s`; %s; macOS %s; release build of `FlashTeXMac` (`swift build -c release`)." % (branch, sha, hw, osv))
lines.append("Script: `tools/typing-bench/run.sh`; raw JSON summaries in `%s/`." % os.path.basename(raw))
lines.append("")
lines.append("Producers:")
for n in notes.splitlines():
    if n.strip(): lines.append("- " + n.strip())
lines.append("")
lines.append("## Results")
lines.append("")
lines.append("Latency is keystroke → paint per typed character (ms); a coalesced keystroke is measured to the first paint that showed it. `compile` is the shell's send → result time on the main thread (waits behind any render pass in progress); `render` is PreviewView body → last page Canvas draw.")
lines.append("")
lines.append("| producer | seed | bytes | interval | keys typed | paints | coalesced | k→p p50 | p95 | p99 | max | compile p50 | compile p95 | render p50 | render p95 | unpainted |")
lines.append("|---|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|")
for d in runs:
    prod, seed, interval = d["_file"][:-5].split("-", 2)
    k = d["keystroke_to_paint_ms"]; c = d["compile_ms"]; r = d.get("render_pass_ms", {})
    typed = "%d" % d["keystrokes"] + (" of %d (budget)" % d["script_keystrokes"] if d.get("typing_budget_exhausted") else "")
    lines.append("| %s | %s | %d | %s | %s | %d | %d | %s | %s | %s | %s | %s | %s | %s | %s | %d |" % (
        d["producer"], seed, d["document_bytes_after"], interval, typed, d["paints"], d["coalesced"],
        ms(k.get("p50_ms")), ms(k.get("p95_ms")), ms(k.get("p99_ms")), ms(k.get("max_ms")),
        ms(c.get("p50_ms")), ms(c.get("p95_ms")), ms(r.get("p50_ms")), ms(r.get("p95_ms")), d["unpainted"]))
if not runs:
    lines.append("| (no runs completed) | | | | | | | | | | | | | | | |")
lines.append("")
lines.append("Typed script: `tools/typing-bench/typed-200.txt` (%d characters, inserted before `\\end{document}` when present, else at the end)." % len(open(typed_path, encoding="utf-8").read()))
lines.append("Seeds: `demo` = `apps/mac/Samples/demo.tex`; `body60k` = the demo's paragraphs repeated to ≥ 60 KB in one document; `fixture` = the entry document of `protocol/fixtures/compile-request.json`.")
lines.append("")
lines.append("## Methodology")
lines.append("")
lines.append("- The app is launched with `FLASHTEX_AUTOATTACH=1 FLASHTEX_NO_ACTIVATE=1 FLASHTEX_SEED_FILE=<seed> FLASHTEX_TYPING_BENCH=<script>` and the producer as `FLASHTEX_COMPILER`. After the worker attached and its first result was painted, `TypingBenchDriver` (`apps/mac/Sources/FlashTeXMac/TypingBench.swift`) inserts the script one extended grapheme cluster at a time into the real editor `NSTextView` through `insertText(_:replacementRange:)` from a main-run-loop `Timer` at the configured interval (30 ms ≈ a fast typist; 0 ms = one keystroke per run-loop turn, a burst). Each insertion takes the production path: `NSTextViewDelegate.textDidChange` → SwiftUI binding → `ShellModel.updateActiveText` (revision bump, `keystroke:` log line) → auto-compile (`FLASHTEX_DEBOUNCE_MS` default 0, one request in flight, newest buffer coalesced) → `compile_result` → `PreviewView` render.")
lines.append("- Keystroke time is stamped immediately before `insertText` on the monotonic clock (`clock_gettime_nsec_np(CLOCK_UPTIME_RAW)`, i.e. `mach_absolute_time` in ns). For a person typing, the same recorder uses the `NSEvent.timestamp` of the `keyDown` seen by an in-process local event monitor (HID time on the same clock) — no Accessibility permission or event tap is involved, and a key that changes no text is discarded at the end of its dispatch.")
lines.append("- Paint time (`paint:` log line) is the first main-queue turn after the run-loop iteration whose SwiftUI render pass evaluated `PreviewView.body` for the result revision; the page `Canvas` draw closures run inside that pass and are counted, so the CoreAnimation commit containing the new pages has completed when the stamp is taken. A revision whose result changed nothing visible (SwiftUI skipped the canvas redraw) is still recorded as painted, flagged `redrawn: false`.")
lines.append("- A paint of revision N makes every unpainted keystroke with revision ≤ N visible; those with revision < N are `coalesced`. p50/p95/p99 are nearest-rank percentiles over the per-keystroke latencies. The run ends when every keystroke is painted (or after the settle timeout), and the app writes the JSON summary and exits.")
lines.append("")
lines.append("## Limitations")
lines.append("")
lines.append("- The bench inserts text programmatically: there is no OS keyboard event, no event-queue wait, no key repeat and no input-method composition; real typing adds the HID → WindowServer → `NSApplication.sendEvent` hop, which the local-monitor path measures but this bench cannot.")
lines.append("- `paint` is the completed CoreAnimation commit, not the display scan-out: the pixels reach the panel at the next vsync (up to one frame, 8–17 ms at 60–120 Hz) after the stamp, and later still if the render server is behind. No IOSurface presentation callback is observed. The window is ordered back (`FLASHTEX_NO_ACTIVATE=1`) and may be occluded during the run; commits still happen, on-screen visibility is not verified.")
lines.append("- Compile time is measured on the main thread from send to result application, so it includes any time the reply waited behind a render pass; the producers' own round trip is 1–6 ms for these documents when measured directly.")
lines.append("- Typing stops after `FLASHTEX_TYPING_BENCH_MAX_MS` (120 s here); a cell marked 'of 200 (budget)' typed fewer characters because each keystroke waited for a main-thread render pass.")
lines.append("- One machine, one run per cell, no warm-up discard beyond the first compile; numbers are indicative, not a regression gate.")
open(out, "w", encoding="utf-8").write("\n".join(lines) + "\n")
print("\n".join(lines[:4]))
for l in lines:
    if l.startswith("| ") and not l.startswith("| producer") and not l.startswith("|---"): print(l)
PY
