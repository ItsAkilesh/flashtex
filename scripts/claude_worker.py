#!/usr/bin/env python3
"""Continuous API-only Opus task worker with verified bounded credit grants.

Uses shared assignment/report/Cursor publication helpers. Local subscription and
extra usage are prohibited; API credits require a verified funded allocation.
"""
import argparse
from datetime import datetime, timezone
import fcntl
import hashlib
import json
import math
import os
from pathlib import Path
import subprocess
import sys
import time
from types import SimpleNamespace

import coord
import integrate
import worker

OVERRIDES = ('ANTHROPIC_API_KEY', 'ANTHROPIC_AUTH_TOKEN', 'CLAUDE_CODE_OAUTH_TOKEN',
             'ANTHROPIC_BASE_URL', 'CLAUDE_CODE_USE_BEDROCK', 'CLAUDE_CODE_USE_VERTEX',
             'CLAUDE_CODE_USE_FOUNDRY')

def durable_state(path, value):
    coord.write_json(path, value)
    with path.open('rb') as handle:
        os.fsync(handle.fileno())
    directory = os.open(str(path.parent), os.O_RDONLY)
    try:
        os.fsync(directory)
    finally:
        os.close(directory)


def check_auth(root):
    # The latest user instruction is API-only on this Linux machine.
    if not os.environ.get('ANTHROPIC_API_KEY'):
        raise ValueError('Local Opus requires its authorized API key; subscription login is prohibited')
    if any(os.environ.get(name) for name in OVERRIDES if name != 'ANTHROPIC_API_KEY'):
        raise ValueError('API worker refuses OAuth/provider overrides')
    return {'route': 'api', 'credential_present': True, 'authentication_verified': False,
            'checked_utc': coord.stamp(), 'remaining_credits': 'unverified'}


def command_for():
    # Bare mode excludes subscription OAuth; safe mode skips custom hooks/plugins.
    # Explicit startup reads are in the prompt because auto-discovery is disabled.
    return ['claude', '-p', '--model', 'opus', '--effort', 'high', '--safe-mode', '--bare',
            '--permission-mode', 'dontAsk', '--permission-prompts', 'none',
            '--allowedTools', 'Read,Write,Edit,Glob,Grep,Bash',
            '--tools', 'Read,Write,Edit,Glob,Grep,Bash',
            '--strict-mcp-config', '--mcp-config', '{"mcpServers":{}}',
            '--setting-sources', '', '--output-format', 'json',
            '--json-schema', json.dumps(worker.SCHEMA), '--no-session-persistence']


def funded_call(root, args, assignment, state):
    """Choose an explicit route; never treat a key or plan label as credits."""
    if getattr(args, 'funding', 'api') != 'api':
        raise ValueError('Local Claude is API-only; subscription usage is prohibited')
    check_auth(root)
    if not args.api_grant:
        raise ValueError('API execution requires an explicit verified credit grant')
    grant = json.loads(Path(args.api_grant).read_text())
    fingerprint = hashlib.sha256(os.environ['ANTHROPIC_API_KEY'].encode()).hexdigest()
    if (grant.get('verified_api_auth') is not True
            or grant.get('credential_sha256') != fingerprint
            or grant.get('verified_prepaid_credits') is not True
            or grant.get('provider_spend_cap_verified') is not True
            or grant.get('agent_id') != args.id
            or grant.get('allocation_id') != assignment['allocation_id']
            or grant.get('worktree') != str(root.resolve())):
        raise ValueError('API credit grant is unverified or belongs to another allocation/worktree')
    if coord.parse_time(grant['expires_utc']) <= datetime.now(timezone.utc):
        raise ValueError('API credit grant expired')
    for field in ('max_total_usd', 'max_call_usd'):
        if type(grant.get(field)) not in (int, float) or not 0 < grant[field] <= 1000:
            raise ValueError('invalid API credit limit')
    if not os.environ.get('ANTHROPIC_API_KEY') or any(os.environ.get(n) for n in OVERRIDES if n != 'ANTHROPIC_API_KEY'):
        raise ValueError('API route needs only its authorized API key, without provider/OAuth overrides')
    grants = state.setdefault('credit_reservations', {})
    old = grants.get(assignment['allocation_id'], 0)
    if type(old) not in (int, float) or not math.isfinite(old) or old < 0:
        raise ValueError('invalid existing credit reservation; audit before spending')
    limit = min(grant['max_call_usd'], grant['max_total_usd'] - old)
    if limit <= 0:
        raise ValueError('API grant exhausted or held in unresolved reservations')
    # Reserve whole call caps. Client cost estimates are not sufficient evidence
    # to release money; only an audited provider reconciliation can release it.
    grants[assignment['allocation_id']] = old + limit
    command = command_for() + ['--max-budget-usd', str(limit)]
    return {'route': 'verified_prepaid_api', 'allocation_id':assignment['allocation_id'],
            'call_cap_usd':limit, 'checked_utc':coord.stamp()}, command


