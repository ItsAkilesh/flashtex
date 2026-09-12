#!/usr/bin/env bash
# Probe what UI automation this machine/session actually permits. Never fails
# the suite: every probe records OK / BLOCKED / SKIPPED plus the exact stderr.
#
# Usage: check_ui_capabilities.sh [--mac-dir <apps/mac checkout>] [--repo <repo root>]
#                                 [--out <markdown fragment>] [--scratch <dir>]
set -euo pipefail

MAC_DIR=""
REPO=""
OUT=""
SCRATCH="${TMPDIR:-/tmp}/flashtex-validation"
while [[ $# -gt 0 ]]; do
  case "$1" in
    --mac-dir) MAC_DIR="$2"; shift 2 ;;
    --repo) REPO="$2"; shift 2 ;;
    --out) OUT="$2"; shift 2 ;;
    --scratch) SCRATCH="$2"; shift 2 ;;
    *) echo "unknown argument: $1" >&2; exit 2 ;;
  esac
done
mkdir -p "$SCRATCH"
[[ -n "$OUT" ]] || OUT="$SCRATCH/ui-capabilities.md"

rows=()
record() { # status name detail
  rows+=("| $1 | $2 | $3 |")
  printf '[%s] %s -- %s\n' "$1" "$2" "$3"
}
esc() { printf '%s' "$1" | tr '\n' ' ' | sed 's/|/\\|/g' | cut -c1-400; }

# 1. Screen Recording: screencapture -x writes a PNG only when the process (the
#    terminal/agent host) has Screen Recording permission. Without it, recent
#    macOS releases either write a blank/desktop-only image or print an error.
#    The probe image is the user's whole screen, so it is deleted right after
#    it is measured; nothing captured here is kept or committed.
shot="$SCRATCH/screencapture-probe.png"
rm -f "$shot"
if err=$(screencapture -x "$shot" 2>&1); then
  if [[ -s "$shot" ]]; then
    size=$(stat -f %z "$shot")
    dims=$(sips -g pixelWidth -g pixelHeight "$shot" 2>/dev/null | awk '/pixel/ {printf "%s ", $2}')
    record OK "screencapture -x" "wrote $size bytes, ${dims}px (file deleted after measuring); a file does not prove window contents are visible: without Screen Recording macOS captures only wallpaper/blank"
  else
    record BLOCKED "screencapture -x" "exit 0 but no file written; stderr=$(esc "$err")"
  fi
else
  record BLOCKED "screencapture -x" "exit $?; stderr=$(esc "$err")"
fi
rm -f "$shot"

# 2. Accessibility / Automation: System Events scripting needs the Automation
#    permission for the calling app, and UI scripting needs Accessibility.
if err=$(osascript -e 'tell application "System Events" to get name of every process' 2>&1); then
  count=$(printf '%s' "$err" | tr ',' '\n' | wc -l | tr -d ' ')
  record OK "osascript System Events process list" "$count process names returned"
else
  record BLOCKED "osascript System Events process list" "stderr=$(esc "$err")"
fi
if err=$(osascript -e 'tell application "System Events" to tell process "Finder" to get name of every window' 2>&1); then
  record OK "osascript UI scripting (Accessibility)" "$(esc "$err")"
else
  record BLOCKED "osascript UI scripting (Accessibility)" "stderr=$(esc "$err")"
fi

# 3. iOS simulator availability (for the companion capture app, FT-004).
if out=$(xcrun simctl list devices available 2>&1); then
  devices=$(printf '%s\n' "$out" | grep -c '(' || true)
  first=$(printf '%s\n' "$out" | grep '(' | head -3 | sed 's/^ *//' | tr '\n' ';')
  if [[ "$devices" -gt 0 ]]; then
    record OK "xcrun simctl list devices available" "$devices device(s); first: $(esc "$first")"
  else
    record BLOCKED "xcrun simctl list devices available" "no available devices; output=$(esc "$(printf '%s' "$out" | head -5)")"
  fi
else
  record BLOCKED "xcrun simctl list devices available" "stderr=$(esc "$out")"
fi
if out=$(xcrun xctrace list devices 2>&1); then
  physical=$(printf '%s\n' "$out" | grep -v -i 'simulator' | grep -c '(' || true)
  record INFO "xcrun xctrace list devices (physical)" "$physical non-simulator entries (includes this Mac)"
else
  record INFO "xcrun xctrace list devices" "stderr=$(esc "$out")"
fi

# 4. Does the built app launch and stay alive for 3 seconds?
if [[ -n "$MAC_DIR" && -x "$MAC_DIR/.build/debug/FlashTeXMac" ]]; then
  log="$SCRATCH/app-launch.log"
  # exec so the backgrounded subshell *becomes* the app and $! is the app's pid
  # (without exec, $! is a wrapper shell and the app would outlive the kill).
  ( cd "$MAC_DIR" && exec env FLASHTEX_REPO="${REPO:-$MAC_DIR/../..}" .build/debug/FlashTeXMac >"$log" 2>&1 ) &
  pid=$!
  echo "$pid" >"$SCRATCH/app.pid"
  sleep 3
  if kill -0 "$pid" 2>/dev/null && pgrep -x FlashTeXMac >/dev/null; then
    record OK "FlashTeXMac launches and survives 3 s" "pid $pid; stderr/stdout=$(esc "$(head -c 300 "$log")")"
    # Process-level System Events queries work without Accessibility; window
    # enumeration does not. Both are recorded verbatim. Deliberately no
    # `set frontmost` here: that switches the user's active Space.
    if fm=$(osascript -e 'tell application "System Events" to get (frontmost of process "FlashTeXMac")' 2>&1); then
      record INFO "FlashTeXMac frontmost (System Events, no Accessibility needed)" "$fm"
    else
      record BLOCKED "FlashTeXMac frontmost (System Events)" "stderr=$(esc "$fm")"
    fi
    if wc_=$(osascript -e 'tell application "System Events" to get (count of windows of process "FlashTeXMac")' 2>&1); then
      record OK "FlashTeXMac window count (Accessibility)" "$wc_ window(s)"
    else
      record BLOCKED "FlashTeXMac window count (Accessibility)" "stderr=$(esc "$wc_")"
    fi
    # Only ever signal the instance this probe launched: other agents or the
    # user may have their own FlashTeXMac running on this machine.
    kill "$pid" 2>/dev/null || true
    sleep 0.5
    kill -0 "$pid" 2>/dev/null && kill -9 "$pid" 2>/dev/null || true
  else
    record BLOCKED "FlashTeXMac launches and survives 3 s" "process exited early; log=$(esc "$(head -c 400 "$log")")"
  fi
else
  record SKIPPED "FlashTeXMac launch" "no built binary at ${MAC_DIR:-<unset>}/.build/debug/FlashTeXMac (run swift build first)"
fi

# 5. Is there a GUI session at all? (Headless SSH sessions cannot show windows.)
if launchctl managername 2>/dev/null | grep -q Aqua; then
  record OK "GUI session (launchctl managername)" "Aqua"
else
  record INFO "GUI session (launchctl managername)" "$(launchctl managername 2>&1 || true)"
fi

{
  echo "| Status | Probe | Detail |"
  echo "|---|---|---|"
  printf '%s\n' "${rows[@]}"
} >"$OUT"
echo "wrote $OUT"
