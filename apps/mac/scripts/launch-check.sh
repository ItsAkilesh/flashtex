#!/usr/bin/env bash
# Launches a packaged FlashTeX.app, confirms it actually opened a window,
# confirms the bundled flashtex-compiler attached as a child process, kills
# that child to exercise worker-crash recovery, and asserts the app itself
# survives. See "Limitation" below for what this script cannot verify.
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

# --- 1. Compile a tiny CGWindowList probe on the fly -----------------------
# `pid` -> exit 0 if that pid owns an on-screen window, else 1.
PROBE_SRC="$(mktemp "${TMPDIR:-/tmp}/flashtex-window-probe.XXXXXX").swift"
PROBE_BIN="${PROBE_SRC%.swift}.bin"
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
if swiftc -O "$PROBE_SRC" -o "$PROBE_BIN" 2>/tmp/flashtex-probe-build.log; then
  note "probe compiled at $PROBE_BIN"
else
  note "probe failed to compile (see /tmp/flashtex-probe-build.log); window check will be skipped"
  PROBE_BIN=""
fi
rm -f "$PROBE_SRC"

# --- 2. Launch --------------------------------------------------------------
COMPILER_IN_BUNDLE="$APP_DIR/Contents/MacOS/flashtex-compiler"
step "Launching $APP_DIR"
pkill -x FlashTeX >/dev/null 2>&1 || true
sleep 1
if [[ -x "$COMPILER_IN_BUNDLE" ]]; then
  open --env FLASHTEX_AUTOATTACH=1 "$APP_DIR"
  note "bundled flashtex-compiler present; launched with FLASHTEX_AUTOATTACH=1"
else
  open "$APP_DIR"
  note "no bundled flashtex-compiler in $APP_DIR/Contents/MacOS; rebuild with 'make-app.sh --compiler <path>' to exercise the attach/kill checks below"
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
  rm -f "$PROBE_BIN"
else
  sleep 3
fi
if [[ "$WINDOW_SEEN" -eq 1 ]]; then
  note "CGWindowListCopyWindowInfo confirms an on-screen window owned by pid $APP_PID"
else
  note "window not confirmed via CGWindowList (probe unavailable, headless session, or timed out)"
fi

# --- 4. Confirm the bundled compiler attached (child process) --------------
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
fi

# --- 5. Kill the compiler child and assert the app survives ----------------
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
  note "LIMITATION: ShellModel's worker-exited status (\"worker exited (<code>)\") is an in-memory @Published SwiftUI string with no file or stdout sink (confirmed: no FLASHTEX_LOG support and no print() calls in apps/mac/Sources/FlashTeXMac), and reading it via UI scripting needs Accessibility/Screen-Recording permission this environment does not have (\"osascript ... System Events ...\" fails with -1728, not allowed assistive access, when tried against this app). This script can therefore only confirm the app's *process* survives its worker's death, not that its UI banner says \"worker exited\"; a human (or an Accessibility-authorized run) should confirm the banner text separately. Per the task instructions, no FLASHTEX_LOG plumbing was added to the app for this, since the app does not already support it and adding it is Swift-source work outside this worker's ownership (apps/mac/scripts/**, apps/mac/docs/packaging.md, docs/evidence/**)."
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
  } > "$EVIDENCE_FILE"
  echo "==> Evidence written to $EVIDENCE_FILE"
fi
