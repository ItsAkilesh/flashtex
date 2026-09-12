#!/usr/bin/env bash
# Independent, repeatable native acceptance runner for the FlashTeX Mac shell.
#
#   1. builds the Rust helpers (flashtex-compiler, flashtex-pdf, flashtex-bridge,
#      flashtex-edit-ledger, flashtex-preview-controller) from the CURRENT
#      integrated main (default origin/main) — a shared clone pinned (detached)
#      at that exact commit — and records SHAs + sha256;
#   2. builds the Mac app (release) from a pinned clone of the branch under
#      test (default origin/agent/mac-claude-a/mac-shell);
#   3. runs tools/typing-bench/run.sh (fixture / demo / body60k at 30 ms and
#      0 ms) with those helpers and records keystroke -> paint p50/p95/p99,
#      paints and coalesced keystrokes — once with the direct compiler worker
#      and once through the durable flashtex-preview-controller route
#      (FLASHTEX_PREVIEW_CONTROLLER, also built from main);
#   4. packages FlashTeX.app with apps/mac/scripts/make-app.sh and runs
#      apps/mac/scripts/launch-check.sh (compiler + bridge child kill, app
#      survival) with FLASHTEX_NO_ACTIVATE=1 through the `open` shim in lib/;
#   5. drives the packaged app's bundled helpers through the capture cycle
#      (submit -> convert refused as provider_disabled -> offline proposal
#      review -> durable ledger apply -> compile -> flashtex-pdf export);
#   6. writes reports/<UTC>.md with every number, hash, SHA, machine/OS/Xcode
#      version and exact command, asserted against thresholds.json.
#
# Usage: tools/native-validation/mac-live/run.sh [--branch <ref>] [--main-ref <ref>]
#          [--intervals "30 0"] [--seeds "fixture demo body60k"] [--no-fetch]
#          [--skip-bench] [--skip-controller] [--skip-launch] [--skip-cycle] [--rebuild] [--force]
#          [--work <dir>] [--session <url-or-id>] [--agent <id>]
# Env:   FLASHTEX_MAC_LIVE_SESSION  provenance: the driving agent session (URL/id)
#        FLASHTEX_MAC_LIVE_AGENT    provenance: the driving agent id
# Exit:  0 when every gate passed, 1 otherwise (the report is written either way).
# Never activates the app window; refuses launch-check while another
# FlashTeX.app is running (launch-check.sh would pkill it) unless --force.
set -uo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT="$(cd "$SCRIPT_DIR/../../.." && pwd)"
LIB="$SCRIPT_DIR/lib"
BRANCH="origin/agent/mac-claude-a/mac-shell"
MAIN_REF="origin/main"
INTERVALS="30 0"
SEEDS="fixture demo body60k"
DO_FETCH=1
SKIP_BENCH=0
SKIP_LAUNCH=0
SKIP_CYCLE=0
SKIP_CONTROLLER=0
REBUILD=0
FORCE=0
WORK="$SCRIPT_DIR/build"
SESSION="${FLASHTEX_MAC_LIVE_SESSION:-unknown}"
AGENT="${FLASHTEX_MAC_LIVE_AGENT:-unknown}"

while [[ $# -gt 0 ]]; do
  case "$1" in
    --branch) BRANCH="$2"; shift 2 ;;
    --main-ref) MAIN_REF="$2"; shift 2 ;;
    --intervals) INTERVALS="$2"; shift 2 ;;
    --seeds) SEEDS="$2"; shift 2 ;;
    --no-fetch) DO_FETCH=0; shift ;;
    --skip-bench) SKIP_BENCH=1; shift ;;
    --skip-launch) SKIP_LAUNCH=1; shift ;;
    --skip-cycle) SKIP_CYCLE=1; shift ;;
    --skip-controller) SKIP_CONTROLLER=1; shift ;;
    --rebuild) REBUILD=1; shift ;;
    --force) FORCE=1; shift ;;
    --work) WORK="$2"; shift 2 ;;
    --session) SESSION="$2"; shift 2 ;;
    --agent) AGENT="$2"; shift 2 ;;
    -h|--help) sed -n '2,32p' "${BASH_SOURCE[0]}"; exit 0 ;;
    *) echo "run.sh: unknown argument $1" >&2; exit 2 ;;
  esac
done

