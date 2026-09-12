#!/usr/bin/env bash
# FlashTeX native validation: build the compiler and the Mac app from given git
# refs in temporary detached worktrees, run their test suites, run the
# runtime-v1 protocol checks and the UI capability probes, and write a Markdown
# report with exact commands, exit codes, durations and pass/fail per check.
#
# Usage:
#   run_all.sh [--repo <path>] [--mac-ref <git ref>] [--compiler-ref <git ref>]
#              [--scratch <dir>] [--keep] [--skip-xcodebuild] [--reports-dir <dir>]
#
# Defaults: --repo = git toplevel of this script's checkout,
#           --mac-ref origin/agent/mac-claude-a/mac-shell,
#           --compiler-ref origin/agent/claude/compiler-foundation,
#           --scratch ${TMPDIR:-/tmp}/flashtex-validation,
#           --reports-dir <repo>/tools/native-validation/reports.
# Exit status: 0 when every gating step passed, 1 otherwise. Never edits
# apps/mac or crates/compiler; worktrees are removed at the end unless --keep.
set -euo pipefail

HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO="$(git -C "$HERE" rev-parse --show-toplevel)"
MAC_REF="origin/agent/mac-claude-a/mac-shell"
COMPILER_REF="origin/agent/claude/compiler-foundation"
SCRATCH="${TMPDIR:-/tmp}/flashtex-validation"
KEEP=0
SKIP_XCODEBUILD=0
REPORTS_DIR=""
while [[ $# -gt 0 ]]; do
  case "$1" in
    --repo) REPO="$(cd "$2" && pwd)"; shift 2 ;;
    --mac-ref) MAC_REF="$2"; shift 2 ;;
    --compiler-ref) COMPILER_REF="$2"; shift 2 ;;
    --scratch) SCRATCH="$2"; shift 2 ;;
    --reports-dir) REPORTS_DIR="$2"; shift 2 ;;
    --keep) KEEP=1; shift ;;
    --skip-xcodebuild) SKIP_XCODEBUILD=1; shift ;;
    -h|--help) sed -n '2,20p' "$0"; exit 0 ;;
    *) echo "unknown argument: $1" >&2; exit 2 ;;
  esac
done
[[ -n "$REPORTS_DIR" ]] || REPORTS_DIR="$REPO/tools/native-validation/reports"

STAMP="$(date -u +%Y%m%dT%H%M%SZ)"
RUN_DIR="$SCRATCH/run-$STAMP"
LOG_DIR="$RUN_DIR/logs"
MAC_WT="$RUN_DIR/mac"
COMPILER_WT="$RUN_DIR/compiler"
REPORT="$REPORTS_DIR/report-$STAMP.md"
mkdir -p "$LOG_DIR" "$REPORTS_DIR"

MAC_SHA="$(git -C "$REPO" rev-parse --verify "$MAC_REF^{commit}")"
COMPILER_SHA="$(git -C "$REPO" rev-parse --verify "$COMPILER_REF^{commit}")"
SUITE_SHA="$(git -C "$REPO" rev-parse HEAD)"

# ---------------------------------------------------------------- reporting
declare -a STEP_ROWS=()
OVERALL=0
now_ms() { python3 -c 'import time; print(int(time.time()*1000))'; }

# run_step <name> <gating:1|0> <log name> <cwd> -- <command...>
# Records the exact command, exit code and duration. Gating failures set OVERALL=1.
run_step() {
  local name="$1" gating="$2" log="$3" cwd="$4"; shift 4
  [[ "$1" == "--" ]] && shift
  local cmd_text="$*"
  local logfile="$LOG_DIR/$log.log"
  local t0 t1 rc=0
  echo "== $name"
  echo "   cd $cwd && $cmd_text"
  t0=$(now_ms)
  ( cd "$cwd" && "$@" ) >"$logfile" 2>&1 || rc=$?
  t1=$(now_ms)
  local dur=$(( (t1 - t0) ))
  local status="PASS"
  if [[ $rc -ne 0 ]]; then
    status="FAIL"; [[ "$gating" == "1" ]] && OVERALL=1
  fi
  echo "   -> $status (exit $rc, $((dur/1000)).$((dur%1000/100)) s)"
  tail -5 "$logfile" | sed 's/^/   | /'
  STEP_ROWS+=("| $status | $name | \`$cmd_text\` | $rc | $((dur/1000)).$(printf '%03d' $((dur%1000))) s | \`logs/$log.log\` |")
  return 0
}

