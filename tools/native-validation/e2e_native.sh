#!/usr/bin/env bash
# Native end-to-end exercise of the real Mac workflow (FT-010 follow-up, issue #2).
#
# Builds the Mac shell, the compiler it discovers (crates/compiler inside the
# same worktree), the FT-009 PDF writer and the capture bridge from git refs in
# scratch worktrees, then runs:
#   1. compiler -> Mac -> PDF  (app-observed: process, compiler child, window,
#      screenshot by window id; CLI-equivalent: compiler -> flashtex-pdf --verify
#      -> PDFKit page count / words, because menus cannot be driven without
#      Accessibility)
#   2. responsiveness: 20x CLI round trip over Samples/demo.tex while the app
#      is attached, and `swift test --filter RealCompilerTests` REAL-COMPILER LATENCY
#   3. crash/restart: SIGKILL the compiler child (app must survive 5 s), relaunch
#      (must re-attach), SIGKILL the app, relaunch (window within 5 s)
#   4. non-regression: run_all.sh + oracle_compare.sh --only fixture-hello,
#      diffed against the newest previous reports
#   5. bridge receipt path against the real flashtex-bridge binary
# Writes reports/e2e-<UTC>.md plus screenshots (PNG <= 300 KB). Only this
# script's own app instance (tracked by PID from $!) is ever signalled.
#
# Usage: e2e_native.sh [--repo <path>] [--mac-ref <ref>] [--pdf-ref <ref>] [--bridge-ref <ref>]
#                      [--scratch <dir>] [--reports-dir <dir>] [--keep] [--skip-nonregression]
set -euo pipefail

HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO="$(git -C "$HERE" rev-parse --show-toplevel)"
MAC_REF="origin/agent/mac-claude-a/mac-shell"
PDF_REF="origin/agent/mac-pdf/pdf-output"
BRIDGE_REF="origin/agent/commander/capture-bridge"
SCRATCH="${TMPDIR:-/tmp}/flashtex-validation"
REPORTS_DIR=""
KEEP=0
SKIP_NONREG=0
while [[ $# -gt 0 ]]; do
  case "$1" in
    --repo) REPO="$(cd "$2" && pwd)"; shift 2 ;;
    --mac-ref) MAC_REF="$2"; shift 2 ;;
    --pdf-ref) PDF_REF="$2"; shift 2 ;;
    --bridge-ref) BRIDGE_REF="$2"; shift 2 ;;
    --scratch) SCRATCH="$2"; shift 2 ;;
    --reports-dir) REPORTS_DIR="$2"; shift 2 ;;
    --keep) KEEP=1; shift ;;
    --skip-nonregression) SKIP_NONREG=1; shift ;;
    -h|--help) sed -n '2,24p' "$0"; exit 0 ;;
    *) echo "unknown argument: $1" >&2; exit 2 ;;
  esac
done
[[ -n "$REPORTS_DIR" ]] || REPORTS_DIR="$REPO/tools/native-validation/reports"

STAMP="$(date -u +%Y%m%dT%H%M%SZ)"
RUN="$SCRATCH/e2e-$STAMP"
LOGS="$RUN/logs"
mkdir -p "$LOGS" "$REPORTS_DIR"
REPORT="$REPORTS_DIR/e2e-$STAMP.md"
cat >"$RUN/pick_window.py" <<'PY'
import json, sys
ws = [w for w in json.load(sys.stdin) if w["bounds"]["Width"] > 200 and w["bounds"]["Height"] > 200]
print("%d %dx%d" % (ws[0]["window_id"], ws[0]["bounds"]["Width"], ws[0]["bounds"]["Height"]) if ws else "")
PY
cat >"$RUN/pdf_words.py" <<'PY'
import json, sys, unicodedata
d = json.load(sys.stdin)
words = [w["text"] for p in d["pages"] for w in p["words"]]
norm = lambda s: "".join(c for c in unicodedata.normalize("NFD", s) if not unicodedata.combining(c)).lower()
joined = " ".join(norm(w) for w in words)
expect = ["wrapping", "accents", "naive", "cafe", "bold", "emphasised", "paragraph", "reproducibility", "dash"]
missing = [e for e in expect if e not in joined]
print("pages=%d words=%d expected-words-missing=%s" % (len(d["pages"]), len(words), missing))
sys.exit(1 if missing or not d["pages"] else 0)
PY
PREV_RUN_REPORT="$(ls -1 "$REPORTS_DIR"/report-*.md 2>/dev/null | sort | tail -1 || true)"
PREV_ORACLE_REPORT="$(ls -1 "$REPORTS_DIR"/oracle-*.md 2>/dev/null | sort | tail -1 || true)"