UTC="$(date -u +%Y%m%dT%H%M%SZ)"
REPORTS="$SCRIPT_DIR/reports"
RUN_DIR="$REPORTS/$UTC"
REPORT="$REPORTS/$UTC.md"
LOGS="$RUN_DIR/logs"
mkdir -p "$RUN_DIR" "$LOGS" "$WORK"
COMMANDS="$RUN_DIR/commands.log"
: > "$COMMANDS"

step() { echo "==> $*"; }
note() { echo "    $*"; }

# cmd <step-name> <command...>: logs the exact command, runs it with stdout+stderr
# captured to logs/<step>.log (also echoed), records exit code and seconds in
# steps.jsonl. Never aborts the runner; callers inspect $CMD_STATUS.
CMD_STATUS=0
cmd() {
  local name="$1"; shift
  local log="$LOGS/$name.log" start end
  printf '[%s] (cwd %s) %q' "$name" "$PWD" "$1" >> "$COMMANDS"
  local a; for a in "${@:2}"; do printf ' %q' "$a" >> "$COMMANDS"; done
  printf '\n' >> "$COMMANDS"
  start="$(date +%s)"
  "$@" > >(tee -a "$log") 2>&1
  CMD_STATUS=$?
  end="$(date +%s)"
  printf '{"step":"%s","exit":%d,"seconds":%d,"log":"logs/%s.log"}\n' "$name" "$CMD_STATUS" "$((end - start))" "$name" >> "$RUN_DIR/steps.jsonl"
  return 0
}

sha256() { shasum -a 256 "$1" | awk '{print $1}'; }

# ---------------------------------------------------------------- 0. provenance
step "provenance"
if [[ $DO_FETCH == 1 ]]; then cmd fetch git -C "$ROOT" fetch origin --prune --quiet; fi
MAIN_SHA="$(git -C "$ROOT" rev-parse --verify "$MAIN_REF^{commit}" 2>/dev/null)" || { echo "run.sh: cannot resolve $MAIN_REF" >&2; exit 2; }
BRANCH_SHA="$(git -C "$ROOT" rev-parse --verify "$BRANCH^{commit}" 2>/dev/null)" || { echo "run.sh: cannot resolve $BRANCH" >&2; exit 2; }
CHECKOUT_SHA="$(git -C "$ROOT" rev-parse HEAD)"
CHECKOUT_BRANCH="$(git -C "$ROOT" rev-parse --abbrev-ref HEAD)"
CHECKOUT_DIRTY="$(git -C "$ROOT" status --porcelain --untracked-files=no | wc -l | tr -d ' ')"
MAIN_IN_BRANCH="$(git -C "$ROOT" merge-base --is-ancestor "$MAIN_SHA" "$BRANCH_SHA" && echo true || echo false)"
MERGE_BASE="$(git -C "$ROOT" merge-base "$MAIN_SHA" "$BRANCH_SHA")"
note "main   $MAIN_REF = $MAIN_SHA"
note "branch $BRANCH = $BRANCH_SHA (contains main: $MAIN_IN_BRANCH; merge-base $MERGE_BASE)"
python3 - "$RUN_DIR/env.json" "$ROOT" "$SCRIPT_DIR" "$MAIN_REF" "$MAIN_SHA" "$BRANCH" "$BRANCH_SHA" "$CHECKOUT_SHA" "$CHECKOUT_BRANCH" "$CHECKOUT_DIRTY" "$MAIN_IN_BRANCH" "$MERGE_BASE" "$SESSION" "$AGENT" "$UTC" "$INTERVALS" "$SEEDS" <<'PY'
import json, os, subprocess, sys, hashlib, platform, glob
(out, root, here, main_ref, main_sha, branch, branch_sha, co_sha, co_branch, co_dirty, main_in_branch, merge_base, session, agent, utc, intervals, seeds) = sys.argv[1:18]
def sh(*a):
    try: return subprocess.run(a, text=True, capture_output=True, timeout=60).stdout.strip()
    except Exception as e: return "unavailable (%s)" % e
def sha(p):
    h = hashlib.sha256(); h.update(open(p, "rb").read()); return h.hexdigest()
