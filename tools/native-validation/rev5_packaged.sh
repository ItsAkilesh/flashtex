#!/usr/bin/env bash
# FT-003 rev 5 "Packaged native recovery and responsiveness" evidence.
#
# Builds the Mac shell tree (which carries crates/compiler, pdf, bridge and
# edit-ledger) from a ref in a scratch worktree, packages FlashTeX.app with
# apps/mac/scripts/make-app.sh, records the integrated component SHAs, and
# produces reports/rev5-<UTC>.md covering:
#   (1) packaged app runs the pinned demo with measured latency
#       - edit->visible: FLASHTEX_LOG "revision N: ... in X ms" over >=10 launches
#         with a modified seed (the only edit route without Accessibility), plus
#         the swift-test REAL-COMPILER LATENCY line (in-app debounce path)
#       - capture: real bridge capture_submit -> capture_received x10 (rev5_bridge.py)
#       - export: flashtex-pdf --verify --default-face lm x10, and PDFExportTests durations
#   (2) recovery: crash/restart of compiler, bridge and ledger children; disconnect/
#       retry (cited Swift tests + real bridge kill/resubmit); stale preview (cited
#       tests + CLI revision ordering); accessibility tests; update path (launch-check)
#   (3) explicit gaps: devices, signing (spctl), provider, visual
# Only this script's own app instance (PID from $!) is signalled. The app
# owner's launch-check.sh does `pkill -x FlashTeX`, so it is run only when no
# foreign FlashTeX process exists.
#
# Usage: rev5_packaged.sh [--app-ref <ref>] [--expect <component>=<sha> ...] [--launches 10]
#                         [--scratch <dir>] [--reports-dir <dir>] [--keep]
set -euo pipefail
HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO="$(git -C "$HERE" rev-parse --show-toplevel)"
APP_REF="origin/agent/mac-claude-a/mac-shell"
LAUNCHES=10
SCRATCH="${TMPDIR:-/tmp}/flashtex-validation"
REPORTS_DIR="$HERE/reports"
KEEP=0
EXPECT=()
while [[ $# -gt 0 ]]; do
  case "$1" in
    --app-ref) APP_REF="$2"; shift 2 ;;
    --expect) EXPECT+=("$2"); shift 2 ;;
    --launches) LAUNCHES="$2"; shift 2 ;;
    --scratch) SCRATCH="$2"; shift 2 ;;
    --reports-dir) REPORTS_DIR="$2"; shift 2 ;;
    --keep) KEEP=1; shift ;;
    -h|--help) sed -n '2,24p' "$0"; exit 0 ;;
    *) echo "unknown argument: $1" >&2; exit 2 ;;
  esac
done
STAMP="$(date -u +%Y%m%dT%H%M%SZ)"
RUN="$SCRATCH/rev5-$STAMP"; LOGS="$RUN/logs"; WT="$RUN/app"
mkdir -p "$LOGS" "$REPORTS_DIR"
REPORT="$REPORTS_DIR/rev5-$STAMP.md"
APP_SHA="$(git -C "$REPO" rev-parse --verify "$APP_REF^{commit}")"

ROWS=(); CMDS=(); OVERALL=0; APP_PID=""
now_ms() { python3 -c 'import time; print(int(time.time()*1000))'; }
row() { ROWS+=("| $1 | $2 | $(printf '%s' "$3" | tr '\n' ' ' | sed 's/|/\\|/g' | cut -c1-900) |"); printf '[%s] %s -- %s\n' "$1" "$2" "$3"; [[ "$1" == "FAIL" ]] && OVERALL=1 || true; }
cmd() { local scope="$1" log="$2" cwd="$3"; shift 3; [[ "$1" == "--" ]] && shift
  local t0 rc=0; t0=$(now_ms); ( cd "$cwd" && "$@" ) >"$LOGS/$log.log" 2>&1 || rc=$?
  local dur=$(( $(now_ms) - t0 )); CMDS+=("| $scope | \`$*\` | $rc | $((dur/1000)).$(printf '%03d' $((dur%1000))) s | logs/$log.log |"); return $rc; }
