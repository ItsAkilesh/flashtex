#!/usr/bin/env bash
# Reproducible-build check for make-app.sh: runs it twice with identical
# arguments and compares the two FlashTeX.app bundles.
#
# Usage: apps/mac/scripts/repro-check.sh [--evidence <file>] [--keep] [-- <make-app.sh args...>]
#
# Gates (exit 1 when any fails):
#   1. every bundled helper in Contents/MacOS is byte-identical between the
#      two runs as shipped (after signing);
#   2. Contents/Resources/components.json is identical apart from
#      build.timestamp (the only time-dependent field make-app.sh writes) —
#      this includes app.sha256, the unsigned app executable's hash.
# Everything else that differs is listed with its reason (the app executable's
# signature seals CodeResources -> components.json -> timestamp, so as-signed
# it differs unless the timestamp is pinned). With SOURCE_DATE_EPOCH exported
# the two bundles must be identical in every file, and that is asserted too.
# --keep leaves the two bundle copies in place and prints their paths.
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
MAC_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
APP_DIR="$MAC_DIR/build/FlashTeX.app"

EVIDENCE_FILE=""
KEEP=0
MAKE_ARGS=()
while [[ $# -gt 0 ]]; do
  case "$1" in
    --evidence) EVIDENCE_FILE="$2"; shift 2 ;;
    --keep) KEEP=1; shift ;;
    --) shift; MAKE_ARGS=("$@"); break ;;
    -h|--help) sed -n '2,17p' "${BASH_SOURCE[0]}"; exit 0 ;;
    *) echo "repro-check.sh: unknown argument: $1 (pass make-app.sh arguments after --)" >&2; exit 1 ;;
  esac
done

WORK_DIR="$(mktemp -d "${TMPDIR:-/tmp}/flashtex-repro.XXXXXX")"
[[ "$KEEP" -eq 1 ]] || trap 'rm -rf "$WORK_DIR"' EXIT
RUN1="$WORK_DIR/run1/FlashTeX.app"
RUN2="$WORK_DIR/run2/FlashTeX.app"
mkdir -p "$WORK_DIR/run1" "$WORK_DIR/run2"

REPORT=()
say() { echo "$1"; REPORT+=("$1"); }
FAILURES=0

run_make() {  # $1=label $2=destination
  echo "==> make-app.sh run $1 (${MAKE_ARGS[*]:-no arguments})"
  "$SCRIPT_DIR/make-app.sh" ${MAKE_ARGS[@]+"${MAKE_ARGS[@]}"} > "$WORK_DIR/make-$1.log" 2>&1 \
    || { echo "make-app.sh run $1 failed; last lines:" >&2; tail -20 "$WORK_DIR/make-$1.log" >&2; exit 1; }
  # ditto preserves the code signature and xattrs exactly as written.
  ditto "$APP_DIR" "$2"
}

run_make 1 "$RUN1"
run_make 2 "$RUN2"

say "# make-app.sh reproducibility — $(date -u +%Y-%m-%dT%H:%M:%SZ)"
say ""
say "Arguments: ${MAKE_ARGS[*]:-(none)}"
say "SOURCE_DATE_EPOCH: ${SOURCE_DATE_EPOCH:-(unset)}"
say ""