tool_files = sorted(glob.glob(os.path.join(here, "run.sh")) + glob.glob(os.path.join(here, "thresholds.json")) + glob.glob(os.path.join(here, "lib", "*.py")) + glob.glob(os.path.join(here, "lib", "open-shim", "open")) + glob.glob(os.path.join(here, "fixtures", "*")))
env = {
  "utc": utc, "session": session, "agent": agent, "user": sh("id", "-un"), "host": sh("scutil", "--get", "LocalHostName"),
  "machine": {"hardware": sh("sysctl", "-n", "machdep.cpu.brand_string"), "arch": platform.machine(),
               "memory_bytes": sh("sysctl", "-n", "hw.memsize"), "cores": sh("sysctl", "-n", "hw.ncpu"),
               "os": sh("sw_vers", "-productVersion"), "os_build": sh("sw_vers", "-buildVersion"), "kernel": platform.release(),
               "xcode": " ".join(sh("xcodebuild", "-version").split()), "swift": sh("swift", "--version").splitlines()[0] if sh("swift", "--version") else "",
               "cargo": sh("cargo", "--version"), "rustc": sh("rustc", "--version"), "python3": sys.version.split()[0],
               "load_average_at_start": sh("sysctl", "-n", "vm.loadavg"),
               "other_flashtex_processes_at_start": sh("pgrep", "-l", "-x", "FlashTeX|FlashTeXMac").replace("\n", "; ")},
  "sources": {"main_ref": main_ref, "main_sha": main_sha, "branch": branch, "branch_sha": branch_sha,
               "branch_contains_main": main_in_branch == "true", "merge_base": merge_base,
               "main_subject": sh("git", "-C", root, "log", "-1", "--format=%s", main_sha),
               "branch_subject": sh("git", "-C", root, "log", "-1", "--format=%s", branch_sha),
               "main_commit_utc": sh("git", "-C", root, "log", "-1", "--format=%cI", main_sha),
               "branch_commit_utc": sh("git", "-C", root, "log", "-1", "--format=%cI", branch_sha)},
  "runner": {"checkout": root, "checkout_sha": co_sha, "checkout_branch": co_branch, "checkout_dirty_tracked_files": int(co_dirty),
              "files": {os.path.relpath(f, here): sha(f) for f in tool_files}},
  "parameters": {"intervals_ms": intervals.split(), "seeds": seeds.split()},
}
json.dump(env, open(out, "w"), indent=1, sort_keys=True)
PY

# --------------------------------------------------------------- 1. helpers
step "helpers from $MAIN_REF ($MAIN_SHA)"
HELPERS="$WORK/helpers/$MAIN_SHA"
HELPER_SRC="$HELPERS/src"
CRATES=(compiler pdf bridge edit-ledger preview-controller)
BUNDLED_CRATES=(compiler pdf bridge edit-ledger)
# The built helpers live where cargo would put them inside the pinned scratch
# clone, so make-app.sh's git lookup next to each binary resolves the real SHA.
helper_path() { echo "$HELPER_SRC/crates/$1/target/release/flashtex-$1"; }
HELPERS_OK=1
if [[ $REBUILD == 1 || ! -f "$HELPERS/.complete" ]]; then
  rm -rf "$HELPERS"; mkdir -p "$HELPERS"
  # A shared, detached clone pinned at the exact commit: a clean tree with no
  # local edits, no build products, and a truthful `git rev-parse HEAD`.
  cmd helpers-clone git clone -q --shared --no-checkout "$ROOT" "$HELPER_SRC"
  [[ $CMD_STATUS == 0 ]] && cmd helpers-checkout git -C "$HELPER_SRC" checkout -q --detach "$MAIN_SHA"
  [[ $CMD_STATUS == 0 ]] || HELPERS_OK=0
  export CARGO_TARGET_DIR="$WORK/cargo-target"
  for c in "${CRATES[@]}"; do
    [[ $HELPERS_OK == 1 ]] || break
    manifest="$HELPER_SRC/crates/$c/Cargo.toml"
    cmd "helpers-build-$c" cargo build --release --offline --manifest-path "$manifest" --bin "flashtex-$c"
    if [[ $CMD_STATUS != 0 ]]; then
      note "offline build of crates/$c failed; retrying with the registry (crates.io only, no paid service)"
      cmd "helpers-build-$c-online" cargo build --release --manifest-path "$manifest" --bin "flashtex-$c"
    fi
    [[ $CMD_STATUS == 0 ]] || { HELPERS_OK=0; break; }
    mkdir -p "$(dirname "$(helper_path "$c")")"
    cp "$CARGO_TARGET_DIR/release/flashtex-$c" "$(helper_path "$c")"
  done
  unset CARGO_TARGET_DIR
  [[ $HELPERS_OK == 1 ]] && touch "$HELPERS/.complete"
else
  note "reusing helpers already built from $MAIN_SHA in $HELPER_SRC (--rebuild to force)"
  printf '[helpers] reused %s\n' "$HELPER_SRC" >> "$COMMANDS"
