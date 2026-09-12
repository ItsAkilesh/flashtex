#!/usr/bin/env bash
# Report-only probe of iOS/iPadOS devices and simulators reachable from this Mac
# (for the FT-004 companion / nearby-transfer work). Never fails; never pairs,
# boots or installs anything. Personal device names and hostnames are redacted
# in the committed report; model, state and identifier are kept.
#
# Usage: probe_devices.sh [--reports-dir <dir>] [--out <file>]
set -euo pipefail
HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPORTS_DIR="$HERE/reports"
OUT=""
while [[ $# -gt 0 ]]; do
  case "$1" in
    --reports-dir) REPORTS_DIR="$2"; shift 2 ;;
    --out) OUT="$2"; shift 2 ;;
    *) echo "unknown argument: $1" >&2; exit 2 ;;
  esac
done
STAMP="$(date -u +%Y%m%dT%H%M%SZ)"
[[ -n "$OUT" ]] || OUT="$REPORTS_DIR/devices-$STAMP.md"
mkdir -p "$(dirname "$OUT")"
TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT

{
  echo "# Device and simulator probe"
  echo
  echo "- Generated: $(date -u +%Y-%m-%dT%H:%M:%SZ) on $(hostname -s) (macOS $(sw_vers -productVersion)); $(xcodebuild -version 2>&1 | tr '\n' ' ')"
  echo "- Report only: nothing was paired, booted, installed or launched. Names/hostnames redacted."
  echo
  echo "## Paired physical devices (\`xcrun devicectl list devices --json-output\`)"
  echo
  if xcrun devicectl list devices --json-output "$TMP/devices.json" >"$TMP/devicectl.txt" 2>&1; then
    python3 - "$TMP/devices.json" <<'PY'
import json, sys
d = json.load(open(sys.argv[1]))
devs = d.get("result", {}).get("devices", [])
print("| # | Model | Identifier | Connection state | Pairing | OS |")
print("|---|---|---|---|---|---|")
for i, dev in enumerate(devs, 1):
    hw = dev.get("hardwareProperties", {})
    conn = dev.get("connectionProperties", {})
    props = dev.get("deviceProperties", {})
    print(f"| {i} | {hw.get('marketingName', '?')} ({hw.get('productType', '?')}) | `{dev.get('identifier', '?')}` | "
          f"{conn.get('tunnelState', '?')} / {conn.get('pairingState', '?')} | {conn.get('transportType', '?')} | "
          f"{props.get('osVersionNumber', '?')} |")
print()
print(f"{len(devs)} paired device(s). A state other than `connected` means the device is not reachable right now "
      "(not on USB and not awake on the same network); nothing here proves a working nearby channel.")
PY
  else
    echo '```'; cat "$TMP/devicectl.txt"; echo '```'
  fi
  echo
  echo "## Simulators (\`xcrun simctl list devices available\`)"
  echo
  echo '```'
  xcrun simctl list devices available 2>&1 | sed -E 's/\(([0-9A-F-]{36})\)/(<udid>)/' | head -40
  echo '```'
  echo
  echo "## Instruments device list (\`xcrun xctrace list devices\`, names redacted)"
  echo
  echo '```'
  xcrun xctrace list devices 2>&1 | sed -E 's/^[^(]*\(/<redacted> (/' | head -30
  echo '```'
  echo
  echo "## Local network (for the future paired-transfer channel)"
  echo
  echo "- Wi-Fi interface: $(networksetup -listallhardwareports 2>/dev/null | awk '/Wi-Fi/{getline; print $2}' | head -1 || echo unknown)"
  echo "- \`dns-sd\` available: $(command -v dns-sd >/dev/null && echo yes || echo no) (Bonjour browsing was not performed)"
  echo "- Local Network permission for the terminal host: not probed (requires a live connection attempt to a LAN peer)"
} >"$OUT"
echo "wrote $OUT"
