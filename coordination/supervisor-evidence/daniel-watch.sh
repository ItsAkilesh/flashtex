#!/bin/zsh
# Deterministic Daniel-lane watcher. No model calls; reports branch activity only.
REPO=/Users/kubar/code/flashtex
HOME_DIR=$HOME/flashtex-supervisor
LOG=$HOME_DIR/daniel-watch.log
REPORT=$HOME_DIR/daniel-latest.md
STATE=$HOME_DIR/daniel-last-state.txt
INTERVAL=${1:-90}

log() { echo "[$(date -u +%H:%M:%SZ)] $*" >> "$LOG"; }
echo "$$" > "$HOME_DIR/daniel-watch.pid"
log "daniel-watch start pid=$$ interval=${INTERVAL}s (deterministic, no model calls)"

snapshot() {
  git -C "$REPO" fetch origin --prune --quiet 2>/dev/null
  git -C "$REPO" branch -r --format='%(refname:short) %(objectname:short)' 2>/dev/null \
    | grep -i 'daniel' | sort | tr '\n' ';'
}

while true; do
  NOW=$(snapshot)
  PREV=$(cat "$STATE" 2>/dev/null)
  if [ -n "$NOW" ] && [ "$NOW" != "$PREV" ]; then
    log "daniel branch activity detected"
    echo "$NOW" > "$STATE"
    NOWE=$(date -u +%s)
    {
      echo "# Daniel lane report, deterministic, $(date -u +%H:%M:%SZ)"
      for b in $(git -C "$REPO" branch -r --format='%(refname:short)' | grep -i daniel); do
        ts=$(git -C "$REPO" log -1 --format='%ct' "$b" 2>/dev/null)
        [ -n "$ts" ] && echo "$(( (NOWE - ts) / 60 ))m  $b  $(git -C "$REPO" log -1 --format=%s "$b" 2>/dev/null | cut -c1-70)"
      done | sort -n
    } > "$REPORT.tmp"
    mv "$REPORT.tmp" "$REPORT" 2>/dev/null
    log "daniel report written"
  else
    log "no daniel change"
  fi
  sleep "$INTERVAL"
done
