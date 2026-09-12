#!/usr/bin/env python3
"""Locally launched, bounded Codex worker. Git is its assignment/completion mailbox."""
import argparse
from datetime import datetime, timezone
import fcntl
import json
import os
from pathlib import Path
import signal
import subprocess
import sys
import time
from types import SimpleNamespace

import coord

SCHEMA = {'type': 'object', 'additionalProperties': False, 'properties': {
    'state': {'type': 'string', 'enum': ['in_progress', 'ready_for_integration', 'blocked']},
    'summary': {'type': 'string'}, 'next_action': {'type': 'string'},
    'eta_minutes': {'type': 'array', 'items': {'type': 'integer'}, 'minItems': 3, 'maxItems': 3},
    'tests': {'type': 'array', 'items': {'type': 'string'}},
    'reviewed_peers': {'type': 'array', 'items': {'type': 'string'}},
    'adaptation': {'type': 'string'}},
    'required': ['state', 'summary', 'next_action', 'eta_minutes', 'tests', 'reviewed_peers', 'adaptation']}


def assignment_for(fleet, agent):
    matches = [a['assignment'] for a in fleet['assignments']
               if a['assignment']['agent_id'] == agent and a['assignment']['state'] == 'assigned']
    if len(matches) > 1:
        raise ValueError('multiple active assignments; Commander must resolve this before starting')
    return matches[0] if matches else None


def read_state(root):
    path = coord.local_state(root) / 'worker.json'
    return path, json.loads(path.read_text()) if path.exists() else {'cycles': 0, 'phase': 'idle'}


def command_for(root, output, schema):
    # Saved account authentication is used. Never add API credentials or change provider.
    return ['codex', 'exec', '--sandbox', 'workspace-write', '-c', 'approval_policy="never"',
            '-c', 'sandbox_workspace_write.network_access=true', '--json',
            '--output-schema', str(schema), '--output-last-message', str(output), '-C', str(root), '-']


def execute(command, root, prompt, seconds, log):
    with log.open('w') as handle:
        process = subprocess.Popen(command, cwd=root, stdin=subprocess.PIPE, stdout=handle,
                                   stderr=subprocess.STDOUT, text=True, start_new_session=True)
        try:
            process.communicate(prompt, timeout=seconds)
        except (subprocess.TimeoutExpired, KeyboardInterrupt):
            os.killpg(process.pid, signal.SIGTERM)
            try:
                process.wait(timeout=10)
            except subprocess.TimeoutExpired:
                os.killpg(process.pid, signal.SIGKILL)
                process.wait()
            raise
        return process.returncode


def changed_paths(root):
    tracked = coord.git(root, 'diff', '--name-only', 'HEAD', '-z').split('\0')
    new = coord.git(root, 'ls-files', '--others', '--exclude-standard', '-z').split('\0')
    return sorted(set(p for p in tracked + new if p))


def paused_by_user(root, agent):
    path = 'coordination/control.json'
    if coord.run(['git', 'cat-file', '-e', 'origin/main:' + path], cwd=root, check=False).returncode:
        return False
    control = coord.peer_json(root, 'origin/main', path)
    return agent in control.get('paused_agents', [])