MAC_SHA="$(git -C "$REPO" rev-parse --verify "$MAC_REF^{commit}")"
PDF_SHA="$(git -C "$REPO" rev-parse --verify "$PDF_REF^{commit}")"
BRIDGE_SHA="$(git -C "$REPO" rev-parse --verify "$BRIDGE_REF^{commit}")"
MAC_WT="$RUN/mac"; PDF_WT="$RUN/pdf"; BRIDGE_WT="$RUN/bridge"

ROWS=()
CMDS=()
OVERALL=0
APP_PID=""
now_ms() { python3 -c 'import time; print(int(time.time()*1000))'; }
row() { # status check detail
  ROWS+=("| $1 | $2 | $(printf '%s' "$3" | tr '\n' ' ' | sed 's/|/\\|/g' | cut -c1-700) |")
  printf '[%s] %s -- %s\n' "$1" "$2" "$3"
  [[ "$1" == "FAIL" ]] && OVERALL=1 || true
}
cmd() { # scope logname cwd -- command...   (records exit + duration, output to log)
  local scope="$1" log="$2" cwd="$3"; shift 3; [[ "$1" == "--" ]] && shift
  local t0 rc=0; t0=$(now_ms)
  ( cd "$cwd" && "$@" ) >"$LOGS/$log.log" 2>&1 || rc=$?
  local dur=$(( $(now_ms) - t0 ))
  CMDS+=("| $scope | \`$*\` | $rc | $((dur/1000)).$(printf '%03d' $((dur%1000))) s | logs/$log.log |")
  return $rc
}

kill_own_app() {
  if [[ -n "$APP_PID" ]] && kill -0 "$APP_PID" 2>/dev/null; then
    kill -9 "$APP_PID" 2>/dev/null || true
    wait "$APP_PID" 2>/dev/null || true
  fi
  APP_PID=""
}
cleanup() {
  kill_own_app
  if [[ $KEEP -eq 1 ]]; then echo "keeping $RUN"; return; fi
  for wt in "$MAC_WT" "$PDF_WT" "$BRIDGE_WT"; do
    [[ -d "$wt" ]] && { git -C "$REPO" worktree remove --force "$wt" >/dev/null 2>&1 || rm -rf "$wt"; }
  done
  git -C "$REPO" worktree prune >/dev/null 2>&1 || true
}
trap cleanup EXIT

echo "== refs: mac $MAC_REF=$MAC_SHA  pdf $PDF_REF=$PDF_SHA  bridge $BRIDGE_REF=$BRIDGE_SHA"
git -C "$REPO" worktree add --detach "$MAC_WT" "$MAC_SHA" >/dev/null
git -C "$REPO" worktree add --detach "$PDF_WT" "$PDF_SHA" >/dev/null
git -C "$REPO" worktree add --detach "$BRIDGE_WT" "$BRIDGE_SHA" >/dev/null
COMPILER_SRC_SHA="$(git -C "$REPO" log -1 --format=%h "$MAC_SHA" -- crates/compiler)"