kill_own() { if [[ -n "$APP_PID" ]] && kill -0 "$APP_PID" 2>/dev/null; then kill -9 "$APP_PID" 2>/dev/null || true; wait "$APP_PID" 2>/dev/null || true; fi; APP_PID=""; }
cleanup() { kill_own; [[ $KEEP -eq 1 ]] && { echo "keeping $RUN"; return; }; git -C "$REPO" worktree remove --force "$WT" >/dev/null 2>&1 || rm -rf "$WT"; git -C "$REPO" worktree prune >/dev/null 2>&1 || true; }
trap cleanup EXIT

cat >"$RUN/pick_window.py" <<'PY'
import json, sys
ws = [w for w in json.load(sys.stdin) if w["bounds"]["Width"] > 200 and w["bounds"]["Height"] > 200]
print("%d %dx%d" % (ws[0]["window_id"], ws[0]["bounds"]["Width"], ws[0]["bounds"]["Height"]) if ws else "")
PY
cat >"$RUN/stats.py" <<'PY'
import statistics, sys
v = [float(x) for x in sys.stdin.read().split()]
print("n=%d min=%.1f median=%.1f max=%.1f ms" % (len(v), min(v), statistics.median(v), max(v)) if v else "n=0")
PY
cat >"$RUN/time_pdf.py" <<'PY'
import statistics, subprocess, sys, time
binary, result, out, n, extra = sys.argv[1], sys.argv[2], sys.argv[3], int(sys.argv[4]), sys.argv[5:]
s = []
for i in range(n):
    t0 = time.perf_counter()
    p = subprocess.run([binary, result, "--out", out, "--verify"] + extra, capture_output=True)
    s.append((time.perf_counter() - t0) * 1000)
    if p.returncode != 0:
        print("exit %d: %s" % (p.returncode, p.stderr.decode()[:200])); sys.exit(1)
note = p.stderr.decode().strip().splitlines()[-1] if p.stderr.strip() else ""
print("n=%d min=%.1f median=%.1f max=%.1f ms; %s" % (n, min(s), statistics.median(s), max(s), note))
PY

echo "== app tree $APP_REF = $APP_SHA"
git -C "$REPO" worktree add --detach "$WT" "$APP_SHA" >/dev/null
# ---------------------------------------------------------------- components
COMP_ROWS=()
for c in compiler pdf bridge edit-ledger; do
  last="$(git -C "$REPO" log -1 --format=%h "$APP_SHA" -- "crates/$c")"
  exp=""; for e in "${EXPECT[@]:-}"; do [[ "${e%%=*}" == "$c" ]] && exp="${e#*=}"; done
  contained="n/a"
  if [[ -n "$exp" ]]; then
    if git -C "$REPO" merge-base --is-ancestor "$exp" "$APP_SHA" 2>/dev/null; then contained="yes"; else contained="**no**"; fi
  fi
  COMP_ROWS+=("| crates/$c | \`$last\` | ${exp:+\`$exp\`} | $contained |")
done
for c in compiler pdf bridge edit-ledger; do
  cmd build "cargo-$c" "$WT/crates/$c" -- cargo build --release && row PASS "build: crates/$c" "cargo build --release" || row FAIL "build: crates/$c" "see logs/cargo-$c.log"
done
COMPILER="$WT/crates/compiler/target/release/flashtex-compiler"; PDF_BIN="$WT/crates/pdf/target/release/flashtex-pdf"
BRIDGE_BIN="$WT/crates/bridge/target/release/flashtex-bridge"; LEDGER_BIN="$WT/crates/edit-ledger/target/release/flashtex-edit-ledger"
MAKEAPP="$WT/apps/mac/scripts/make-app.sh"
MAKE_ARGS=(--compiler "$COMPILER" --pdf "$PDF_BIN")
if grep -q -- '--bridge' "$MAKEAPP"; then MAKE_ARGS+=(--bridge "$BRIDGE_BIN"); BRIDGE_VIA="bundled via make-app.sh --bridge"; else BRIDGE_VIA="make-app.sh has no --bridge at this ref; supplied at launch via FLASHTEX_BRIDGE"; fi
if grep -q -- '--ledger' "$MAKEAPP"; then MAKE_ARGS+=(--ledger "$LEDGER_BIN"); LEDGER_VIA="bundled via make-app.sh --ledger"; else LEDGER_VIA="make-app.sh has no --ledger at this ref; supplied at launch via FLASHTEX_EDIT_LEDGER"; fi
cmd build make-app "$WT" -- bash "$MAKEAPP" "${MAKE_ARGS[@]}" && row PASS "package: make-app.sh (release) -> FlashTeX.app" "$(grep -h 'bundled\|codesign' "$LOGS/make-app.log" | tr '\n' ';' | cut -c1-300); $BRIDGE_VIA; $LEDGER_VIA" || row FAIL "package: make-app.sh" "see logs/make-app.log"
BUNDLE="$WT/apps/mac/build/FlashTeX.app"; APP="$BUNDLE/Contents/MacOS/FlashTeX"
[[ -x "$APP" ]] || { row FAIL "bundle executable present" "$APP"; exit 1; }
if [[ -f "$BUNDLE/Contents/Resources/components.json" ]]; then COMPONENTS_JSON="$(cat "$BUNDLE/Contents/Resources/components.json")"; row INFO "components.json in the bundle" "$COMPONENTS_JSON"; else COMPONENTS_JSON=""; row INFO "components.json in the bundle" "absent at this ref; SHAs computed from git (table in report)"; fi
swiftc -O "$HERE/window_probe.swift" -o "$RUN/window_probe" 2>"$LOGS/swiftc.log" || row FAIL "build: window_probe" "see logs/swiftc.log"
BUNDLED_LIST="$(ls "$BUNDLE/Contents/MacOS" | tr '\n' ' ')"

