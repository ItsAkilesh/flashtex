#!/bin/zsh
# FlashTeX Commander supervisor — session-independent.
#
# Survives the Claude session that launched it. Polls the fleet, and when
# something changes, spends a Codex call to triage it. Codex carries the load
# because it has the usage headroom; the Commander reads the reports.
#
# SAFETY, deliberate and non-negotiable:
#   - read-only. No commits, no pushes, no killing or restarting anything.
#   - never infers an agent is dead from one observation; reports elapsed time.
#   - writes reports only; every decision stays with the Commander.
REPO=/Users/kubar/code/flashtex
HOME_DIR=$HOME/flashtex-supervisor
LOG=$HOME_DIR/supervisor.log
REPORT=$HOME_DIR/latest-report.md
STATE=$HOME_DIR/last-state.txt
INTERVAL=${1:-90}

log() { echo "[$(date -u +%H:%M:%SZ)] $*" >> "$LOG"; }
log "supervisor start pid=$$ interval=${INTERVAL}s"
echo "$$" > "$HOME_DIR/supervisor.pid"

snapshot() {
  git -C "$REPO" fetch origin --prune --quiet 2>/dev/null
  git -C "$REPO" ls-tree --name-only origin/main coordination/assignments/ 2>/dev/null \
    | while read -r f; do
        git -C "$REPO" show "origin/main:$f" 2>/dev/null | python3 -c "
import json,sys
try: d=json.load(sys.stdin)
except Exception: raise SystemExit
print(d['task_id'],d['revision'],d['state'],d['agent_id'])
" 2>/dev/null
      done | sort | tr '\n' ';'
}

while true; do
  NOW=$(snapshot)
  PREV=$(cat "$STATE" 2>/dev/null)
  MAIN=$(git -C "$REPO" log -1 --format=%h origin/main 2>/dev/null)

  if [ -n "$NOW" ] && [ "$NOW" != "$PREV" ]; then
    log "change detected; invoking codex triage"
    echo "$NOW" > "$STATE"
    codex exec --skip-git-repo-check --sandbox read-only --cd "$REPO" \
      -c model_reasoning_effort="low" \
      "You are the FlashTeX Commander's fleet triage agent. READ ONLY: make no
edits, commits, pushes, and never kill or restart anything.

Using git on origin/main and remote branches, produce a report under 40 lines:

1. Every assignment whose state is 'assigned': task id, revision, agent_id, and
   minutes since that agent's branch last committed.
2. Classify: ACTIVE (<45m), QUIET (45-120m), STALLED (>120m).
3. A section 'NEEDS COMMANDER ATTENTION' listing ONLY stalled assigned tasks and
   any assignment whose revision changed since the previous report.
4. One line on main: short sha and how many minutes since its last commit.

Report elapsed times as facts. Do NOT conclude an agent is dead: a quiet branch
may be completed work awaiting integration. State uncertainty plainly." \
      </dev/null 2>>"$LOG" | tail -n 60 > "$REPORT.tmp"
    mv "$REPORT.tmp" "$REPORT" 2>/dev/null
    log "triage written to $REPORT (main=$MAIN)"
  else
    log "no change (main=$MAIN)"
  fi
  sleep "$INTERVAL"
done