cleanup() {
  if [[ $KEEP -eq 1 ]]; then
    echo "keeping worktrees under $RUN_DIR"
    return
  fi
  for wt in "$MAC_WT" "$COMPILER_WT"; do
    if [[ -d "$wt" ]]; then
      git -C "$REPO" worktree remove --force "$wt" >/dev/null 2>&1 || rm -rf "$wt"
    fi
  done
  git -C "$REPO" worktree prune >/dev/null 2>&1 || true
}
trap cleanup EXIT

# ---------------------------------------------------------------- worktrees
echo "repo:      $REPO (suite at $SUITE_SHA)"
echo "mac:       $MAC_REF = $MAC_SHA"
echo "compiler:  $COMPILER_REF = $COMPILER_SHA"
echo "run dir:   $RUN_DIR"
run_step "worktree: compiler" 1 worktree-compiler "$REPO" -- git worktree add --detach "$COMPILER_WT" "$COMPILER_SHA"
run_step "worktree: mac" 1 worktree-mac "$REPO" -- git worktree add --detach "$MAC_WT" "$MAC_SHA"

COMPILER_BIN="$COMPILER_WT/crates/compiler/target/release/flashtex-compiler"
MAC_DIR="$MAC_WT/apps/mac"

# ---------------------------------------------------------------- compiler
if [[ -d "$COMPILER_WT/crates/compiler" ]]; then
  run_step "compiler: cargo build --release" 1 cargo-build "$COMPILER_WT/crates/compiler" -- cargo build --release
  run_step "compiler: cargo test --release" 1 cargo-test "$COMPILER_WT/crates/compiler" -- cargo test --release
else
  STEP_ROWS+=("| FAIL | compiler: crates/compiler present in $COMPILER_REF | - | - | - | - |"); OVERALL=1
fi

# ---------------------------------------------------------------- mac app
if [[ -d "$MAC_DIR" ]]; then
  run_step "mac: swift build" 1 swift-build "$MAC_DIR" -- swift build
  if [[ -x "$COMPILER_BIN" ]]; then
    run_step "mac: swift test (FLASHTEX_COMPILER set, real-compiler test enabled)" 1 swift-test "$MAC_DIR" -- \
      env FLASHTEX_COMPILER="$COMPILER_BIN" swift test
  else
    run_step "mac: swift test (no compiler binary; RealCompilerTests will be skipped)" 1 swift-test "$MAC_DIR" -- swift test
  fi
  if [[ $SKIP_XCODEBUILD -eq 0 ]]; then
    run_step "mac: xcodebuild build" 1 xcodebuild "$MAC_DIR" -- \
      xcodebuild -scheme FlashTeXMac -destination 'platform=macOS' -derivedDataPath "$RUN_DIR/DerivedData" build
  else
    STEP_ROWS+=("| SKIPPED | mac: xcodebuild build | --skip-xcodebuild | - | - | - |")
  fi
else
  STEP_ROWS+=("| FAIL | mac: apps/mac present in $MAC_REF | - | - | - | - |"); OVERALL=1
fi

# ---------------------------------------------------------------- protocol checks
PROTOCOL_JSON="$RUN_DIR/protocol.json"
if [[ -x "$COMPILER_BIN" ]]; then
  run_step "protocol: check_protocol.py" 1 check-protocol "$HERE" -- \
    python3 "$HERE/check_protocol.py" --compiler "$COMPILER_BIN" --repo "$COMPILER_WT" --json "$PROTOCOL_JSON"
else
  STEP_ROWS+=("| FAIL | protocol: check_protocol.py | compiler binary missing | - | - | - |"); OVERALL=1
fi