# ---------------------------------------------------------------- launch helpers
SEED_SRC="$WT/apps/mac/Samples/demo.tex"; SEED="$RUN/seed.tex"; cp "$SEED_SRC" "$SEED"
LOGFILE=""
launch() { # <log path> ; sets APP_PID, LOGFILE. Env hooks per apps/mac/README.md.
  LOGFILE="$1"; rm -f "$LOGFILE"
  ( exec env FLASHTEX_AUTOATTACH=1 FLASHTEX_LOG="$LOGFILE" FLASHTEX_SEED_FILE="$SEED" FLASHTEX_BRIDGE="$BRIDGE_BIN" \
      FLASHTEX_EDIT_LEDGER="$LEDGER_BIN" FLASHTEX_BRIDGE_STORE="$RUN/app-store" "$APP" >"$LOGFILE.stdio" 2>&1 ) &
  APP_PID=$!
}
LAUNCH_CMD="FLASHTEX_AUTOATTACH=1 FLASHTEX_LOG=<log> FLASHTEX_SEED_FILE=Samples/demo.tex(+comment) FLASHTEX_BRIDGE=<built> FLASHTEX_EDIT_LEDGER=<built> FLASHTEX_BRIDGE_STORE=<tmp> $APP"
wait_log() { local pat="$1" secs="$2" d; d=$(( $(now_ms) + secs*1000 )); while [[ $(now_ms) -lt $d ]]; do grep -qF -- "$pat" "$LOGFILE" 2>/dev/null && return 0; sleep 0.1; done; return 1; }
child() { pgrep -P "$APP_PID" -f "$1" 2>/dev/null | head -1 || true; }
wait_window() { local d=$(( $(now_ms) + $1*1000 )) id; while [[ $(now_ms) -lt $d ]]; do id="$("$RUN/window_probe" "$APP_PID" 2>/dev/null | python3 "$RUN/pick_window.py")"; [[ -n "$id" ]] && { echo "$id"; return 0; }; sleep 0.25; done; echo ""; }
foreign_flashtex() { pgrep -x FlashTeX 2>/dev/null | grep -vx "${APP_PID:-0}" | tr '\n' ' ' || true; }

# ================================================================ (1) latency
echo "== (1) edit->visible over $LAUNCHES launches"
COMPILE_MS=(); COLD_NOTES=()
for i in $(seq 1 "$LAUNCHES"); do
  { cat "$SEED_SRC"; echo; echo "% rev5 edit $i $(date -u +%H:%M:%S)"; } >"$SEED"
  launch "$RUN/launch-$i.log"
  if wait_log "status: revision " 20 && grep -q 'diagnostics in' "$LOGFILE"; then
    ms="$(grep -o 'in [0-9]* ms' "$LOGFILE" | head -1 | grep -o '[0-9]*')"; COMPILE_MS+=("$ms")
  else COMPILE_MS+=("timeout"); fi
  [[ $i -eq 1 ]] && cp "$LOGFILE" "$RUN/first-launch.log"
  kill_own