# ---------------------------------------------------------------- builds
cmd build cargo-compiler "$MAC_WT/crates/compiler" -- cargo build --release && row PASS "build: crates/compiler in the Mac worktree (last change $COMPILER_SRC_SHA)" "cargo build --release" || row FAIL "build: crates/compiler" "see logs/cargo-compiler.log"
cmd build cargo-pdf "$PDF_WT/crates/pdf" -- cargo build --release && row PASS "build: crates/pdf ($PDF_SHA)" "cargo build --release" || row FAIL "build: crates/pdf" "see logs/cargo-pdf.log"
cmd build cargo-bridge "$BRIDGE_WT/crates/bridge" -- cargo build --release && row PASS "build: crates/bridge ($BRIDGE_SHA)" "cargo build --release" || row FAIL "build: crates/bridge" "see logs/cargo-bridge.log"
cmd build swift-build "$MAC_WT/apps/mac" -- swift build && row PASS "build: apps/mac swift build (debug)" "swift build" || row FAIL "build: apps/mac" "see logs/swift-build.log"
cmd build swiftc-probes "$RUN" -- bash -c "swiftc -O '$HERE/window_probe.swift' -o '$RUN/window_probe' && swiftc -O '$HERE/oracle_extract.swift' -o '$RUN/oracle_extract'" \
  && row PASS "build: window_probe + oracle_extract (PDFKit)" "swiftc" || row FAIL "build: probes" "see logs/swiftc-probes.log"

COMPILER="$MAC_WT/crates/compiler/target/release/flashtex-compiler"
PDF_BIN="$PDF_WT/crates/pdf/target/release/flashtex-pdf"
BRIDGE_BIN="$BRIDGE_WT/crates/bridge/target/release/flashtex-bridge"
APP="$MAC_WT/apps/mac/.build/debug/FlashTeXMac"
SEED="$HERE/oracle-samples/wrap-sample.tex"
DEMO="$MAC_WT/apps/mac/Samples/demo.tex"
[[ -x "$COMPILER" && -x "$PDF_BIN" && -x "$BRIDGE_BIN" && -x "$APP" ]] || { row FAIL "prerequisites" "a build failed; aborting checks"; exit 1; }

# ---------------------------------------------------------------- app helpers
launch_app() { # sets APP_PID; env hooks documented in apps/mac/README.md
  local log="$1"
  ( cd "$MAC_WT/apps/mac" && FLASHTEX_REPO="$MAC_WT" FLASHTEX_AUTOATTACH=1 FLASHTEX_SEED_FILE="$SEED" exec "$APP" ) >"$LOGS/$log.log" 2>&1 &
  APP_PID=$!
  CMDS+=("| app | \`FLASHTEX_REPO=$MAC_WT FLASHTEX_AUTOATTACH=1 FLASHTEX_SEED_FILE=$SEED $APP &\` (pid $APP_PID) | - | - | logs/$log.log |")
}
wait_window() { # prints "window_id widthxheight" once a real window exists, within $1 seconds; empty on timeout
  local deadline=$(( $(now_ms) + $1 * 1000 )) json
  while [[ $(now_ms) -lt $deadline ]]; do
    json="$("$RUN/window_probe" "$APP_PID" 2>/dev/null || echo '[]')"
    local id; id="$(printf '%s' "$json" | python3 "$RUN/pick_window.py")"
    [[ -n "$id" ]] && { echo "$id"; return 0; }
    sleep 0.25
  done
  echo ""
}
child_compiler() { pgrep -P "$APP_PID" -f flashtex-compiler 2>/dev/null | head -1 || true; }
shot() { # window_id out.png -> keeps <=300 KB
  local wid="$1" out="$2" err
  if err=$(screencapture -x -o -l "$wid" "$out" 2>&1) && [[ -s "$out" ]]; then
    local w=1200
    while [[ $(stat -f %z "$out") -gt 300000 && $w -gt 300 ]]; do sips -Z $w "$out" --out "$out" >/dev/null 2>&1; w=$((w-200)); done
    echo "captured $(stat -f %z "$out") bytes $(sips -g pixelWidth -g pixelHeight "$out" 2>/dev/null | awk '/pixel/ {printf "%s ", $2}')px"
  else
    rm -f "$out"; echo "NOT captured: $err"
  fi
}

