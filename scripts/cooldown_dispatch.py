#!/usr/bin/env python3
"""One-shot deadline dispatch notification; no inference, purchases or authority claim.

Pending delivery is never retried blindly. Remote startup still requires local
capability/quota/ownership verification; a published dispatch is not a live PID.
"""
import argparse
from datetime import datetime, timezone
import json
from pathlib import Path
import subprocess


def due(schedule, now):
    deadline=datetime.fromisoformat(schedule['resume_utc'].replace('Z','+00:00'))
    return now >= deadline


def run(repo, schedule, journal, now):
    if not due(schedule, now):
        return {'state':'cooldown', 'resume_utc':schedule['resume_utc']}
    if journal.exists():
        return json.loads(journal.read_text())
    def git(*args):
        return subprocess.check_output(['git',*args],cwd=repo,text=True,timeout=30).strip()
    git('fetch','origin')
    authority=json.loads(git('show','origin/main:coordination/authority.json'))
    control=json.loads(git('show','origin/main:coordination/control.json'))
    if authority['commander_id'] != schedule['commander_id'] or authority['authority_state'] != 'active' or control.get('state') != 'running':
        return {'state':'blocked_authority_or_stop'}
    state={'state':'pending','observed_main':git('rev-parse','origin/main'),'resume_utc':schedule['resume_utc'],'delivered':[]}
    # Exclusive durable journal before any externally visible mutation.
    with journal.open('x') as f:
        json.dump(state,f); f.flush()
        import os
        os.fsync(f.fileno())
    for issue in schedule['dispatch_issues']:
        body=schedule['resume_prompt']
        result=subprocess.run(['gh','issue','comment',str(issue),'--repo','flash-tex/flashtex','--body',body],cwd=repo,text=True,capture_output=True,timeout=45,check=True)
        state['delivered'].append({'issue':issue,'url':result.stdout.strip()})
        journal.write_text(json.dumps(state,indent=2)+'\n')
    state['state']='published_dispatch'; journal.write_text(json.dumps(state,indent=2)+'\n')
    return state

if __name__ == '__main__':
    p=argparse.ArgumentParser(); p.add_argument('--repo',required=True); p.add_argument('--schedule',required=True); p.add_argument('--journal',required=True)
    a=p.parse_args(); print(json.dumps(run(Path(a.repo),json.loads(Path(a.schedule).read_text()),Path(a.journal),datetime.now(timezone.utc)),indent=2))