# ---------------------------------------------------------------- UI capability probes (never gating)
UI_MD="$RUN_DIR/ui-capabilities.md"
run_step "ui: check_ui_capabilities.sh (informational)" 0 check-ui "$HERE" -- \
  bash "$HERE/check_ui_capabilities.sh" --mac-dir "$MAC_DIR" --repo "$MAC_WT" --out "$UI_MD" --scratch "$RUN_DIR"

# ---------------------------------------------------------------- swift test summary
swift_summary() {
  local f="$LOG_DIR/swift-test.log"
  [[ -f "$f" ]] || { echo "(no swift test log)"; return; }
  grep -E "Executed [0-9]+ tests" "$f" | tail -1 || echo "(no XCTest summary line found)"
  if grep -q "RealCompilerTests" "$f"; then
    if grep -E "RealCompilerTests.*skipped" "$f" >/dev/null; then echo "RealCompilerTests: SKIPPED (FLASHTEX_COMPILER not usable)"; \
    else grep -E "Test Case.*RealCompilerTests.*(passed|failed)" "$f" | tail -1; fi
  fi
}

# ---------------------------------------------------------------- report
{
  echo "# FlashTeX native validation report"
  echo
  echo "- Generated: $(date -u +%Y-%m-%dT%H:%M:%SZ) on $(hostname -s) (macOS $(sw_vers -productVersion), $(uname -m))"
  echo "- Suite: \`tools/native-validation\` at \`$SUITE_SHA\`"
  echo "- Mac app ref: \`$MAC_REF\` = \`$MAC_SHA\`"
  echo "- Compiler ref: \`$COMPILER_REF\` = \`$COMPILER_SHA\`"
  echo "- Toolchain: $(swift --version 2>&1 | head -1); $(xcodebuild -version 2>&1 | tr '\n' ' '); $(cargo --version); $(python3 --version)"
  echo "- Worktrees: \`$MAC_WT\`, \`$COMPILER_WT\` (removed after the run unless \`--keep\`)"
  echo "- Full logs: \`$LOG_DIR\` (not committed)"
  echo "- Overall: **$([[ $OVERALL -eq 0 ]] && echo PASS || echo FAIL)** (gating steps only; UI probes are informational)"
  echo
  echo "## Steps"
  echo
  echo "| Status | Step | Command (run in the step's worktree dir) | Exit | Duration | Log |"
  echo "|---|---|---|---|---|---|"
  printf '%s\n' "${STEP_ROWS[@]}"
  echo
  echo "## swift test summary"
  echo
  echo '```'
  swift_summary
  echo '```'
  echo
  echo "## Protocol checks (check_protocol.py)"
  echo
  if [[ -f "$PROTOCOL_JSON" ]]; then
    python3 - "$PROTOCOL_JSON" <<'PY'
import json, sys
d = json.load(open(sys.argv[1]))
print("| Status | Check | Detail |")
print("|---|---|---|")
for s, n, det in d["rows"]:
    det = det.replace("|", "\\|").replace("\n", " ")
    if len(det) > 600: det = det[:600] + "…"
    print(f"| {s} | {n} | {det} |")
c = d["counts"]
print()
print(f"Counts: {c['PASS']} PASS, {c['FAIL']} FAIL, {c['INFO']} INFO.")
lat = d.get("latency")
if lat:
    print(f"\nEdit-to-result latency over {lat['count']} requests ({lat['document_bytes']}-byte document): "
          f"min {lat['min_ms']:.2f} ms, median {lat['median_ms']:.2f} ms, max {lat['max_ms']:.2f} ms "
          f"(subprocess pipe round trip; excludes UI work).")
PY
  else
    echo "(not run)"
  fi
  echo
  echo "## UI automation capabilities (check_ui_capabilities.sh)"
  echo
  if [[ -f "$UI_MD" ]]; then cat "$UI_MD"; else echo "(not run)"; fi
  echo
  echo "## Manual checklist"
  echo
  echo "See \`tools/native-validation/expectations.md\` for the human-only checks (visual click-to-source, dark preview versus export)."
} >"$REPORT"

echo
echo "report: $REPORT"
echo "overall: $([[ $OVERALL -eq 0 ]] && echo PASS || echo FAIL)"
exit $OVERALL