fi
HELPER_HEAD="$(git -C "$HELPER_SRC" rev-parse HEAD 2>/dev/null || echo unknown)"
HELPER_CLEAN="$(git -C "$HELPER_SRC" status --porcelain 2>/dev/null | wc -l | tr -d ' ')"
python3 - "$RUN_DIR/helpers.json" "$HELPER_SRC" "$MAIN_REF" "$MAIN_SHA" "$HELPERS_OK" "$LIB" "$HELPER_HEAD" "$HELPER_CLEAN" "${CRATES[@]}" <<'PY'
import json, os, sys
out, src, ref, sha, ok, lib, head, dirty = sys.argv[1:9]; crates = sys.argv[9:]
sys.path.insert(0, lib)
from hashes import describe
bins = {}
for c in crates:
    d = describe(os.path.join(src, "crates", c, "target", "release", "flashtex-" + c))
    d.update({"crate": "crates/" + c, "git_sha": sha})
    bins["flashtex-" + c] = d
json.dump({"ref": ref, "sha": sha, "built_ok": ok == "1", "scratch_clone": src, "scratch_head": head,
           "scratch_dirty_entries": int(dirty), "binaries": bins}, open(out, "w"), indent=1, sort_keys=True)
PY
for c in "${CRATES[@]}"; do [[ -x "$(helper_path "$c")" ]] && note "flashtex-$c $(sha256 "$(helper_path "$c")")"; done
[[ "$HELPER_HEAD" == "$MAIN_SHA" ]] || { note "helpers scratch clone HEAD $HELPER_HEAD != $MAIN_SHA"; HELPERS_OK=0; }

# ------------------------------------------------------------------- 2. app
step "app from $BRANCH ($BRANCH_SHA)"
APP_SRC="$WORK/app/$BRANCH_SHA"
MAC="$APP_SRC/apps/mac"
APP_OK=1
if [[ $REBUILD == 1 || ! -f "$APP_SRC/.extracted" ]]; then
  rm -rf "$APP_SRC"
  cmd app-clone git clone -q --shared --no-checkout "$ROOT" "$APP_SRC"
  [[ $CMD_STATUS == 0 ]] && cmd app-checkout git -C "$APP_SRC" checkout -q --detach "$BRANCH_SHA"
  [[ $CMD_STATUS == 0 ]] && echo "$BRANCH_SHA" > "$APP_SRC/.extracted" || APP_OK=0
else
  note "reusing sources already checked out at $BRANCH_SHA in $APP_SRC"
  printf '[app] reused %s\n' "$APP_SRC" >> "$COMMANDS"
fi
APP_HEAD="$(git -C "$APP_SRC" rev-parse HEAD 2>/dev/null || echo unknown)"
[[ "$APP_HEAD" == "$BRANCH_SHA" ]] || { note "app scratch clone HEAD $APP_HEAD != $BRANCH_SHA"; APP_OK=0; }
# The step-1 helpers where typing-bench/run.sh looks by default (target/ is
# gitignored, so the pinned clone stays clean); the branch's own crates are
# never built here.
for c in "${CRATES[@]}"; do
  mkdir -p "$APP_SRC/crates/$c/target/release"
  [[ -x "$(helper_path "$c")" ]] && cp "$(helper_path "$c")" "$APP_SRC/crates/$c/target/release/flashtex-$c"
done
if [[ $APP_OK == 1 ]]; then
  cmd app-build swift build -c release --package-path "$MAC"
  [[ $CMD_STATUS == 0 ]] || APP_OK=0
fi
APP_BIN="$MAC/.build/release/FlashTeXMac"
if [[ $APP_OK == 1 && -x "$APP_BIN" ]]; then
  note "FlashTeXMac $(sha256 "$APP_BIN")"
  python3 -c 'import json,sys; sys.path.insert(0, sys.argv[5]); from hashes import describe; d=describe(sys.argv[2]); d.update({"branch":sys.argv[3],"sha":sys.argv[4],"built_ok":True,"binary":sys.argv[2]}); json.dump(d, open(sys.argv[1],"w"), indent=1, sort_keys=True)' "$RUN_DIR/app.json" "$APP_BIN" "$BRANCH" "$BRANCH_SHA" "$LIB"
else
  APP_OK=0
  python3 -c 'import json,sys; json.dump({"branch":sys.argv[2],"sha":sys.argv[3],"built_ok":False,"binary":None,"sha256":None}, open(sys.argv[1],"w"), indent=1)' "$RUN_DIR/app.json" "$BRANCH" "$BRANCH_SHA"