done
LAT_SUMMARY="$(printf '%s\n' "${COMPILE_MS[@]}" | grep -v timeout | python3 "$RUN/stats.py")"
if printf '%s\n' "${COMPILE_MS[@]}" | grep -q timeout; then row FAIL "1 edit->visible via seed relaunch" "some launches never logged a result: ${COMPILE_MS[*]}"; else
row PASS "1 edit->visible: app-logged compile time 'revision N: ok ... in X ms' over $LAUNCHES relaunches with a modified Samples/demo.tex seed" "$LAT_SUMMARY; per launch: ${COMPILE_MS[*]} ms. Method: no Accessibility, so each 'edit' is a relaunch with an appended comment line; X is the app's own send->result timer for that launch's first compile, which includes cold worker start and the app's first render request, not a warm keystroke."; fi
cmd lat swift-real "$WT/apps/mac" -- env FLASHTEX_COMPILER="$COMPILER" swift test --filter RealCompilerTests \
  && row PASS "1 edit->visible: in-app debounce/coalescing path (swift test --filter RealCompilerTests)" "$(grep -h 'REAL-COMPILER LATENCY' "$LOGS/swift-real.log" | head -1); $(grep -h 'Executed' "$LOGS/swift-real.log" | tail -1)" \
  || row FAIL "1 in-app path: RealCompilerTests" "$(grep -h 'error' "$LOGS/swift-real.log" | head -2)"
cmd lat cli-latency "$RUN" -- python3 "$HERE/e2e_latency.py" --compiler "$BUNDLE/Contents/MacOS/flashtex-compiler" --document "$SEED_SRC" --count 20 --json "$RUN/cli-latency.json" \
  && row INFO "1 compiler round trip via the pipe, no UI (bundled flashtex-compiler, fresh process, 20x demo.tex): first request is the cold worker, the rest are warm" "$(grep LATENCY "$LOGS/cli-latency.log"); first request $(python3 -c 'import json,sys; print("%.1f" % json.load(open(sys.argv[1]))["samples_ms"][0])' "$RUN/cli-latency.json") ms"
cmd lat bridge "$RUN" -- python3 "$HERE/rev5_bridge.py" --bridge "$BRIDGE_BIN" --fixture "$WT/protocol/fixtures/capture-submission.json" --store "$RUN/bridge" --submits 10 --kill-after 5 --json "$RUN/bridge.json" \
  && row PASS "1 capture latency + disconnect/retry (real flashtex-bridge, Python JSON Lines client; the app's BridgeClient speaks the same envelopes)" "$(grep -h '^\[PASS\] capture latency' "$LOGS/bridge.log" | sed 's/^\[PASS\] //')" \
  || row FAIL "1 capture latency / disconnect" "$(grep -h '^\[FAIL\]' "$LOGS/bridge.log" | tr '\n' ' ')"
BRIDGE_ROWS="$(grep -h '^\[' "$LOGS/bridge.log")"
python3 - "$SEED_SRC" "$RUN/demo-req.json" <<'PY'
import json, sys
t = open(sys.argv[1], encoding="utf-8").read()
json.dump({"protocol_version": 1, "id": "demo", "type": "compile", "payload": {"project_id": "demo", "revision": 1, "entry_path": "main.tex", "documents": [{"path": "main.tex", "text": t}]}}, open(sys.argv[2], "w"), ensure_ascii=False)
PY
"$BUNDLE/Contents/MacOS/flashtex-compiler" <"$RUN/demo-req.json" >"$RUN/demo-result.json"
E1="$(python3 "$RUN/time_pdf.py" "$BUNDLE/Contents/MacOS/flashtex-pdf" "$RUN/demo-result.json" "$RUN/export-lm.pdf" 10 --default-face lm 2>&1)" && row PASS "1 export latency: bundled flashtex-pdf --verify --default-face lm x10 (demo.tex, 3 pages)" "$E1" || row FAIL "1 export latency (lm)" "$E1"
E2="$(python3 "$RUN/time_pdf.py" "$BUNDLE/Contents/MacOS/flashtex-pdf" "$RUN/demo-result.json" "$RUN/export-embed.pdf" 10 --embed-font auto --default-face lm 2>&1)" && row INFO "1 export latency: --embed-font auto --default-face lm x10 (embeds a Latin Modern .otf found on this machine; a font file, not TeX)" "$E2 ($(stat -f %z "$RUN/export-embed.pdf") bytes)" || row INFO "1 export --embed-font auto" "$E2"
cmd lat swift-pdf "$WT/apps/mac" -- env FLASHTEX_PDF="$PDF_BIN" swift test --filter 'PDFExportTests|RustPDFExportTests' \
  && row PASS "1 export: CoreGraphics path PDFExportTests + Rust-writer RustPDFExportTests" "$(grep -h "Test Case.*passed" "$LOGS/swift-pdf.log" | sed -E 's/.*\.(test[A-Za-z]+)\]. passed \(([0-9.]+) seconds\)\./\1 \2s/' | tr '\n' '; ' | cut -c1-500)" \
  || row FAIL "1 export tests" "$(grep -h 'error' "$LOGS/swift-pdf.log" | head -2)"

