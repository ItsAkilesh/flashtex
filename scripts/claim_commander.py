#!/usr/bin/env python3
"""Guarded one-shot successor claim after reviewed terminal witness.

Never calls a model. Requires fresh capacity evidence and exact reviewed witness
commit. A pending/uncertain claim is never retried. A non-force main push fences
competing successors; failed publication leaves the recovery journal intact.
"""
import argparse
from datetime import datetime, timezone
import json
import os
from pathlib import Path
import re
import subprocess
import tempfile
from select_successor import select
from commander_failover import validate_pin


def gate(config, authority, control, receipt, main_sha, candidates, machine, now):
    validate_pin(config['process'])
    if config.get('mode') != 'read_only_standby':
        raise ValueError('standby mode disabled')
    if control.get('state') != 'running':
        raise ValueError('project not running')
    if authority.get('commander_id') != config['predecessor_id'] or authority.get('authority_state') != 'active':
        raise ValueError('authority changed')
    if receipt.get('state') != 'terminal_quiescent_observed' or receipt.get('claim_authorized') is not False:
        raise ValueError('missing terminal witness')
    if receipt.get('predecessor_id') != config['predecessor_id'] or receipt.get('process_pin') != config['process']:
        raise ValueError('witness identity mismatch')
    if receipt.get('observed_main_sha') != main_sha:
        raise ValueError('witness main changed')
    age=(now-datetime.fromisoformat(receipt['observed_utc'].replace('Z','+00:00'))).total_seconds()
    if not 0 <= age <= 120:
        raise ValueError('stale or future witness')
    if any(receipt.get(k) != [] for k in ('reasons','publisher_processes','journal_blockers')):
        raise ValueError('witness has unresolved blockers')
    services=receipt.get('services')
    if not services or any(s.get('pid') != 0 or s.get('active') not in ('inactive','failed') for s in services):
        raise ValueError('publishers not stopped')
    if sorted(s.get('unit','') for s in services) != sorted(config['publisher_services']):
        raise ValueError('publisher service set mismatch')
    # The reviewed exact terminal witness supersedes older 'working' evidence
    # for this predecessor host only. Never prefer a dead predecessor record.
    available=[r for r in candidates if r.get('machine') != authority.get('machine')]
    decision=select(available,now)
    if decision['selected'] != machine:
        raise ValueError('this machine is not the selected successor: '+decision['reason'])
    return decision


def claim(repo, evidence_path, machine, successor, reviewed_witness, journal, executor, coauthor):
    if not re.fullmatch(r'[a-z0-9][a-z0-9-]{1,80}',successor):
        raise ValueError('invalid successor ID')
    if not re.fullmatch(r'[0-9a-f]{40}',reviewed_witness):
        raise ValueError('exact reviewed witness commit required')
    if journal.exists():
        raise ValueError('claim journal exists; reconcile before any retry')
    if evidence_path.stat().st_size > 1048576:
        raise ValueError('evidence too large')
    if any('\n' in v or '\r' in v or not v.strip() for v in (executor,coauthor)):
        raise ValueError('invalid truthful provenance')
    def git(*args,cwd=repo):
        return subprocess.check_output(['git',*args],cwd=cwd,text=True,timeout=45).strip()
    git('fetch','origin')
    base=git('rev-parse','origin/main')
    def record(path):
        return json.loads(git('show',base+':'+path))
    config=record('coordination/failover.json')
    if successor == config['predecessor_id']:
        raise ValueError('successor must have a unique identity')
    witness_ref='origin/'+config['witness_branch']
    if git('rev-parse',witness_ref) != reviewed_witness:
        raise ValueError('reviewed witness no longer branch tip')
    if git('show','-s','--format=%an <%ae>',reviewed_witness) != 'FlashTeX Witness <witness@flashtex.invalid>':
        raise ValueError('unexpected witness author')
    if 'Commit-Executor: commander_failover.py deterministic local witness' not in git('show','-s','--format=%B',reviewed_witness).splitlines():
        raise ValueError('unexpected witness executor')
    receipt=json.loads(git('show',reviewed_witness+':coordination/failover/witness-linux.json'))
    authority=record('coordination/authority.json')
    decision=gate(config,authority,record('coordination/control.json'),receipt,base,json.loads(evidence_path.read_text()),machine,datetime.now(timezone.utc))
    if git('show','-s','--format=%P',reviewed_witness) != base:
        raise ValueError('witness must directly parent observed main')
    state={'state':'pending','base_main':base,'witness':reviewed_witness,'successor':successor,'machine':machine,'decision':decision}
    journal.parent.mkdir(parents=True,exist_ok=True)
    with journal.open('x') as handle:
        json.dump(state,handle);handle.flush();os.fsync(handle.fileno())
    with tempfile.TemporaryDirectory(prefix='flashtex-claim-') as temporary:
        tree=Path(temporary)/'checkout'
        git('worktree','add','--detach',str(tree),base)
        try:
            authority.update(commander_id=successor,machine=machine,predecessor=config['predecessor_id'],claim_mode='verified_terminal_resource_selected',claim_base_main=base,witness_commit=reviewed_witness,updated_utc=datetime.now(timezone.utc).isoformat(),dispatch_service_pid_at_claim=0)
            authority.pop('agent_handle',None)
            (tree/'coordination/authority.json').write_text(json.dumps(authority,indent=2)+'\n')
            git('add','coordination/authority.json',cwd=tree)
            git('commit','-m','Claim Commander after verified terminal witness and resource selection','-m',f'Implementation-Agent: {executor}\nCommit-Executor: {executor}\nCo-authored-by: {coauthor}',cwd=tree)
            state['claim_commit']=git('rev-parse','HEAD',cwd=tree);journal.write_text(json.dumps(state,indent=2)+'\n')
            # Push is intentionally non-force. Any race rejects, never auto-merges.
            git('push','origin','HEAD:main',cwd=tree)
            git('fetch','origin')
            if git('rev-parse','origin/main') != state['claim_commit']:
                raise ValueError('claim no longer authoritative; remain quiesced')
            state['state']='published';journal.write_text(json.dumps(state,indent=2)+'\n')
            return state
        finally:
            git('worktree','remove',str(tree))

if __name__ == '__main__':
    p=argparse.ArgumentParser()
    for key in ('repo','evidence','machine','successor','reviewed-witness','journal','executor','coauthor'):
        p.add_argument('--'+key,required=True)
    a=p.parse_args()
    print(json.dumps(claim(Path(a.repo),Path(a.evidence),a.machine,a.successor,a.reviewed_witness,Path(a.journal),a.executor,a.coauthor),indent=2))
