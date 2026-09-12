#!/bin/sh
# Live acceptance of the Mac's Grok/xAI wiring (apps/mac/docs/grok-live.md).
#
# Runs ONLY when an xAI key is present: the Keychain item
# tech.jay3332.flashtex.xai (account xai or XAI_API_KEY), or XAI_API_KEY / FLASHTEX_GROK_API_KEY
# in the environment. Without one it exits 3 and does nothing. With one it
# makes exactly three xAI requests through the Mac's own code paths:
#   probe        GET /v1/models                       (GrokProbe, HTTP class)
#   explanation  flashtex-assistant-context --provider-session   (one request)
#   conversion   flashtex-bridge --enable-grok capture_convert    (one request)
# and records ids/model/sizes/timings/HTTP class — never a key, prompt or reply
# body — under docs/evidence/grok-live-<UTC>/. The key is never printed, never
# put in argv, and never written to disk by this script.
#
# Supplying the key (either):
#   security add-generic-password -U -s tech.jay3332.flashtex.xai -a xai -w '<key>'
#   FlashTeX > Preferences (⌘,) > Grok (xAI) > xAI API key > Save to Keychain
# or, for one run only:  XAI_API_KEY='<key>' apps/mac/scripts/grok-live-check.sh
#
# Options: --no-build   do not (re)build the grok helper / bridge
set -eu
here=$(cd "$(dirname "$0")" && pwd)
root=$(cd "$here/../../.." && pwd)
mac="$root/apps/mac"
build=1
for arg in "$@"; do
  case "$arg" in
    --no-build) build=0 ;;
    -h|--help) sed -n '2,22p' "$0"; exit 0 ;;
    *) echo "unknown option: $arg" >&2; exit 2 ;;
  esac
done

# 1. Key presence (status only; -w is never used, so the secret is never read here).
source=""
if [ "${FLASHTEX_KEYCHAIN_OFF:-}" != "1" ] && { security find-generic-password -s tech.jay3332.flashtex.xai -a xai >/dev/null 2>&1 || security find-generic-password -s tech.jay3332.flashtex.xai -a XAI_API_KEY >/dev/null 2>&1; }; then
  source="Keychain"
elif [ -n "${XAI_API_KEY:-}" ]; then
  source="environment: XAI_API_KEY"
elif [ -n "${FLASHTEX_GROK_API_KEY:-}" ]; then
  source="environment: FLASHTEX_GROK_API_KEY"
fi
if [ -z "$source" ]; then
  echo "grok-live-check: no xAI key (Keychain item tech.jay3332.flashtex.xai (account xai / XAI_API_KEY) absent, XAI_API_KEY and FLASHTEX_GROK_API_KEY unset); nothing was run." >&2
  exit 3
fi
echo "grok-live-check: xAI key present ($source)"

# 2. Helpers: the grok-enabled assistant helper and the bridge, from this checkout.
helper="$root/crates/assistant-context/target/release/flashtex-assistant-context"
bridge="$root/crates/bridge/target/release/flashtex-bridge"
if [ "$build" = 1 ]; then
  (cd "$root/crates/assistant-context" && cargo build --release --features grok >/dev/null)
  (cd "$root/crates/bridge" && cargo build --release >/dev/null)
fi
for bin in "$helper" "$bridge"; do
  [ -x "$bin" ] || { echo "grok-live-check: missing $bin (run without --no-build)" >&2; exit 2; }
done
if ! "$helper" --provider-session probe grok-4.6 </dev/null 2>&1 | grep -q "explicit provider credential missing"; then
  echo "grok-live-check: $helper was not built with --features grok" >&2; exit 2
fi

# 3. Evidence directory.
stamp=$(date -u +%Y%m%dT%H%M%SZ)
evidence="$root/docs/evidence/grok-live-$stamp"
mkdir -p "$evidence"
sha=$(git -C "$root" rev-parse HEAD)
model="${FLASHTEX_GROK_MODEL:-grok-4.6}"
capture_model="${FLASHTEX_GROK_CAPTURE_MODEL:-grok-4.20-0309-non-reasoning}"
timeout_s="${FLASHTEX_ASSISTANT_TIMEOUT_S:-110}"

# 4. The three live calls, through the Mac test target (GrokLiveAcceptanceTests).
echo "grok-live-check: running GrokLiveAcceptanceTests (explanation model $model, capture model $capture_model, timeout ${timeout_s}s); evidence -> $evidence"
status=0
(cd "$mac" && env FLASHTEX_GROK_LIVE=1 FLASHTEX_GROK_EVIDENCE_DIR="$evidence" \
    FLASHTEX_ASSISTANT_CONTEXT="$helper" FLASHTEX_ASSISTANT_CONTEXT_GROK="$helper" FLASHTEX_BRIDGE="$bridge" \
    FLASHTEX_GROK_MODEL="$model" FLASHTEX_GROK_CAPTURE_MODEL="$capture_model" FLASHTEX_ASSISTANT_TIMEOUT_S="$timeout_s" FLASHTEX_ASSISTANT_PROVIDER=grok FLASHTEX_COMPILER="${FLASHTEX_COMPILER:-$root/crates/compiler/target/release/flashtex-compiler}" \
    swift test --filter GrokLiveAcceptanceTests 2>&1 | grep -v -i "api[_-]key" | tee "$evidence/swift-test.log" | grep -E "Test Case|Executed|error:") || status=$?

# 5. Summary (no bodies; the JSON files carry only ids/sizes/timings).
{
  echo "# Grok live check $stamp"
  echo
  echo "- commit: $sha"
  echo "- key source: $source (value never recorded)"
  echo "- explanation model: $model (helper timeout ${timeout_s}s)"
  echo "- capture model: $capture_model"
  echo "- helper: $helper (built --features grok)"
  echo "- bridge: $bridge"
  echo "- swift test status: $status (0 = all three live cases passed)"
  echo
  for f in probe explanation conversion; do
    if [ -f "$evidence/$f.json" ]; then
      echo "## $f"; echo; echo '```json'; cat "$evidence/$f.json"; echo; echo '```'; echo
    else
      echo "## $f"; echo; echo "not recorded (case skipped or failed before recording; see swift-test.log)"; echo
    fi
  done
  echo "xAI response ids and token usage are not surfaced by the helper/bridge yet (helper request R4 in apps/mac/docs/grok-live.md); the ids above are the Mac's session/request ids."
} > "$evidence/README.md"
if grep -q -i "xai-[A-Za-z0-9]" "$evidence"/*.json "$evidence/README.md" 2>/dev/null; then
  echo "grok-live-check: evidence looked key-like; removing $evidence" >&2
  rm -rf "$evidence"; exit 2
fi
echo "grok-live-check: evidence written to $evidence (status $status)"
exit $status