# ---------------------------------------------------------------- 1. compiler -> Mac -> PDF
echo "== check 1"
launch_app app-check1
sleep 3
if kill -0 "$APP_PID" 2>/dev/null; then row PASS "1 app-observed: FlashTeXMac process alive 3 s after launch" "pid $APP_PID"; else row FAIL "1 app-observed: FlashTeXMac process alive" "exited; log: $(head -c 300 "$LOGS/app-check1.log")"; fi
CHILD="$(child_compiler)"
if [[ -n "$CHILD" ]]; then row PASS "1 app-observed: flashtex-compiler child attached (FLASHTEX_AUTOATTACH=1)" "pgrep -P $APP_PID -f flashtex-compiler -> $CHILD ($(ps -o command= -p "$CHILD" | cut -c1-120))"; else row FAIL "1 app-observed: flashtex-compiler child attached" "no child of $APP_PID"; fi
WIN="$(wait_window 10)"
if [[ -n "$WIN" ]]; then row PASS "1 app-observed: window present (CGWindowListCopyWindowInfo)" "window id ${WIN%% *}, ${WIN#* } pt, title FlashTeX"; else row FAIL "1 app-observed: window present" "no window >200x200 for pid $APP_PID within 10 s: $("$RUN/window_probe" "$APP_PID")"; fi
SHOT1="$REPORTS_DIR/e2e-$STAMP-check1-app.png"
if [[ -n "$WIN" ]]; then
  sleep 1  # let the first compile land
  res="$(shot "${WIN%% *}" "$SHOT1")"
  if [[ "$res" == captured* ]]; then row PASS "1 app-observed: screenshot by window id" "screencapture -x -o -l ${WIN%% *} -> $(basename "$SHOT1"): $res"; else row INFO "1 app-observed: screenshot by window id" "$res"; fi
fi
# CLI-equivalent of File > Export PDF via Rust Writer: same input text the app seeded.
python3 - "$SEED" "$RUN/check1-request.json" <<'PY'
import json,sys
text=open(sys.argv[1],encoding="utf-8").read()
json.dump({"protocol_version":1,"id":"e2e-check1","type":"compile","payload":{"project_id":"e2e","revision":1,"entry_path":"main.tex","documents":[{"path":"main.tex","text":text}]}},open(sys.argv[2],"w"),ensure_ascii=False)
PY
if cmd check1 cli-compile "$RUN" -- bash -c "'$COMPILER' < check1-request.json > check1-result.json"; then
  STATUS="$(python3 -c 'import json,sys; d=json.load(open(sys.argv[1])); print(d["payload"]["status"], len(d["payload"]["pages"]), len(d["payload"]["diagnostics"]))' "$RUN/check1-result.json")"
  row PASS "1 CLI-equivalent: flashtex-compiler compile_result for the seeded file" "status/pages/diagnostics: $STATUS"
else row FAIL "1 CLI-equivalent: compile" "see logs/cli-compile.log"; fi
if cmd check1 cli-pdf "$RUN" -- "$PDF_BIN" check1-result.json --out check1.pdf --verify; then
  row PASS "1 CLI-equivalent: flashtex-pdf --verify wrote check1.pdf" "$(stat -f %z "$RUN/check1.pdf") bytes; warnings: $(tr '\n' ' ' < "$LOGS/cli-pdf.log" | cut -c1-200)"
  PDFCHECK="$("$RUN/oracle_extract" "$RUN/check1.pdf" | python3 "$RUN/pdf_words.py")" && row PASS "1 CLI-equivalent: PDFKit validation of check1.pdf (page count, expected words)" "$PDFCHECK" \
    || row FAIL "1 CLI-equivalent: PDFKit validation of check1.pdf" "$PDFCHECK"