# --- Gate 1: bundled helper copies byte-identical ---------------------------
# The helpers are signed before components.json is written, so their as-shipped
# bytes must match between runs. The app executable is signed LAST and its
# signature seals CodeResources -> components.json -> build.timestamp, so its
# as-signed bytes can only match when the timestamp is pinned; its unsigned
# bytes are compared through components.json's app.sha256 instead.
say "## Gate 1 — bundled helper copies byte-identical (as shipped, after signing)"
say ""
say "| file | run 1 sha256 | run 2 sha256 | result |"
say "|---|---|---|---|"
for f in "$RUN1"/Contents/MacOS/*; do
  name="$(basename "$f")"
  h1="$(shasum -a 256 "$f" | awk '{print $1}')"
  if [[ -f "$RUN2/Contents/MacOS/$name" ]]; then
    h2="$(shasum -a 256 "$RUN2/Contents/MacOS/$name" | awk '{print $1}')"
  else
    h2="(missing)"
  fi
  if [[ "$name" == "FlashTeX" ]]; then
    if [[ "$h1" == "$h2" ]]; then r="identical (signed)"; else r="differs as signed (seal chain; unsigned bytes checked in Gate 2)"; fi
  elif [[ "$h1" == "$h2" ]]; then r="identical"; else r="DIFFERENT"; FAILURES=$((FAILURES + 1)); fi
  say "| $name | $h1 | $h2 | $r |"
done
for f in "$RUN2"/Contents/MacOS/*; do
  name="$(basename "$f")"
  [[ -f "$RUN1/Contents/MacOS/$name" ]] || { say "| $name | (missing) | present | DIFFERENT |"; FAILURES=$((FAILURES + 1)); }
done
say ""

# --- Gate 2: components.json identical apart from build.timestamp ----------
say "## Gate 2 — components.json identical apart from build.timestamp"
say ""
COMP_RESULT="$(python3 - "$RUN1/Contents/Resources/components.json" "$RUN2/Contents/Resources/components.json" <<'PY'
import json, sys
a = json.load(open(sys.argv[1])); b = json.load(open(sys.argv[2]))
ta = a.get("build", {}).get("timestamp"); tb = b.get("build", {}).get("timestamp")
for d in (a, b):
    d.get("build", {}).pop("timestamp", None)
if a == b:
    print("PASS: identical after removing build.timestamp (run 1 %s, run 2 %s); app executable unsigned sha256 %s in both runs" % (ta, tb, a.get("app", {}).get("sha256")))
    sys.exit(0)
print("FAIL: components.json differs beyond build.timestamp:")
keys = sorted(set(a) | set(b))
for k in keys:
    if a.get(k) != b.get(k):
        print("  %s:\n    run 1: %s\n    run 2: %s" % (k, json.dumps(a.get(k), sort_keys=True), json.dumps(b.get(k), sort_keys=True)))
sys.exit(1)
PY
)" || FAILURES=$((FAILURES + 1))
say "$COMP_RESULT"
say ""

# --- Whole-bundle diff with reasons -----------------------------------------
say "## Every differing file (content comparison, timestamps ignored)"
say ""
DIFF_OUT="$WORK_DIR/diff.txt"
if diff -rq "$RUN1" "$RUN2" > "$DIFF_OUT" 2>&1; then
  say "No file differs: the two bundles are byte-identical."
  WHOLE_IDENTICAL=1
else
  WHOLE_IDENTICAL=0
  while IFS= read -r line; do
    rel="${line#Files $RUN1/}"; rel="${rel%% and *}"
    case "$rel" in
      Contents/Resources/components.json)
        reason="expected: build.timestamp is the wall-clock build time (pin with SOURCE_DATE_EPOCH)" ;;
      Contents/_CodeSignature/CodeResources)
        reason="expected consequence: the resource seal hashes components.json, so it changes whenever components.json does" ;;
      Contents/MacOS/FlashTeX)
        reason="expected consequence: the app signature (embedded in the main executable) seals CodeResources, which seals components.json; unsigned bytes are identical when Gate 2 passes" ;;
      *) reason="UNEXPLAINED" ;;
    esac
    say "- \`$rel\` — $reason"
  done < "$DIFF_OUT"
  grep -v '^Files ' "$DIFF_OUT" | while IFS= read -r line; do say "- $line"; done || true
fi
say ""
if [[ -n "${SOURCE_DATE_EPOCH:-}" && "$WHOLE_IDENTICAL" -ne 1 ]]; then
  say "FAIL: SOURCE_DATE_EPOCH is set, so the bundles were expected to be byte-identical in every file."
  FAILURES=$((FAILURES + 1))
fi

say "## Signature state of run 2"
say ""
say '```'
while IFS= read -r line; do say "$line"; done < <(codesign -dvv "$RUN2" 2>&1 | grep -E '^(Identifier|CodeDirectory|Signature|TeamIdentifier|Sealed Resources)' || true)
say '```'
say ""
if [[ "$FAILURES" -eq 0 ]]; then
  say "RESULT: PASS (helper copies byte-identical; components.json identical apart from build.timestamp$( [[ "$WHOLE_IDENTICAL" -eq 1 ]] && echo '; whole bundle byte-identical'))"
else
  say "RESULT: FAIL ($FAILURES gate failure(s))"
fi

if [[ -n "$EVIDENCE_FILE" ]]; then
  printf '%s\n' "${REPORT[@]}" > "$EVIDENCE_FILE"
  echo "==> Evidence written to $EVIDENCE_FILE"
fi
[[ "$KEEP" -eq 1 ]] && echo "==> Bundles kept: $RUN1 and $RUN2"
[[ "$FAILURES" -eq 0 ]]