def read_result(log):
    # execute() combines stderr and stdout. Accept one result record, ignore only
    # non-JSON startup notices; do not guess successful work from plain prose.
    raw = log.read_text()
    try:
        data = json.loads(raw)
    except ValueError:
        records = []
        for line in raw.splitlines():
            try:
                value = json.loads(line)
                if isinstance(value, dict) and value.get('type') == 'result':
                    records.append(value)
            except ValueError:
                pass
        if len(records) != 1:
            raise ValueError('missing or ambiguous Claude result record; preserve local log')
        data = records[0]
    if not isinstance(data, dict) or data.get('is_error') or data.get('subtype') != 'success':
        raise ValueError('Claude reported a failed/limited cycle; reconcile usage and work before retry')
    result = data.get('structured_output')
    validate_result(result)
    return result, usage_summary(data)


def validate_result(result):
    if not isinstance(result, dict) or set(result) != set(worker.SCHEMA['required']):
        raise ValueError('invalid structured Claude result fields')
    if result['state'] not in ('in_progress', 'ready_for_integration', 'blocked'):
        raise ValueError('invalid Claude result state')
    eta = result['eta_minutes']
    if not isinstance(eta, list) or len(eta) != 3 or any(type(n) is not int or n < 0 for n in eta) or eta != sorted(eta):
        raise ValueError('invalid optimistic/likely/pessimistic ETA')
    for name in ('summary', 'next_action', 'adaptation'):
        if not isinstance(result[name], str) or len(result[name]) > 16000:
            raise ValueError('invalid result text')
    for name in ('tests', 'reviewed_peers'):
        if not isinstance(result[name], list) or len(result[name]) > 128 or any(not isinstance(x, str) or len(x)>4096 for x in result[name]):
            raise ValueError('invalid result list')


def usage_summary(data):
    # Never publish arbitrary provider output, prompts or credential metadata.
    result = {'pool': 'local-claude-api-unassigned', 'remaining_quota': 'unknown',
              'api_credit_balance': 'unverified', 'source': 'Claude CLI result',
              'reported_utc': coord.stamp()}
    value = data.get('total_cost_usd')
    if type(value) in (int, float) and value >= 0:
        result['client_estimated_equivalent_usd'] = value  # not subscription spending
    usage = data.get('usage', {})
    result['tokens'] = {k: usage[k] for k in ('input_tokens', 'output_tokens',
        'cache_creation_input_tokens', 'cache_read_input_tokens')
        if isinstance(usage, dict) and type(usage.get(k)) is int and usage[k] >= 0}
    return result