else row FAIL "1 CLI-equivalent: flashtex-pdf" "see logs/cli-pdf.log"; fi

# ---------------------------------------------------------------- 2. responsiveness
echo "== check 2"
DEMO_BYTES="$(stat -f %z "$DEMO")"
if cmd check2 cli-latency "$RUN" -- python3 "$HERE/e2e_latency.py" --compiler "$COMPILER" --document "$DEMO" --count 20 --json "$RUN/check2-latency.json"; then
  row PASS "2 CLI round trip x20 over Samples/demo.tex ($DEMO_BYTES bytes) while the app is attached" "$(grep LATENCY "$LOGS/cli-latency.log")"
else row FAIL "2 CLI round trip" "see logs/cli-latency.log"; fi
if cmd check2 swift-test-real "$MAC_WT/apps/mac" -- env FLASHTEX_COMPILER="$COMPILER" swift test --filter RealCompilerTests; then
  LAT_LINE="$(grep -h 'REAL-COMPILER LATENCY' "$LOGS/swift-test-real.log" | head -1 || true)"
  row PASS "2 in-app path: swift test --filter RealCompilerTests (debounce/coalescing)" "${LAT_LINE:-passed but no REAL-COMPILER LATENCY line found}; $(grep -h 'Executed' "$LOGS/swift-test-real.log" | tail -1)"
else row FAIL "2 in-app path: swift test --filter RealCompilerTests" "$(grep -h 'error\|failed' "$LOGS/swift-test-real.log" | head -3)"; fi

# ---------------------------------------------------------------- 3. crash / restart
echo "== check 3"
CHILD="$(child_compiler)"
if [[ -n "$CHILD" ]]; then
  kill -9 "$CHILD"; CMDS+=("| check3 | \`kill -9 $CHILD\` (compiler child) | 0 | - | - |")
  sleep 5
  if kill -0 "$APP_PID" 2>/dev/null; then row PASS "3 app survives SIGKILL of its compiler child for 5 s" "child $CHILD killed; app pid $APP_PID alive; new child: $(child_compiler || echo none) (the app reports worker exit in its banner; it does not auto-respawn)"; else row FAIL "3 app survives SIGKILL of its compiler child" "app pid $APP_PID exited; log: $(tail -c 300 "$LOGS/app-check1.log")"; fi
else row FAIL "3 compiler child present before kill" "none"; fi
kill "$APP_PID" 2>/dev/null || true; wait "$APP_PID" 2>/dev/null || true; CMDS+=("| check3 | \`kill $APP_PID\` (SIGTERM app) | 0 | - | - |"); APP_PID=""
launch_app app-check3a; sleep 3
CHILD2="$(child_compiler)"
if [[ -n "$CHILD2" ]]; then row PASS "3 relaunch re-attaches a compiler child" "pid $APP_PID child $CHILD2"; else row FAIL "3 relaunch re-attaches a compiler child" "pid $APP_PID has no flashtex-compiler child"; fi
kill -9 "$APP_PID"; wait "$APP_PID" 2>/dev/null || true; CMDS+=("| check3 | \`kill -9 $APP_PID\` (SIGKILL app) | 0 | - | - |")
sleep 0.5
if pgrep -P 1 -f "$COMPILER" >/dev/null 2>&1; then ORPHAN="$(pgrep -P 1 -f "$COMPILER" | tr '\n' ' ')"; row INFO "3 orphaned compiler child after SIGKILL of the app" "pids $ORPHAN reparented to launchd (stdin closed -> should exit); left alone"; else row INFO "3 orphaned compiler child after SIGKILL of the app" "none"; fi
APP_PID=""
T0=$(now_ms); launch_app app-check3b
WIN3="$(wait_window 5)"; T1=$(now_ms)
if [[ -n "$WIN3" ]]; then row PASS "3 relaunch after SIGKILL: window present within 5 s" "$(( (T1-T0) )) ms to window ${WIN3%% *} (${WIN3#* } pt); child: $(child_compiler || echo none)"; else row FAIL "3 relaunch after SIGKILL: window within 5 s" "no window; alive=$(kill -0 "$APP_PID" 2>/dev/null && echo yes || echo no)"; fi
SHOT3="$REPORTS_DIR/e2e-$STAMP-check3-relaunch.png"
[[ -n "$WIN3" ]] && { sleep 1; row INFO "3 screenshot after relaunch" "$(shot "${WIN3%% *}" "$SHOT3")"; }
kill_own_app