def cycle(root, args, fleet, state_path, state):
    if paused_by_user(root, args.id):
        return 'paused_by_user'
    assignment = assignment_for(fleet, args.id)
    if not assignment:
        return 'waiting_for_assignment'
    key = assignment['task_id'] + ':' + str(assignment['revision'])
    if state.get('completed_assignment') == key:
        return 'waiting_for_next_assignment'
    if state.get('phase') in ['model_running', 'model_failed', 'publication_started', 'publication_failed']:
        raise ValueError('previous execution needs reconciliation; inspect worker.json, logs, Git and running processes before another call')
    if coord.branch(root, args.id) != assignment['branch']:
        raise ValueError('start on the assigned branch in a clean dedicated checkout; no automatic checkout of existing work')
    if state.get('assignment') != key:
        state.update(assignment=key, task_cycles=0, task_started_utc=coord.stamp())
    elapsed = (datetime.now(timezone.utc) - coord.parse_time(state['task_started_utc'])).total_seconds()
    if not getattr(args, 'continuous', False) and (elapsed >= assignment['timebox_minutes'] * 60 or state.get('task_cycles', 0) >= args.max_cycles):
        return 'allocation_exhausted'
    if getattr(args, 'continuous', False) and args.max_cycles > 0 and state.get('task_cycles', 0) >= args.max_cycles:
        return 'allocation_exhausted'
    if coord.git(root, 'status', '--porcelain'):
        raise ValueError('preserve and reconcile existing dirty work before starting a cycle')
    record_path = root / 'coordination' / 'agents' / (args.id + '.json')
    if not record_path.exists():
        coord.register(root, SimpleNamespace(id=args.id, machine=args.machine, tool='Codex',
                       capability=args.capability, allocation=assignment['allocation_id']))
    coord.acknowledge(root, SimpleNamespace(id=args.id, task=assignment['task_id'], revision=assignment['revision'],
                      adaptation='Worker supervisor accepted this exact published assignment; Codex reads requirements before implementation.'))
    folder = coord.local_state(root)
    index = state['cycles'] + 1
    output, schema, log = folder / f'worker-{index}-result.json', folder / 'worker-schema.json', folder / f'worker-{index}.jsonl'
    coord.write_json(schema, SCHEMA)
    seconds = args.cycle_seconds if getattr(args, 'continuous', False) else max(1, min(args.cycle_seconds, int(assignment['timebox_minutes']*60-elapsed)))
    prompt = f'''You are worker {args.id} on {args.machine}, implementing assignment {key}.
Read AGENTS.md, ORCHESTRATION.md, coordination/COMMANDER.md, coordination/RESOURCES.md,
your exact assignment and relevant docs/contracts before editing. Existing task authorization
permits autonomous project work without questions. Work only in {json.dumps(assignment['owned_paths'])}.
Do not change shared control files. The supervisor prepared your registration/acknowledgement.
Use this cycle for at most {max(1, seconds-40)} seconds, then save a useful checkpoint.
Do not commit, push, stage, switch branches, merge, or invoke Cursor yourself: supervisor publishes
through actual Cursor after you return. Do not invoke Claude, nested agents, other paid tools,
API purchases, billing changes, or account resets. Missing permissions/auth/quota are a concrete
blocker: return blocked rather than waiting for a sleeping user or trying another funding source.
Fetch was performed by the supervisor. Read relevant peer diffs/handoffs from the fetched branches,
and report exact origin/ref=FULL_SHA values you actually reviewed plus resulting adaptations.
Never assume a fetched change was reviewed. Existing main and relevant peer source may have advanced;
if that changes an interface, adapt this task or explicitly report the dependency/blocker.
Assignment data: {json.dumps(assignment)}
Continue from your committed handoff and actual files. Implement and run meaningful checks.
Only report ready_for_integration when the assignment acceptance gate is met; it does not mean the
whole project is done. Return the requested JSON with honest tests, ETA optimistic/likely/pessimistic,
and exact next action. Preserve all unrelated work. Do not include secrets or raw transcripts.
'''
    state.update(phase='model_running', cycles=index, task_cycles=state['task_cycles']+1,
                 started_utc=coord.stamp(), model_log=str(log), output=str(output),
                 allocation=assignment['allocation_id'])
    coord.write_json(state_path, state)
    expected_head = coord.git(root, 'rev-parse', 'HEAD')
    expected_branch = coord.branch(root, args.id)
    code = execute(command_for(root, output, schema), root, prompt, seconds, log)
    if coord.git(root, 'rev-parse', 'HEAD') != expected_head or coord.branch(root, args.id) != expected_branch:
        raise ValueError('model changed Git HEAD/branch; preserve and inspect before publication')
    if code or not output.exists():
        state.update(phase='model_failed', exit_code=code)
        coord.write_json(state_path, state)
        raise RuntimeError('Codex failed; preserve work and report account/permission/error details from local log. No automatic paid retry or billing fallback.')
    result = json.loads(output.read_text())
    if result.get('state') not in ['in_progress', 'ready_for_integration', 'blocked']:
        raise ValueError('invalid worker result state')
    allowed = assignment['owned_paths'] + [f'coordination/agents/{args.id}.json']
    illegal = [p for p in changed_paths(root) if not any(coord.overlaps(p, a) and (p == a or p.startswith(a+'/')) for a in allowed)]
    if illegal:
        raise ValueError('unexpected changed paths; preserve and inspect: ' + repr(illegal))
    coord.report(root, SimpleNamespace(id=args.id, state=result['state'], summary=result['summary'],
                 next=result['next_action'], eta=result['eta_minutes'], usage='unknown',
                 evidence='; '.join(result['tests']), review=result['reviewed_peers'], adaptation=result['adaptation']))
    report = json.loads(record_path.read_text())
    report['supervisor'] = {'assignment': key, 'cycle': index, 'state': result['state'],
                            'last_completed_utc': coord.stamp(), 'tests': result['tests']}
    coord.write_json(record_path, report)
    coord.git(root, 'add', '--', *changed_paths(root))
    state.update(phase='publication_started', head_before_publish=coord.git(root, 'rev-parse', 'HEAD'))
    coord.write_json(state_path, state)
    try:
        coord.publish(root, SimpleNamespace(allocation=assignment['allocation_id']+'-cursor-checkpoint',
                      implementation='Codex', message=f'{assignment["task_id"]}: {result["state"]} checkpoint', timeout=180))
    except Exception:
        state['phase'] = 'publication_failed'
        coord.write_json(state_path, state)
        raise
    state.update(phase='idle', last_published_sha=coord.git(root, 'rev-parse', 'HEAD'), last_result=result['state'])
    if result['state'] in ['ready_for_integration', 'blocked']:
        state['completed_assignment'] = key  # Await changed dispatch; never repeat blocked model calls.
    coord.write_json(state_path, state)
    return result['state']