# ================================================================ (2) recovery
echo "== (2) recovery"
cp "$SEED_SRC" "$SEED"
launch "$RUN/recovery.log"; sleep 1
WIN="$(wait_window 10)"
wait_log "diagnostics in" 20 || true
C="$(child flashtex-compiler)"; B="$(child flashtex-bridge)"; L="$(child flashtex-edit-ledger)"
if [[ -n "$C" && -n "$B" && -n "$L" ]]; then row PASS "2 packaged app attaches all three helpers as children" "compiler $C, bridge $B, edit-ledger $L; window ${WIN:-none}; log: $(grep -h 'attached' "$LOGFILE" | cut -f2 | sort -u | tr '\n' ';')"; else row FAIL "2 helper children" "compiler=$C bridge=$B ledger=$L"; fi
SHOT="$REPORTS_DIR/rev5-$STAMP-packaged.png"
if [[ -n "$WIN" ]]; then screencapture -x -o -l "${WIN%% *}" "$SHOT" 2>/dev/null && { w=1200; while [[ $(stat -f %z "$SHOT") -gt 300000 && $w -gt 300 ]]; do sips -Z $w "$SHOT" --out "$SHOT" >/dev/null 2>&1; w=$((w-200)); done; row INFO "2 screenshot of the packaged window by id" "$(basename "$SHOT") $(stat -f %z "$SHOT") bytes"; }; fi
# compiler crash
kill -9 "$C"; if wait_log "worker exited" 10 && kill -0 "$APP_PID" 2>/dev/null; then row PASS "2 crash: SIGKILL compiler child -> app survives and logs the exit" "$(grep -h 'worker exited' "$LOGFILE" | tail -2 | cut -f2 | tr '\n' ';')"; else row FAIL "2 crash: compiler child" "alive=$(kill -0 "$APP_PID" 2>/dev/null && echo yes || echo no); tail: $(tail -2 "$LOGFILE" | cut -f2 | tr '\n' ';')"; fi
# bridge crash
before=$(wc -l <"$LOGFILE"); kill -9 "$B"; sleep 2
if kill -0 "$APP_PID" 2>/dev/null; then row PASS "2 crash: SIGKILL bridge child -> app survives" "log after kill: $(tail -n +$((before+1)) "$LOGFILE" | cut -f2 | sort -u | tr '\n' ';' | cut -c1-300)"; else row FAIL "2 crash: bridge child" "app exited"; fi
# ledger crash
before=$(wc -l <"$LOGFILE"); kill -9 "$L"; sleep 2
if kill -0 "$APP_PID" 2>/dev/null; then row PASS "2 crash: SIGKILL edit-ledger helper -> app survives" "log after kill: $(tail -n +$((before+1)) "$LOGFILE" | cut -f2 | sort -u | tr '\n' ';' | cut -c1-300)"; else row FAIL "2 crash: ledger helper" "app exited"; fi
cp "$LOGFILE" "$RUN/recovery-kept.log"; kill_own
launch "$RUN/relaunch.log"; sleep 3; wait_log "diagnostics in" 20 || true
C2="$(child flashtex-compiler)"; B2="$(child flashtex-bridge)"; L2="$(child flashtex-edit-ledger)"
if [[ -n "$C2" && -n "$B2" && -n "$L2" ]]; then row PASS "2 restart: relaunch re-attaches compiler, bridge and ledger" "children $C2/$B2/$L2; $(grep -h 'revision' "$LOGFILE" | tail -1 | cut -f2)"; else row FAIL "2 restart" "compiler=$C2 bridge=$B2 ledger=$L2"; fi
kill_own
cmd rec swift-recovery "$WT/apps/mac" -- swift test --filter 'BridgeRecoveryTests/(testTransientStatusFailuresRetainTransactionsAndEvidence|testDetachAndReattachIgnoresOldSessionEvents)' \
  && row PASS "2 disconnect/retry: cited Swift tests" "$(grep -h "Test Case.*passed" "$LOGS/swift-recovery.log" | sed -E 's/.*\.(test[A-Za-z]+)\]. passed \(([0-9.]+) seconds\)\./\1 \2s/' | tr '\n' '; ')" \
  || row FAIL "2 disconnect/retry tests" "$(grep -h 'error\|failed' "$LOGS/swift-recovery.log" | head -3)"