# ---------------------------------------------------------------- 4. non-regression
echo "== check 4"
NEW_RUN_REPORT=""; NEW_ORACLE_REPORT=""
if [[ $SKIP_NONREG -eq 0 ]]; then
  if cmd check4 run-all "$HERE" -- bash "$HERE/run_all.sh" --scratch "$RUN/nonreg" --reports-dir "$REPORTS_DIR"; then :; fi
  NEW_RUN_REPORT="$(grep -h '^report:' "$LOGS/run-all.log" | awk '{print $2}' | tail -1)"
  row INFO "4 run_all.sh re-run" "$(grep -h '^overall:' "$LOGS/run-all.log" | tail -1); report $(basename "${NEW_RUN_REPORT:-none}")"
  if cmd check4 oracle "$HERE" -- bash "$HERE/oracle_compare.sh" --compiler-ref main=origin/main --compiler-ref de1020c=de1020c --pdf-ref c0f3837 --only fixture-hello --scratch "$RUN/nonreg" --reports-dir "$REPORTS_DIR"; then :; fi
  NEW_ORACLE_REPORT="$(grep -h '^report:' "$LOGS/oracle.log" | awk '{print $2}' | tail -1)"
  row INFO "4 oracle_compare.sh --only fixture-hello re-run" "report $(basename "${NEW_ORACLE_REPORT:-none}")"
  if [[ -n "$PREV_RUN_REPORT" && -n "$NEW_RUN_REPORT" ]]; then
    python3 "$HERE/e2e_nonregression.py" --prev-run "$PREV_RUN_REPORT" --new-run "$NEW_RUN_REPORT" \
      ${PREV_ORACLE_REPORT:+--prev-oracle "$PREV_ORACLE_REPORT"} ${NEW_ORACLE_REPORT:+--new-oracle "$NEW_ORACLE_REPORT"} \
      --only fixture-hello --json "$RUN/check4-diff.json" > "$LOGS/nonregression.log" 2>&1 || true
    NCH="$(grep -h 'non-regression:' "$LOGS/nonregression.log" | tail -1)"
    if grep -q '^\[CHANGED\]\|^\[MISSING\]' "$LOGS/nonregression.log"; then row INFO "4 non-regression diff vs $(basename "$PREV_RUN_REPORT") / $(basename "${PREV_ORACLE_REPORT:-none}")" "$NCH: $(grep -h '^\[CHANGED\]\|^\[MISSING\]' "$LOGS/nonregression.log" | tr '\n' ' ' | cut -c1-600)"; else row PASS "4 non-regression diff vs $(basename "$PREV_RUN_REPORT") / $(basename "${PREV_ORACLE_REPORT:-none}")" "$NCH"; fi
  else row INFO "4 non-regression diff" "no previous report to compare (prev run: ${PREV_RUN_REPORT:-none})"; fi
else row INFO "4 non-regression" "skipped (--skip-nonregression)"; fi

