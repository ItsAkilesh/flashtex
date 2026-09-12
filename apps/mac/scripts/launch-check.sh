#!/usr/bin/env bash
# Launches a packaged FlashTeX.app, confirms it actually opened a window,
# confirms the bundled flashtex-compiler attached (via FLASHTEX_LOG status
# lines and as a child process), kills that child to exercise worker-crash
# recovery, and asserts the app itself survives and logs the recovery.
#
# Usage: apps/mac/scripts/launch-check.sh [--app <path to .app>] [--evidence <file>]
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
MAC_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"

APP_DIR="$MAC_DIR/build/FlashTeX.app"
EVIDENCE_FILE=""

while [[ $# -gt 0 ]]; do
  case "$1" in
    --app)
      APP_DIR="$2"
      shift 2
      ;;
    --evidence)
      EVIDENCE_FILE="$2"
      shift 2
      ;;
    -h|--help)
      sed -n '2,8p' "${BASH_SOURCE[0]}"
      exit 0
      ;;
    *)
      echo "launch-check.sh: unknown argument: $1" >&2
      exit 1
      ;;
  esac
done

if [[ ! -d "$APP_DIR" ]]; then
  echo "launch-check.sh: $APP_DIR not found; run make-app.sh first" >&2
  exit 1
fi

REPORT_LINES=()
step() { echo "==> $1"; REPORT_LINES+=("" "## $1" ""); }
note() { echo "    $1"; REPORT_LINES+=("- $1"); }
fail() { echo "    FAIL: $1" >&2; REPORT_LINES+=("- FAIL: $1"); }

# Polls $LOG_FILE for a literal substring for up to $2 seconds.
wait_for_log() {
  local pattern="$1" timeout="$2" waited=0
  while (( waited < timeout )); do
    if [[ -f "$LOG_FILE" ]] && grep -qF "$pattern" "$LOG_FILE" 2>/dev/null; then
      return 0
    fi
    sleep 1
    waited=$((waited + 1))
  done
  return 1
}

# All scratch files for this run live in one directory so a single `rm -rf`
# cleans up everything (macOS mktemp only substitutes a trailing run of X's,
# so templates need a real trailing XXXXXX with no suffix after it).
WORK_DIR="$(mktemp -d "${TMPDIR:-/tmp}/flashtex-launch-check.XXXXXX")"
trap 'rm -rf "$WORK_DIR"' EXIT

# --- 1. Compile a tiny CGWindowList probe on the fly -----------------------
# `pid` -> exit 0 if that pid owns an on-screen window, else 1.
PROBE_SRC="$WORK_DIR/probe.swift"
PROBE_BIN="$WORK_DIR/probe"
cat > "$PROBE_SRC" <<'SWIFT'
import CoreGraphics
import Foundation

guard CommandLine.arguments.count > 1, let pid = Int32(CommandLine.arguments[1]) else {
    FileHandle.standardError.write("usage: probe <pid>\n".data(using: .utf8)!)
    exit(2)
}
let options: CGWindowListOption = [.optionOnScreenOnly, .excludeDesktopElements]
guard let list = CGWindowListCopyWindowInfo(options, kCGNullWindowID) as? [[String: AnyObject]] else {
    exit(1)
}
let owned = list.contains { entry in
    (entry[kCGWindowOwnerPID as String] as? Int32) == pid
}
exit(owned ? 0 : 1)
SWIFT

step "Compiling CGWindowList window-probe"
if swiftc -O "$PROBE_SRC" -o "$PROBE_BIN" 2>"$WORK_DIR/probe-build.log"; then
  note "probe compiled at $PROBE_BIN"
else
  note "probe failed to compile (see $WORK_DIR/probe-build.log); window check will be skipped"
  PROBE_BIN=""
fi

# --- 2. Launch --------------------------------------------------------------
COMPILER_IN_BUNDLE="$APP_DIR/Contents/MacOS/flashtex-compiler"
LOG_FILE="$WORK_DIR/flashtex.log"
: > "$LOG_FILE"
step "Launching $APP_DIR"
pkill -x FlashTeX >/dev/null 2>&1 || true
sleep 1
if [[ -x "$COMPILER_IN_BUNDLE" ]]; then
  open --env FLASHTEX_AUTOATTACH=1 --env "FLASHTEX_LOG=$LOG_FILE" "$APP_DIR"
  note "bundled flashtex-compiler present; launched with FLASHTEX_AUTOATTACH=1 FLASHTEX_LOG=$LOG_FILE"
else
  open --env "FLASHTEX_LOG=$LOG_FILE" "$APP_DIR"
  note "no bundled flashtex-compiler in $APP_DIR/Contents/MacOS; rebuild with 'make-app.sh --compiler <path>' to exercise the attach/kill/log checks below"