row INFO "2 disconnect/retry: real bridge" "$(printf '%s\n' "$BRIDGE_ROWS" | grep -h 'disconnect\|retry' | sed 's/^\[PASS\] //' | tr '\n' ';' | cut -c1-600)"
cmd rec swift-stale "$WT/apps/mac" -- swift test --filter 'testAutoCompileDebouncesAndCoalescesEdits' \
  && row PASS "2 stale preview: cited test testAutoCompileDebouncesAndCoalescesEdits" "$(grep -h "Test Case.*passed" "$LOGS/swift-stale.log" | head -1 | sed -E 's/.*\.(test[A-Za-z]+)\]. passed \(([0-9.]+) seconds\)\./\1 \2s/')" \
  || row FAIL "2 stale preview test" "$(grep -h 'error' "$LOGS/swift-stale.log" | head -2)"
cmd rec proto "$RUN" -- python3 "$HERE/check_protocol.py" --compiler "$BUNDLE/Contents/MacOS/flashtex-compiler" --repo "$WT" || true
row INFO "2 stale preview: log evidence of edit-while-compiling not obtainable" "no typing route without Accessibility and an external change to FLASHTEX_SEED_FILE does not recompile (verified); worker-side ordering from check_protocol.py: $(grep -h 'revision:' "$LOGS/proto.log" | sed 's/^\[[A-Z]*\] //' | tr '\n' ';' | cut -c1-300); the UI stale guard is the app's WorkerClientTests/ShellModelTests"
cmd rec swift-a11y "$WT/apps/mac" -- swift test --filter FlashTeXAccessibilityTests \
  && row PASS "2 accessibility: FlashTeXAccessibilityTests" "$(grep -h 'Executed' "$LOGS/swift-a11y.log" | tail -1); human VoiceOver script apps/mac/docs/accessibility.md NOT executed (needs a person)" \
  || row FAIL "2 accessibility tests" "$(grep -h 'error' "$LOGS/swift-a11y.log" | head -2)"
LC="$WT/apps/mac/scripts/launch-check.sh"
if grep -q -- '--install' "$LC"; then LC_INSTALL="--install"; else LC_INSTALL=""; fi
FOREIGN="$(foreign_flashtex)"
if [[ -n "$FOREIGN" ]]; then row INFO "2 update path: launch-check.sh${LC_INSTALL:+ $LC_INSTALL} x2" "SKIPPED: launch-check.sh runs 'pkill -x FlashTeX' and foreign FlashTeX process(es) $FOREIGN belong to another session on this Mac; ${LC_INSTALL:-no --install option at this ref}"
else
  # Three consecutive runs: the owner's script exits 0 even when a step FAILs,
  # so count its "- FAIL:" evidence lines and only PASS a run with none.
  for k in 1 2 3; do
    cmd upd "launch-check-$k" "$WT/apps/mac" -- bash "$LC" --app "$BUNDLE" --evidence "$RUN/launch-check-$k.md" ${LC_INSTALL:+$LC_INSTALL} || true
    nf="$(grep -c '^- FAIL' "$RUN/launch-check-$k.md" 2>/dev/null || true)"; nf="${nf:-0}"
    if [[ "$nf" == "0" ]]; then row PASS "2 update path: launch-check.sh run $k of 3 (app owner's script, launched via open)" "0 FAIL lines; ${LC_INSTALL:-no --install option at this ref, plain launch-check}"
    else row INFO "2 update path: launch-check.sh run $k of 3 had $nf FAIL line(s) (app owner's script; exit status was 0)" "$(grep -h '^- FAIL' "$RUN/launch-check-$k.md" | tr '\n' ';' | cut -c1-400); log: $(sed -n '/^```/,/^```/p' "$RUN/launch-check-$k.md" | grep -v '^```' | cut -f2 | tr '\n' ';' | cut -c1-300)"; fi
  done