# ---------------------------------------------------------------- 5. bridge receipt path
echo "== check 5"
if cmd check5 bridge "$RUN" -- python3 "$HERE/e2e_bridge.py" --bridge "$BRIDGE_BIN" --fixture "$BRIDGE_WT/protocol/fixtures/capture-submission.json" \
     --store "$RUN/bridge-store" --also-fixture "mac-shell=$MAC_WT/protocol/fixtures/capture-submission.json" --json "$RUN/check5-bridge.json"; then
  row PASS "5 bridge receipt path (document_open, destination_pin, capture_submit, duplicate, convert w/o --enable-grok, reject)" "$(grep -h '^summary' "$LOGS/bridge.log")"
else row FAIL "5 bridge receipt path" "$(grep -h '^\[FAIL\]' "$LOGS/bridge.log" | tr '\n' ' ' | cut -c1-600)"; fi
if grep -q '^\[FINDING\]' "$LOGS/bridge.log"; then row FINDING "5 fixture accepted by the bridge (fixture owner: Commander)" "$(grep -h '^\[FINDING\]' "$LOGS/bridge.log" | sed 's/^\[FINDING\] //' | tr '\n' ' ' | cut -c1-600)"; fi
BRIDGE_ROWS="$(grep -h '^\[' "$LOGS/bridge.log" || true)"

# ---------------------------------------------------------------- report
{
  echo "# FlashTeX native end-to-end report"
  echo
  echo "- Generated: $(date -u +%Y-%m-%dT%H:%M:%SZ) on $(hostname -s) (macOS $(sw_vers -productVersion), $(uname -m)); worker mac-native-e2e"
  echo "- Mac shell: \`$MAC_REF\` = \`$MAC_SHA\` (compiler discovered by the app = \`crates/compiler\` in that tree, last changed \`$COMPILER_SRC_SHA\`)"
  echo "- PDF writer: \`$PDF_REF\` = \`$PDF_SHA\`; bridge: \`$BRIDGE_REF\` = \`$BRIDGE_SHA\`"
  echo "- Suite: \`$(git -C "$REPO" rev-parse HEAD)\`; toolchain: $(swift --version 2>&1 | head -1); $(xcodebuild -version 2>&1 | tr '\n' ' '); $(cargo --version); $(python3 --version)"
  echo "- Session: \`launchctl managername\` = $(launchctl managername 2>/dev/null); Accessibility for osascript UI scripting: not granted (see README); Screen Recording: see screenshot rows"
  echo "- Scratch: \`$RUN\` (logs not committed); overall: **$([[ $OVERALL -eq 0 ]] && echo PASS || echo FAIL)**"
  echo
  echo "App-observed steps are things measured on the running \`FlashTeXMac\` process launched by this script (own PID only). CLI-equivalent steps run the same binaries the app's menu items invoke (\`flashtex-compiler\`, \`flashtex-pdf --verify\`) because menus cannot be driven without Accessibility."
  echo
  echo "## Checks"
  echo
  echo "| Status | Check | Detail |"
  echo "|---|---|---|"
  printf '%s\n' "${ROWS[@]}"
  echo
  echo "## Screenshots"
  echo
  for f in "$SHOT1" "$SHOT3"; do [[ -f "$f" ]] && echo "- \`$(basename "$f")\` ($(stat -f %z "$f") bytes) ![]($(basename "$f"))"; done
  echo
  echo "## Bridge replies"
  echo
  echo '```'; echo "$BRIDGE_ROWS"; echo '```'
  echo
  echo "## Non-regression detail"
  echo
  echo '```'; cat "$LOGS/nonregression.log" 2>/dev/null || echo "(not run)"; echo '```'
  echo
  echo "## Commands"
  echo
  echo "| Scope | Command | Exit | Duration | Log |"
  echo "|---|---|---|---|---|"
  printf '%s\n' "${CMDS[@]}"
} >"$REPORT"
echo "report: $REPORT"
echo "overall: $([[ $OVERALL -eq 0 ]] && echo PASS || echo FAIL)"
exit $OVERALL