def cycle(root, args, assignment, state_path, state):
    key = assignment['task_id'] + ':' + str(assignment['revision'])
    if state.get('pending_blocker'):
        report_blocker(root,args,RuntimeError('Worker reported blocked; inspect its published handoff'))
        state.pop('pending_blocker')
        durable_state(state_path,state)
    if state.get('completed_assignment') == key:
        return 'waiting_for_next_assignment'
    if state.get('phase', 'idle') != 'idle':
        raise ValueError('previous Claude cycle/publication requires reconciliation; no automatic repeat')
    if coord.git(root, 'status', '--porcelain'):
        raise ValueError('preserve and reconcile dirty work before a new Claude cycle')
    if coord.branch(root, args.id) != assignment['branch']:
        raise ValueError('launch in the dedicated assigned branch, not another worker checkout')
    auth, command = funded_call(root, args, assignment, state)
    record_path = root / 'coordination' / 'agents' / (args.id + '.json')
    if not record_path.exists():
        coord.register(root, SimpleNamespace(id=args.id, machine=args.machine, tool='Claude Code Opus',
            capability=['rust', 'linux', 'python', 'headless'], allocation=assignment['allocation_id']))
    coord.acknowledge(root, SimpleNamespace(id=args.id, task=assignment['task_id'], revision=assignment['revision'],
        adaptation='API-only Opus supervisor accepted exact assignment and verified grant before inference.'))
    folder = coord.local_state(root)
    index = state.get('cycles', 0) + 1
    log = folder / ('claude-cycle-%d.json' % index)
    prompt = f'''You are {args.id}, a Claude Opus engineering worker on {args.machine}.
Read AGENTS.md, ORCHESTRATION.md, coordination/COMMANDER.md, coordination/RESOURCES.md,
your assignment and relevant contracts before editing. The user explicitly authorizes
API-only Claude work on this Linux machine. Subscription/extra usage is prohibited.
Do not assume available API credits or change the selected allocation.
Selected API allocation: {json.dumps(auth)}. Do not change it yourself.
Assignment: {json.dumps(assignment)}
Only edit assigned owned paths. Read relevant fetched peer revisions and adapt your work;
report exact reviewed refs=SHAs. Never claim a fetch is a review. Continue from files and
committed handoff. Work up to {max(1,args.cycle_seconds-45)} seconds then return a useful
checkpoint in the requested JSON. The supervisor continues new cycles and new queued tasks;
your response does not end the project. Run meaningful tests and report honest limitations.
Do not stage, commit, push, switch branches, merge, change shared controls or edit your report;
supervisor does that using actual Cursor and coauthors the authenticated GitHub user.
Do not launch other paid CLIs/APIs/subagents, purchase credits, change billing or account settings.
Do not start detached background tasks. Missing permissions/auth/quota are blockers to report,
not prompts to wait on. Local subscription/extra usage is prohibited. Other machines have
separate grants; never use their credentials or silently switch provider or billing source.
Return state, summary, next_action, eta_minutes, tests, reviewed_peers and adaptation.
'''
    state.update(phase='model_running', cycles=index, assignment=key,
                 started_utc=coord.stamp(), log=str(log), authentication=auth,
                 allocation=assignment['allocation_id'], supervisor_pid=os.getpid(),
                 usage={'cycle':index,'status':'in_flight_unknown'})
    durable_state(state_path, state)
    head = coord.git(root, 'rev-parse', 'HEAD')
    branch = coord.branch(root, args.id)
    prepared_report = record_path.read_text()
    code = worker.execute(command, root, prompt, args.cycle_seconds, log)
    if code:
        state.update(phase='model_failed', exit_code=code)
        durable_state(state_path, state)
        raise RuntimeError('Claude cycle failed; preserve changes/log, reconcile quota; no automatic credit fallback')
    result, usage = read_result(log)
    if auth.get('route') == 'verified_prepaid_api':
        usage['pool'] = assignment['allocation_id']
        usage['reserved_call_cap_usd'] = auth['call_cap_usd']
        usage['billing_route'] = 'verified_prepaid_api'
    if head != coord.git(root, 'rev-parse', 'HEAD') or branch != coord.branch(root, args.id):
        raise ValueError('Claude changed HEAD/branch; preserve and inspect')
    if record_path.read_text() != prepared_report:
        raise ValueError('Claude changed supervisor-owned registration/report; preserve and inspect')
    allowed = assignment['owned_paths'] + ['coordination/agents/' + args.id + '.json']
    paths = worker.changed_paths(root)
    if any(not any(p == a or p.startswith(a + '/') for a in allowed) for p in paths):
        raise ValueError('Claude changed unassigned paths; preserve and inspect')
    coord.report(root, SimpleNamespace(id=args.id, state=result['state'], summary=result['summary'],
        next=result['next_action'], eta=result['eta_minutes'], usage='API usage reservations recorded; actual bill and remaining credits require provider reconciliation',
        evidence='; '.join(result['tests']), review=result['reviewed_peers'], adaptation=result['adaptation']))
    report = json.loads(record_path.read_text())
    report['allocation_id'] = assignment['allocation_id']
    report['claude_usage'] = usage
    report['supervisor'] = {'pid':os.getpid(), 'cycle':index, 'assignment':key,
        'last_completed_utc':coord.stamp(), 'state':result['state']}
    coord.write_json(record_path, report)
    coord.git(root, 'add', '--', *worker.changed_paths(root))
    state.update(phase='publication_started', usage=usage, head_before_publish=head)
    durable_state(state_path, state)
    coord.publish(root, SimpleNamespace(allocation=assignment['allocation_id']+'-cursor-checkpoint',
        implementation='Claude Code Opus', message=assignment['task_id']+': '+result['state']+' Opus checkpoint', timeout=180))
    state.update(phase='idle', last_published_sha=coord.git(root,'rev-parse','HEAD'))
    if result['state'] in ('blocked','ready_for_integration'):
        state['completed_assignment'] = key
    if result['state'] == 'blocked':
        state['pending_blocker'] = True
    durable_state(state_path, state)
    if result['state'] == 'blocked':
        report_blocker(root,args,RuntimeError('Worker reported blocked; inspect its published handoff'))
        state.pop('pending_blocker')
        durable_state(state_path,state)
    return result['state']