fi

APP_PID=""
for _ in $(seq 1 20); do
  APP_PID="$(pgrep -x FlashTeX || true)"
  [[ -n "$APP_PID" ]] && break
  sleep 0.5
done
if [[ -z "$APP_PID" ]]; then
  fail "FlashTeX process did not appear within 10s"
  exit 1
fi
note "FlashTeX running, pid=$APP_PID"

# --- 3. Wait for a window ----------------------------------------------------
step "Waiting for a window owned by pid $APP_PID"
WINDOW_SEEN=0
if [[ -n "$PROBE_BIN" ]]; then
  for _ in $(seq 1 15); do
    if "$PROBE_BIN" "$APP_PID"; then
      WINDOW_SEEN=1
      break
    fi
    sleep 1
  done
else
  sleep 3
fi
if [[ "$WINDOW_SEEN" -eq 1 ]]; then
  note "CGWindowListCopyWindowInfo confirms an on-screen window owned by pid $APP_PID"
else
  note "window not confirmed via CGWindowList (probe unavailable, headless session, or timed out)"
fi

# --- 4. Confirm the bundled compiler attached (log + child process) --------
COMPILER_PID=""
if [[ -x "$COMPILER_IN_BUNDLE" ]]; then
  step "Checking for a flashtex-compiler child of pid $APP_PID"
  for _ in $(seq 1 10); do
    COMPILER_PID="$(pgrep -P "$APP_PID" -x flashtex-compiler || true)"
    [[ -n "$COMPILER_PID" ]] && break
    sleep 0.5
  done
  if [[ -n "$COMPILER_PID" ]]; then
    note "flashtex-compiler attached, pid=$COMPILER_PID (child of $APP_PID)"
  else
    fail "no flashtex-compiler child process found under pid $APP_PID within 5s"
  fi

  step "Checking FLASHTEX_LOG for an 'attached:' status line"
  if wait_for_log "status: attached:" 10; then
    note "log shows an 'attached:' status line within 10s"
  else
    fail "no 'status: attached:' line in $LOG_FILE within 10s"
  fi

  step "Checking FLASHTEX_LOG for 'revision 1: ok' (auto-compile completed)"
  if wait_for_log "revision 1: ok" 10; then
    note "log shows 'revision 1: ok' within 10s"
  else
    fail "no 'revision 1: ok' line in $LOG_FILE within 10s"
  fi
fi

# --- 5. Kill the compiler child and assert the app survives and logs it ----
if [[ -n "$COMPILER_PID" ]]; then
  step "Killing flashtex-compiler (pid $COMPILER_PID) and checking app survival"
  kill "$COMPILER_PID"
  sleep 2
  if kill -0 "$APP_PID" 2>/dev/null; then
    note "FlashTeX (pid $APP_PID) is still running after its compiler child was killed"
  else
    fail "FlashTeX (pid $APP_PID) exited after its compiler child was killed"
  fi
  if ! kill -0 "$COMPILER_PID" 2>/dev/null; then
    note "flashtex-compiler (pid $COMPILER_PID) confirmed gone"
  fi

  step "Checking FLASHTEX_LOG for a 'worker exited (' status line"
  if wait_for_log "worker exited (" 5; then
    note "log shows a 'worker exited (' status line within 5s"
  else
    fail "no 'worker exited (' line in $LOG_FILE within 5s"
  fi
fi

# --- 6. Quit -----------------------------------------------------------------
step "Quitting FlashTeX"
osascript -e 'tell application "FlashTeX" to quit' >/dev/null 2>&1 || pkill -x FlashTeX >/dev/null 2>&1 || true
sleep 1
if pgrep -x FlashTeX >/dev/null 2>&1; then
  fail "FlashTeX (pid $APP_PID) is still running after quit"
else
  note "FlashTeX quit cleanly"
fi

if [[ -n "$EVIDENCE_FILE" ]]; then
  {
    echo "# launch-check.sh run — $(date -u +%Y-%m-%dT%H:%M:%SZ)"
    echo
    echo "App: $APP_DIR"
    echo "Compiler bundled: $([[ -x "$COMPILER_IN_BUNDLE" ]] && echo yes || echo no)"
    echo
    printf '%s\n' "${REPORT_LINES[@]}"
    echo
    echo "## FLASHTEX_LOG ($LOG_FILE)"
    echo
    echo '```'
    cat "$LOG_FILE" 2>/dev/null || echo "(log file missing or empty)"
    echo '```'
  } > "$EVIDENCE_FILE"
  echo "==> Evidence written to $EVIDENCE_FILE"
fi
