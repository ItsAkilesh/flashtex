#!/bin/zsh
# FlashTeX Commander deterministic supervisor. Launchd-managed, session-independent.
#
# DETERMINISTIC ONLY: no model calls of any kind. Reads git state via git/python,
# writes a factual assignment-state report. Decisions and analysis stay with the
# active Claude parent session; this script only detects and reports change.
REPO=/Users/kubar/code/flashtex
HOME_DIR=$HOME/flashtex-supervisor
LOG=$HOME_DIR/supervisor.log
REPORT=$HOME_DIR/latest-report.md
STATE=$HOME_DIR/last-state.txt
INTERVAL=${1:-90}

log() { echo "[$(date -u +%H:%M:%SZ)] $*" >> "$LOG"; }
echo "$$" > "$HOME_DIR/supervisor.pid"
log "supervisor start pid=$$ interval=${INTERVAL}s (deterministic, no model calls)"

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

write_report() {
  local now_epoch=$(date -u +%s)
  {
    echo "# Fleet report, deterministic, generated $(date -u +%H:%M:%SZ)"
    echo
    echo "main: $(git -C "$REPO" log -1 --format=%h origin/main 2>/dev/null), $(( (now_epoch - $(git -C "$REPO" log -1 --format=%ct origin/main 2>/dev/null)) / 60 ))m ago"
    echo
    echo "| task | rev | state | agent | branch age (min) |"
    echo "|---|---|---|---|---|"
    for f in $(git -C "$REPO" ls-tree --name-only origin/main coordination/assignments/ 2>/dev/null); do
      python3 -c "
import json,subprocess,sys
try:
    d=json.loads(subprocess.check_output(['git','-C','$REPO','show','origin/main:$f']))
except Exception:
    sys.exit()
agent=d.get('agent_id','')
age='?'
try:
    ts=subprocess.check_output(['git','-C','$REPO','log','-1','--format=%ct',
        f'origin/agent/{agent}']).decode().strip()
    if ts: age=str(($now_epoch - int(ts))//60)
except Exception:
    pass
print(f\"| {d['task_id']} | {d['revision']} | {d['state']} | {agent} | {age} |\")
" 2>/dev/null
    done
  } > "$REPORT.tmp"
  mv "$REPORT.tmp" "$REPORT" 2>/dev/null
}

while true; do
  NOW=$(snapshot)
  PREV=$(cat "$STATE" 2>/dev/null)
  MAIN=$(git -C "$REPO" log -1 --format=%h origin/main 2>/dev/null)

  if [ -n "$NOW" ] && [ "$NOW" != "$PREV" ]; then
    log "change detected (deterministic diff); writing report"
    echo "$NOW" > "$STATE"
    write_report
    log "report written to $REPORT (main=$MAIN)"
  else
    log "no change (main=$MAIN)"
  fi
  sleep "$INTERVAL"
done