fi

# ================================================================ (3) gaps
echo "== (3) gaps"
DEV="$(xcrun devicectl list devices 2>/dev/null | tail -n +3 | awk '{print $NF" "$(NF-2)}' | tr '\n' ';' || true)"
row INFO "3 gap: devices" "paired devices (state): $(xcrun devicectl list devices 2>/dev/null | tail -n +3 | grep -c . || echo 0); $(xcrun devicectl list devices 2>/dev/null | tail -n +3 | grep -o 'unavailable\|connected' | sort | uniq -c | tr '\n' ' '); simulators available: $(xcrun simctl list devices available 2>/dev/null | grep -c '(' ); no device or simulator run in this suite"
SP="$(spctl --assess --type execute -v "$BUNDLE" 2>&1 | tr '\n' ' ' || true)"; CS="$(codesign -dv "$BUNDLE" 2>&1 | grep -E 'Signature|TeamIdentifier' | tr '\n' ' ' || true)"
row INFO "3 gap: signing/notarization" "spctl: $SP; codesign: $CS"
row INFO "3 gap: provider" "$(printf '%s\n' "$BRIDGE_ROWS" | grep -h 'provider' | sed 's/^\[PASS\] //' | cut -c1-300); no XAI_API_KEY is used"
A11Y="$(osascript -e 'tell application "System Events" to tell process "Finder" to get name of every window' 2>&1 || true)"
row INFO "3 gap: visual" "Accessibility: $A11Y; capture only by window id ($(basename "$SHOT" 2>/dev/null)); preview face logged: $(grep -h 'preview face' "$RUN/first-launch.log" | head -1 | sed 's/.*(preview face: //; s/)//'); the preview draws with this compiler's metrics until the render pipeline lands"

# ================================================================ report
{
  echo "# FT-003 rev 5: packaged native recovery and responsiveness"
  echo
  echo "- Generated: $(date -u +%Y-%m-%dT%H:%M:%SZ) on $(hostname -s) (macOS $(sw_vers -productVersion)); $(xcodebuild -version 2>&1 | tr '\n' ' '); $(cargo --version); suite \`$(git -C "$REPO" rev-parse --short HEAD)\`"
  echo "- App tree: \`$APP_REF\` = \`$APP_SHA\`; bundle: \`apps/mac/scripts/make-app.sh ${MAKE_ARGS[*]//$WT\//}\` -> Contents/MacOS: $BUNDLED_LIST"
  echo "- Launch command (own instance, PID tracked): \`$LAUNCH_CMD\`"
  echo "- Overall: **$([[ $OVERALL -eq 0 ]] && echo PASS || echo FAIL)**; scratch \`$RUN\` (logs not committed)"
  echo
  echo "## Integrated component SHAs"
  echo
  echo "| Component | Last commit touching it in the app tree | Expected | Expected contained |"
  echo "|---|---|---|---|"
  printf '%s\n' "${COMP_ROWS[@]}"
  echo
  echo "\`Contents/Resources/components.json\`: ${COMPONENTS_JSON:-absent at this ref (SHAs computed from git)}"
  echo
  echo "## Checks"
  echo
  echo "| Status | Check | Detail |"
  echo "|---|---|---|"
  printf '%s\n' "${ROWS[@]}"
  echo
  echo "## First launch log (FLASHTEX_LOG, launch 1 of $LAUNCHES)"
  echo
  echo '```'; cat "$RUN/first-launch.log"; echo '```'
  echo
  echo "## Recovery launch log (compiler, bridge, ledger killed in turn)"
  echo
  echo '```'; cat "$RUN/recovery-kept.log"; echo '```'
  echo
  echo "## Bridge client rows"
  echo
  echo '```'; echo "$BRIDGE_ROWS"; echo '```'
  echo
  echo "## Commands"
  echo
  echo "| Scope | Command | Exit | Duration | Log |"; echo "|---|---|---|---|---|"
  printf '%s\n' "${CMDS[@]}" | sed "s#$WT/#<app>/#g; s#$HERE/#<suite>/#g"
} >"$REPORT"
echo "report: $REPORT"; echo "overall: $([[ $OVERALL -eq 0 ]] && echo PASS || echo FAIL)"
exit $OVERALL