fi

# ---------------------------------------------------------- 3. typing bench
EXPECTED_CELLS=$(( $(wc -w <<< "$SEEDS") * $(wc -w <<< "$INTERVALS") ))
# bench_pass <name> <out-dir>: runs the bench once; if the app was killed
# mid-run by something outside this runner (other agents run pkill/launch
# checks on this machine) some cells have no summary — retry the whole pass
# once into <out-dir>/retry so the report can fill the gaps and say so.
bench_pass() {
  local name="$1" out="$2" found
  mkdir -p "$out"
  cmd "$name" bash "$APP_SRC/tools/typing-bench/run.sh" --no-render --producers compiler \
      --intervals "$INTERVALS" --seeds "$SEEDS" --out "$out/typing-bench.md"
  note "$name exit $CMD_STATUS"
  found=$(ls "$out"/typing-bench-*/*.json 2>/dev/null | wc -l | tr -d ' ')
  if (( found < EXPECTED_CELLS )); then
    note "$name: $found of $EXPECTED_CELLS cells have a summary (app killed or timed out mid-run); retrying the pass once"
    printf '[%s] retry: %s of %s cells had a summary\n' "$name" "$found" "$EXPECTED_CELLS" >> "$COMMANDS"
    mkdir -p "$out/retry"
    cmd "$name-retry" bash "$APP_SRC/tools/typing-bench/run.sh" --no-render --producers compiler \
        --intervals "$INTERVALS" --seeds "$SEEDS" --out "$out/retry/typing-bench.md"
    note "$name-retry exit $CMD_STATUS"
  fi
}
if [[ $SKIP_BENCH == 0 && $APP_OK == 1 && $HELPERS_OK == 1 ]]; then
  step "typing bench ($SEEDS × $INTERVALS ms)"
  bench_pass typing-bench "$RUN_DIR/typing-bench"
  if [[ $SKIP_CONTROLLER == 0 && -x "$(helper_path preview-controller)" ]]; then
    # Same bench, durable helper route: the app attaches flashtex-preview-controller
    # (which owns the ledger and launches the same compiler) instead of the direct
    # worker. run.sh passes the environment through to the app unchanged.
    step "typing bench via flashtex-preview-controller"
    CTRL_LEDGERS="$WORK/controller-ledgers-$UTC"
    mkdir -p "$CTRL_LEDGERS"
    export FLASHTEX_PREVIEW_CONTROLLER="$(helper_path preview-controller)" FLASHTEX_CONTROLLER_LEDGER_ROOT="$CTRL_LEDGERS"
    printf '[typing-bench-controller] FLASHTEX_PREVIEW_CONTROLLER=%q FLASHTEX_CONTROLLER_LEDGER_ROOT=%q\n' "$FLASHTEX_PREVIEW_CONTROLLER" "$CTRL_LEDGERS" >> "$COMMANDS"
    bench_pass typing-bench-controller "$RUN_DIR/typing-bench-controller"
    unset FLASHTEX_PREVIEW_CONTROLLER FLASHTEX_CONTROLLER_LEDGER_ROOT
    rm -rf "$CTRL_LEDGERS"
  fi
else
  step "typing bench skipped (skip=$SKIP_BENCH app_ok=$APP_OK helpers_ok=$HELPERS_OK)"
fi

# -------------------------------------------------------------- 4. package
BUNDLE="$MAC/build/FlashTeX.app"
BUNDLE_OK=0
if [[ $APP_OK == 1 && $HELPERS_OK == 1 ]]; then
  step "make-app.sh (bundle with the four helpers)"
  cmd make-app bash "$MAC/scripts/make-app.sh" --compiler "$(helper_path compiler)" --pdf "$(helper_path pdf)" \
      --bridge "$(helper_path bridge)" --ledger "$(helper_path edit-ledger)"
  [[ $CMD_STATUS == 0 && -x "$BUNDLE/Contents/MacOS/FlashTeX" ]] && BUNDLE_OK=1
  python3 - "$RUN_DIR/bundle.json" "$BUNDLE" "$BUNDLE_OK" "$LIB" <<'PY'
import json, os, sys
out, bundle, ok, lib = sys.argv[1:5]
sys.path.insert(0, lib)
from hashes import describe
macos = os.path.join(bundle, "Contents", "MacOS")
bins = {}
if os.path.isdir(macos):
    for n in sorted(os.listdir(macos)):
        p = os.path.join(macos, n)
        if os.path.isfile(p):
            bins[n] = describe(p)
comp = os.path.join(bundle, "Contents", "Resources", "components.json")
components = json.load(open(comp)) if os.path.isfile(comp) else None
plist = os.path.join(bundle, "Contents", "Info.plist")
json.dump({"path": bundle, "built_ok": ok == "1", "binaries": bins, "components_json": components,
           "info_plist_present": os.path.isfile(plist)}, open(out, "w"), indent=1, sort_keys=True)
PY
else
  step "make-app.sh skipped (app_ok=$APP_OK helpers_ok=$HELPERS_OK)"
fi

# --------------------------------------------------------- 5. launch check
if [[ $SKIP_LAUNCH == 0 && $BUNDLE_OK == 1 ]]; then
  step "launch-check.sh (FLASHTEX_NO_ACTIVATE=1 via lib/open-shim)"
  RUNNING="$(pgrep -x FlashTeX || true)"
  if [[ -n "$RUNNING" && $FORCE == 0 ]]; then
    note "REFUSED: FlashTeX.app already running (pid $RUNNING); launch-check.sh would pkill it. Use --force to override."
    printf '{"refused":true,"reason":"FlashTeX already running (pid %s)"}\n' "$RUNNING" > "$RUN_DIR/launch-check.json"
  else
    STORE="$RUN_DIR/launch-store"
    mkdir -p "$STORE"
    export FLASHTEX_MAC_LIVE_OPEN_ENV=$'FLASHTEX_NO_ACTIVATE=1\n'"FLASHTEX_BRIDGE_STORE=$STORE/captures"$'\n'"FLASHTEX_TRANSCRIPT=$RUN_DIR/launch-transcript.jsonl"
    export FLASHTEX_MAC_LIVE_OPEN_LOG="$RUN_DIR/launch-open.log"
    : > "$FLASHTEX_MAC_LIVE_OPEN_LOG"
    chmod +x "$LIB/open-shim/open"
    printf '[launch-check] PATH=%q:$PATH FLASHTEX_MAC_LIVE_OPEN_ENV=%q\n' "$LIB/open-shim" "$FLASHTEX_MAC_LIVE_OPEN_ENV" >> "$COMMANDS"
    PATH_SAVED="$PATH"; export PATH="$LIB/open-shim:$PATH"
    cmd launch-check bash "$MAC/scripts/launch-check.sh" --app "$BUNDLE" --evidence "$RUN_DIR/launch-check.md"
    LC_EXIT=$CMD_STATUS
    export PATH="$PATH_SAVED"
    unset FLASHTEX_MAC_LIVE_OPEN_ENV FLASHTEX_MAC_LIVE_OPEN_LOG
    python3 "$LIB/launch_summary.py" --evidence "$RUN_DIR/launch-check.md" --open-log "$RUN_DIR/launch-open.log" \
        --exit "$LC_EXIT" --out "$RUN_DIR/launch-check.json"
    rm -rf "$STORE"
  fi
else
  step "launch-check skipped (skip=$SKIP_LAUNCH bundle_ok=$BUNDLE_OK)"
fi

# --------------------------------------------------------- 6. capture cycle
if [[ $SKIP_CYCLE == 0 && $BUNDLE_OK == 1 ]]; then
  step "capture cycle through the bundled helpers"
  cmd capture-cycle python3 "$LIB/capture_cycle.py" --app "$BUNDLE" --fixtures "$APP_SRC/protocol/fixtures" \
      --proposal "$SCRIPT_DIR/fixtures/capture-proposal.json" --work "$RUN_DIR/capture-cycle" --out "$RUN_DIR/capture-cycle.json"
  note "capture-cycle exit $CMD_STATUS"
else
  step "capture cycle skipped (skip=$SKIP_CYCLE bundle_ok=$BUNDLE_OK)"
fi

# ---------------------------------------------------------------- 7. report
step "report"
printf '{"load_average_at_end":"%s","utc_end":"%s"}\n' "$(sysctl -n vm.loadavg)" "$(date -u +%Y-%m-%dT%H:%M:%SZ)" > "$RUN_DIR/end.json"
python3 "$LIB/report.py" --run-dir "$RUN_DIR" --thresholds "$SCRIPT_DIR/thresholds.json" --out "$REPORT"
STATUS=$?
note "report: $REPORT"
exit $STATUS