def main():
    import integrate
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--id', required=True); parser.add_argument('--machine', required=True)
    parser.add_argument('--capability', action='append', default=[])
    parser.add_argument('--max-cycles', type=int, default=0, help='optional per-assignment cycle cap; 0 continues within account quota')
    parser.add_argument('--bounded', dest='continuous', action='store_false', default=True)
    parser.add_argument('--cycle-seconds', type=int, default=240)
    parser.add_argument('--poll-seconds', type=int, default=30)
    parser.add_argument('--once', action='store_true')
    parser.add_argument('--check', action='append', default=[], help='additional integration validation JSON argv; include native/Rust builds when available')
    args = parser.parse_args()
    if not args.continuous and args.max_cycles == 0:
        args.max_cycles = 8
    try:
        coord.identifier(args.id)
        if args.max_cycles < 0 or min(args.cycle_seconds, args.poll_seconds) <= 0:
            raise ValueError('cycle/poll limits must be positive')
        root = coord.root_dir()
        auth = coord.run(['codex', 'login', 'status'], cwd=root)
        if 'ChatGPT' not in auth.stdout + auth.stderr:
            raise ValueError('this worker grant requires authenticated ChatGPT access; no API funding fallback configured')
        if os.environ.get('CODEX_API_KEY') or os.environ.get('OPENAI_API_KEY'):
            raise ValueError('API credential override present; use the approved subscription route or obtain a separate explicit API allocation')
        cursor_status = coord.run(['cursor-agent', 'status'], cwd=root)
        if 'Logged in' not in cursor_status.stdout + cursor_status.stderr:
            raise ValueError('Cursor authentication unavailable; report startup blocker before implementation spending')
        coord.current_git_user_trailer(root)  # Preflight human coauthor before model spending.
        folder = coord.local_state(root); folder.mkdir(parents=True, exist_ok=True)
        with (folder / 'worker.lock').open('a') as lock:
            fcntl.flock(lock, fcntl.LOCK_EX | fcntl.LOCK_NB)
            state_path, state = read_state(root)
            while True:
                fleet = coord.checkpoint(root, emit=False)
                control = coord.peer_json(root, 'origin/main', 'coordination/control.json')
                if control.get('state') == 'user_stopped' or args.id in control.get('paused_agents', []):
                    print('Project stop/completion control received; worker stopped.', flush=True)
                    break
                selected = assignment_for(fleet, args.id)
                if selected:
                    pointer = coord.peer_json(root, 'origin/main', f'coordination/next/{args.id}.json')
                    if pointer.get('task_id') != selected['task_id'] or pointer.get('revision') != selected['revision']:
                        raise ValueError('next-task pointer and assignment revision disagree; Commander must reconcile')
                if selected and state.get('phase') == 'idle' and not coord.git(root, 'status', '--porcelain') and state.get('completed_assignment') != selected['task_id'] + ':' + str(selected['revision']):
                    integrate.sync(root, SimpleNamespace(id=args.id, allocation=selected['allocation_id']+'-cursor-sync', check=args.check or ['["python3","-m","unittest","discover","-s","tests","-q"]'], timeout=180, check_timeout=300))
                status = cycle(root, args, fleet, state_path, state)
                print(json.dumps({'worker': args.id, 'status': status, 'utc': coord.stamp()}), flush=True)
                if args.once:
                    break
                time.sleep(min(args.poll_seconds, 60))
        return 0
    except (OSError, ValueError, RuntimeError, KeyError, subprocess.TimeoutExpired) as exc:
        print('worker: ' + str(exc), file=sys.stderr)
        # Report a concise blocker without sending raw logs, prompts, or credentials.
        try:
            root = coord.root_dir()
            folder = coord.local_state(root)
            marker = folder / 'worker-blocker-reported.json'
            if not marker.exists():
                body = folder / 'worker-blocker.md'
                body.write_text(f'Worker `{args.id}` on `{args.machine}` stopped with `{type(exc).__name__}`. Commander: inspect its published branch and request local supervisor state/log evidence. Existing work is preserved. No automatic billing fallback or repeated model call was attempted. This needs agent recovery, not a waiting permission prompt.\n')
                notice = coord.run(['gh','issue','create','--title',f'Worker recovery needed: {args.id}','--body-file',str(body)],cwd=root)
                coord.write_json(marker, {'reported_utc': coord.stamp(), 'issue': notice.stdout.strip()})
        except Exception as report_error:
            print('Could not publish recovery notice: ' + type(report_error).__name__, file=sys.stderr)
        return 1


if __name__ == '__main__':
    sys.exit(main())
