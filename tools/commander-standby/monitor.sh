#!/usr/bin/env bash
# Read-only standby monitor for orchestrator-jaysen-opus (see tools/commander-standby/README.md).
#
# Every INTERVAL seconds (default 60) this script fetches origin and prints
# evidence about the active Commander. It NEVER writes repository content,
# never pushes, never commits, never touches coordination/* files, never
# comments on issues, and never dispatches. The only local side effect is
# `git fetch`, which updates remote-tracking refs in this checkout.
# It exits only on SIGINT (Ctrl-C) or SIGTERM.
set -u

INTERVAL="${STANDBY_MONITOR_INTERVAL:-60}"
ROOT="$(git -C "$(cd "$(dirname "$0")" && pwd)" rev-parse --show-toplevel 2>/dev/null)"
if [ -z "$ROOT" ]; then
  echo "monitor: not inside a git checkout" >&2
  exit 2
fi
REMOTE="${STANDBY_MONITOR_REMOTE:-origin}"
MAIN_REF="$REMOTE/main"
# Files the dispatcher/Commander write on main. Their newest commit is the
# dispatcher/publication evidence (author identity + timestamp), nothing more.
DISPATCH_PATHS=(coordination/queues coordination/next coordination/assignments)

stop() {
  echo
  echo "[$(date -u +%Y-%m-%dT%H:%M:%SZ)] monitor: signal received; exiting without any write."
  exit 0
}
trap stop INT TERM

show() { git -C "$ROOT" show "$MAIN_REF:$1" 2>/dev/null; }

json_fields() {
  # $1 = path on main, remaining args = top-level keys to print
  local path="$1"; shift
  show "$path" | python3 -c '
import json, sys
keys = sys.argv[1:]
try:
    d = json.load(sys.stdin)
except Exception as exc:
    print("  (unreadable: %s)" % exc); sys.exit(0)
for k in keys:
    print("  %s: %s" % (k, json.dumps(d.get(k))))
' "$@"
}

round=0
echo "monitor: read-only standby evidence loop; root=$ROOT remote=$REMOTE interval=${INTERVAL}s"
echo "monitor: writes: none (git fetch only). Stop with Ctrl-C."
while true; do
  round=$((round + 1))
  now="$(date -u +%Y-%m-%dT%H:%M:%SZ)"
  echo
  echo "===== round $round @ $now ====="
  if git -C "$ROOT" fetch --prune "$REMOTE" >/dev/null 2>&1; then
    fetch_state="ok"
  else
    fetch_state="FAILED (evidence below may be stale; a fetch failure is NOT takeover evidence)"
  fi
  echo "fetch: $fetch_state"

  echo "main tip:"
  git -C "$ROOT" log -1 --format='  %H%n  author=%an <%ae>%n  committed=%cI%n  subject=%s' "$MAIN_REF" 2>/dev/null || echo "  (unavailable)"

  echo "authority.json (on $MAIN_REF):"
  json_fields coordination/authority.json commander_id authority_state claim_mode claim_base_main \
    dispatch_service dispatch_service_pid_at_claim predecessor predecessor_main_writes_quiesced updated_utc

  echo "control.json (on $MAIN_REF):"
  json_fields coordination/control.json state stop_at_utc local_max_active_agents paused_agents

  echo "failover.json (on $MAIN_REF; Astra's pinned process + witness transport):"
  if show coordination/failover.json >/dev/null; then
    json_fields coordination/failover.json predecessor_id successor_id mode process publisher_services \
      quota_terminal_adapter mac_standby_ack witness_branch
    witness_branch="$(show coordination/failover.json | python3 -c 'import json,sys; print(json.load(sys.stdin).get("witness_branch",""))' 2>/dev/null)"
  else
    echo "  (absent)"
    witness_branch=""
  fi

  echo "terminal witness branches on $REMOTE (agent/orchestrator-witness/*):"
  witness_refs="$(git -C "$ROOT" ls-remote --heads "$REMOTE" 'refs/heads/agent/orchestrator-witness/*' 2>/dev/null)"
  if [ -n "$witness_refs" ]; then
    echo "$witness_refs" | sed 's/^/  /'
    if [ -n "$witness_branch" ]; then
      echo "  receipt $witness_branch:coordination/failover/witness-linux.json:"
      git -C "$ROOT" show "$REMOTE/$witness_branch:coordination/failover/witness-linux.json" 2>/dev/null | sed 's/^/    /' \
        || echo "    (branch listed remotely but not fetched yet or receipt missing; next round will show it)"
    fi
  else
    echo "  none (no terminal receipt has been published)"
  fi

  echo "dispatcher evidence (newest commit touching ${DISPATCH_PATHS[*]}):"
  git -C "$ROOT" log -1 --format='  %h author=%an <%ae> committed=%cI%n  subject=%s' "$MAIN_REF" -- "${DISPATCH_PATHS[@]}" 2>/dev/null || echo "  (unavailable)"
  for p in "${DISPATCH_PATHS[@]}"; do
    git -C "$ROOT" log -1 --format="  $p: %h %cI %an" "$MAIN_REF" -- "$p" 2>/dev/null
  done

  echo "recovery issues (open; title '[recovery]' or label 'recovery'):"
  if command -v gh >/dev/null 2>&1; then
    {
      gh issue list --state open --limit 100 --search '"[recovery]" in:title' --json number,title,updatedAt \
        --jq '.[] | "  #\(.number) \(.updatedAt) \(.title)"' 2>/dev/null
      gh issue list --state open --limit 100 --label recovery --json number,title,updatedAt \
        --jq '.[] | "  #\(.number) \(.updatedAt) \(.title)"' 2>/dev/null
    } | sort -u
    echo "  standby issue #20 last comment:"
    gh issue view 20 --json comments --jq '.comments[-1] | "    \(.createdAt) \(.author.login)"' 2>/dev/null || echo "    (unavailable)"
  else
    echo "  gh not available"
  fi

  echo "COMMANDER.md markers on $MAIN_REF (quiesce/handoff/successor lines):"
  show coordination/COMMANDER.md | grep -n -i -E 'quiesc|handoff|hand-off|successor|terminat|standby|takeover' | sed 's/^/  /' | head -20
  echo "COMMANDER-RESUME.md markers on $MAIN_REF:"
  show coordination/COMMANDER-RESUME.md | grep -n -i -E 'quiesc|handoff|hand-off|successor|terminat|standby|takeover' | sed 's/^/  /' | head -20

  echo "deterministic gate (standby_gate.py, read-only, never executes a claim):"
  python3 "$ROOT/tools/commander-standby/standby_gate.py" --repo "$ROOT" --remote "$REMOTE" --no-fetch \
    ${STANDBY_CLAIM_JOURNAL:+--journal "$STANDBY_CLAIM_JOURNAL"} 2>&1 \
    | python3 -c 'import json,sys
try:
    v=json.load(sys.stdin); print("  verdict=%s revival_permitted=%s executed=%s reasons=%s" % (v.get("verdict"), v.get("revival_permitted"), v.get("executed"), v.get("reasons")))
except Exception as e:
    print("  (gate output unreadable: %s)" % e)'
  echo "note: the gate verdict is a review flag. No takeover is executed or asserted by this script."
  sleep "$INTERVAL" &
  wait $!
done