def report_blocker(root, args, error):
    folder = coord.local_state(root)
    folder.mkdir(parents=True, exist_ok=True)
    marker = folder / 'claude-blocker-reported.json'
    state_file = folder / 'claude-worker.json'
    state = json.loads(state_file.read_text()) if state_file.exists() else {}
    fingerprint = hashlib.sha256(json.dumps([state.get('assignment'), state.get('phase'),
        state.get('cycles'), type(error).__name__, str(error)]).encode()).hexdigest()
    previous = json.loads(marker.read_text()) if marker.exists() else {}
    if previous.get('fingerprint') == fingerprint:
        return
    body = folder / 'claude-blocker.md'
    body.write_text(f'Worker `{args.id}` on `{args.machine}` needs recovery ({type(error).__name__}). '
        'Inspect local claude-worker.json and Git work before another inference call. '
        'No API fallback or purchase was attempted. Logs stay local; ask for sanitized auth/quota evidence. '
        'Orchestrator: assign a resolver and keep other eligible workers active.\n'
        + '\nSanitized execution state:\n```json\n' + json.dumps({
            'assignment':state.get('assignment'), 'phase':state.get('phase'),
            'cycle':state.get('cycles'), 'usage':state.get('usage','unknown'),
            'reserved_api_call_caps_usd':state.get('credit_reservations',{}),
            'unresolved_call':state.get('phase') in ('model_running','model_failed'),
        },indent=2) + '\n```\n')
    result = coord.run(['gh','issue','create','--title','Claude worker recovery: '+args.id,'--body-file',str(body)],cwd=root)
    coord.write_json(marker, {'issue':result.stdout.strip(),'utc':coord.stamp(), 'fingerprint':fingerprint})


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--id',default='local-claude-opus')
    parser.add_argument('--machine',default='linux-primary')
    parser.add_argument('--cycle-seconds',type=int,default=300)
    parser.add_argument('--poll-seconds',type=int,default=30)
    parser.add_argument('--once',action='store_true')
    parser.add_argument('--funding',choices=['api'],default='api')
    parser.add_argument('--api-grant',help='private verified prepaid-credit allocation JSON; no secrets in this file')
    parser.add_argument('--check',action='append',default=[])
    args = parser.parse_args()
    root = None
    try:
        coord.identifier(args.id)
        if args.cycle_seconds < 60 or not 1 <= args.poll_seconds <= 60:
            raise ValueError('cycle must allow >=60 seconds; polling must be 1..60 seconds')
        root = coord.root_dir()
        folder = coord.local_state(root)
        folder.mkdir(parents=True,exist_ok=True)
        with (folder/'claude-worker.lock').open('a') as lock:
            fcntl.flock(lock,fcntl.LOCK_EX|fcntl.LOCK_NB)
            state_path = folder/'claude-worker.json'
            state = json.loads(state_path.read_text()) if state_path.exists() else {'phase':'idle','cycles':0}
            status = coord.run(['cursor-agent','status'],cwd=root)
            if 'Logged in' not in status.stdout + status.stderr:
                raise ValueError('Cursor login required before Claude implementation spending')
            coord.current_git_user_trailer(root)
            while True:
                fleet = coord.checkpoint(root,emit=False)
                control = coord.peer_json(root,'origin/main','coordination/control.json')
                if control.get('state') == 'user_stopped':
                    return 0
                assignment = worker.assignment_for(fleet,args.id)
                result = 'waiting_for_assignment'
                if assignment:
                    pointer = coord.peer_json(root,'origin/main','coordination/next/'+args.id+'.json')
                    if pointer.get('task_id') != assignment['task_id'] or pointer.get('revision') != assignment['revision']:
                        raise ValueError('assignment and next-task pointer disagree')
                    if state.get('phase') == 'idle' and not coord.git(root,'status','--porcelain') and state.get('completed_assignment') != assignment['task_id']+':'+str(assignment['revision']):
                        integrate.sync(root,SimpleNamespace(id=args.id,allocation=assignment['allocation_id']+'-cursor-sync',
                            check=args.check or ['["python3","-m","unittest","discover","-s","tests","-q"]'],
                            timeout=180,check_timeout=300))
                    result = cycle(root,args,assignment,state_path,state)
                print(json.dumps({'worker':args.id,'status':result,'utc':coord.stamp()}),flush=True)
                if args.once:
                    return 0
                time.sleep(args.poll_seconds)
    except (OSError,ValueError,RuntimeError,KeyError,subprocess.TimeoutExpired) as error:
        print('claude-worker: '+str(error),file=sys.stderr)
        if root:
            try:
                report_blocker(root,args,error)
            except Exception as notice_error:
                print('recovery issue unavailable: '+type(notice_error).__name__,file=sys.stderr)
        return 1


if __name__ == '__main__':
    sys.exit(main())
