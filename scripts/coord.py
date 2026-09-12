#!/usr/bin/env python3
"""Git-backed FlashTeX coordination. Python 3.9+, standard library only."""
import argparse
from contextlib import contextmanager
from datetime import datetime, timezone
import fcntl
import json
import os
from pathlib import Path, PurePosixPath
import re
import subprocess
import sys
import time

DEADLINE = '2026-09-12T14:00:00Z'
ID = re.compile(r'^[a-zA-Z0-9][a-zA-Z0-9_-]{0,63}$')
BRANCH = re.compile(r'^agent/([a-zA-Z0-9][a-zA-Z0-9_-]{0,63})/[a-zA-Z0-9][a-zA-Z0-9_/-]*$')
STATES = ['registered', 'accepted', 'in_progress', 'blocked', 'ready_for_integration', 'integrated', 'cancelled']


def run(argv, cwd=None, timeout=45, check=True):
    p = subprocess.run(argv, cwd=cwd, text=True, capture_output=True, timeout=timeout)
    if check and p.returncode:
        raise RuntimeError(p.stderr.strip() or p.stdout.strip() or f'Command failed: {argv[0]}')
    return p


def git(root, *args, check=True):
    return run(['git', *args], cwd=root, check=check).stdout.strip()


def root_dir():
    return Path(git(None, 'rev-parse', '--show-toplevel'))


def stamp():
    return datetime.now(timezone.utc).isoformat(timespec='seconds').replace('+00:00', 'Z')


def parse_time(value):
    result = datetime.fromisoformat(value.replace('Z', '+00:00'))
    if result.tzinfo is None:
        raise ValueError('timestamp must include timezone')
    return result


def identifier(value):
    if not ID.fullmatch(value):
        raise ValueError('invalid identifier: use letters, digits, hyphen, underscore')
    return value


def branch(root, agent=None):
    name = git(root, 'symbolic-ref', '--quiet', '--short', 'HEAD')
    match = BRANCH.fullmatch(name)
    if not match or '..' in name or (agent and match.group(1) != agent):
        raise ValueError('use your own agent/<id>/<task> branch, never main or detached HEAD')
    return name


def local_state(root):
    path = Path(git(root, 'rev-parse', '--git-path', 'flashtex'))
    return path if path.is_absolute() else root / path


def write_json(path, value):
    path.parent.mkdir(parents=True, exist_ok=True)
    temporary = path.with_suffix(path.suffix + '.tmp')
    temporary.write_text(json.dumps(value, indent=2, sort_keys=True) + '\n')
    temporary.replace(path)


@contextmanager
def lock(root):
    folder = local_state(root)
    folder.mkdir(parents=True, exist_ok=True)
    with (folder / 'checkpoint.lock').open('a') as handle:
        fcntl.flock(handle, fcntl.LOCK_EX)
        yield folder


def peer_json(root, ref, path):
    size = int(git(root, 'cat-file', '-s', f'{ref}:{path}'))
    if size > 200_000:
        raise ValueError('coordination record exceeds 200 KB')
    obj = json.loads(git(root, 'show', f'{ref}:{path}'))
    if not isinstance(obj, dict) or obj.get('schema_version') != 1:
        raise ValueError('unsupported coordination schema')
    return obj


def own_record(root, agent):
    identifier(agent)
    name = branch(root, agent)
    path = root / 'coordination' / 'agents' / (agent + '.json')
    record = json.loads(path.read_text())
    if record.get('agent_id') != agent or record.get('schema_version') != 1:
        raise ValueError('invalid local registration')
    record['branch'] = name
    return path, record


def save_report(root, path, record):
    record['updated_utc'] = stamp()
    record['code_revision'] = git(root, 'rev-parse', 'HEAD')
    record['main_integrated_through'] = git(root, 'merge-base', 'HEAD', 'origin/main')
    write_json(path, record)
    print(f'Updated {path.relative_to(root)}; not committed or published yet.')


def register(root, args):
    agent = identifier(args.id)
    name = branch(root, agent)
    path = root / 'coordination' / 'agents' / (agent + '.json')
    if path.exists():
        raise ValueError('registration exists; use report to update it')
    save_report(root, path, {
        'schema_version': 1, 'agent_id': agent, 'machine': args.machine,
        'tool': args.tool, 'capabilities': args.capability, 'branch': name,
        'state': 'registered', 'summary': 'Available for Commander assignment',
        'next_action': 'Await and acknowledge assignment', 'eta_minutes': None,
        'allocation_id': args.allocation, 'usage': {'status': 'unknown'},
        'reviewed_peers': {}, 'assignment_acknowledgements': {},
    })


def report(root, args):
    path, obj = own_record(root, args.id)
    if args.eta is not None and (min(args.eta) < 0 or sorted(args.eta) != args.eta):
        raise ValueError('ETA must be nonnegative optimistic <= likely <= pessimistic')
    obj.update(state=args.state, summary=args.summary, next_action=args.next)
    if args.eta is not None:
        obj['eta_minutes'] = dict(zip(['optimistic', 'likely', 'pessimistic'], args.eta))
    if args.usage is not None:
        obj['usage'] = {'status': args.usage, 'evidence': args.evidence}
    if args.review:
        if not args.adaptation:
            raise ValueError('--review requires --adaptation explaining your actual response')
        for item in args.review:
            ref, sha = item.split('=', 1)
            if ref != 'origin/main' and not (ref.startswith('origin/') and BRANCH.fullmatch(ref.removeprefix('origin/'))):
                raise ValueError('review ref must be origin/main or an origin/agent/<id>/<task> branch')
            if not re.fullmatch(r'[0-9a-f]{40}', sha):
                raise ValueError('review requires a full commit SHA')
            if git(root, 'rev-parse', '--verify', ref + '^{commit}') != sha:
                raise ValueError('reviewed SHA must match the named fetched ref')
            obj['reviewed_peers'][ref] = {'sha': sha, 'adaptation': args.adaptation, 'reviewed_utc': stamp()}
    save_report(root, path, obj)


def owned_path(value):
    p = PurePosixPath(value)
    if p.is_absolute() or not p.parts or '..' in p.parts or p.parts[0] == '.git':
        raise ValueError('owned paths must be repository-relative, outside .git')
    return str(p).rstrip('/')


def overlaps(a, b):
    return a == b or a.startswith(b + '/') or b.startswith(a + '/')


def dispatch(root, args):
    branch(root, 'commander')
    task, agent = identifier(args.task), identifier(args.agent)
    if args.minutes <= 0:
        raise ValueError('timebox must be positive')
    if not args.branch.startswith(f'agent/{agent}/') or not BRANCH.fullmatch(args.branch):
        raise ValueError('assignment branch must belong to the assignee')
    if run(['git', 'merge-base', '--is-ancestor', 'origin/main', 'HEAD'], cwd=root, check=False).returncode:
        raise ValueError('incorporate current origin/main before issuing assignments')
    paths = [owned_path(p) for p in args.path]
    folder = root / 'coordination' / 'assignments'
    for file in folder.glob('*.json'):
        other = json.loads(file.read_text())
        if other.get('task_id') == task:
            continue
        if other.get('state') not in ['cancelled', 'integrated', 'verified']:
            if any(overlaps(a, b) for a in paths for b in other['owned_paths']):
                raise ValueError(f'ownership overlaps active task {other["task_id"]}')
    path = folder / (task + '.json')
    old = json.loads(path.read_text()) if path.exists() else None
    if old and old['agent_id'] != agent and old['state'] not in ['cancelled', 'integrated', 'verified']:
        raise ValueError('cannot reassign active ownership; obtain cancellation acknowledgement first')
    obj = {
        'schema_version': 1, 'task_id': task, 'revision': old['revision'] + 1 if old else 1,
        'agent_id': agent, 'branch': args.branch, 'state': 'assigned',
        'objective': args.objective, 'acceptance': args.acceptance,
        'owned_paths': paths, 'dependencies': args.depends, 'timebox_minutes': args.minutes,
        'allocation_id': args.allocation, 'deadline_utc': None if (root / 'coordination/control.json').exists() else DEADLINE,
        'input_main_sha': git(root, 'rev-parse', 'origin/main'), 'updated_utc': stamp(),
    }
    write_json(path, obj)
    print(f'Dispatch {task} revision {obj["revision"]} prepared; authoritative only after publication to main.')


def acknowledge(root, args):
    if not args.adaptation.strip():
        raise ValueError('acknowledgement requires a nonempty adaptation/acceptance note')
    path, obj = own_record(root, args.id)
    task = identifier(args.task)
    assignment = peer_json(root, 'origin/main', f'coordination/assignments/{task}.json')
    if assignment['agent_id'] != args.id or assignment['branch'] != obj['branch']:
        raise ValueError('assignment owner/branch does not match this worker')
    if assignment['revision'] != args.revision or assignment['state'] != 'assigned':
        raise ValueError('acknowledge the currently published active assignment revision')
    obj['assignment_acknowledgements'][task] = {
        'revision': args.revision, 'main_sha': git(root, 'rev-parse', 'origin/main'),
        'adaptation': args.adaptation, 'acknowledged_utc': stamp(),
    }
    obj['state'] = 'accepted'
    save_report(root, path, obj)


def inspect_fleet(root, previous):
    now = datetime.now(timezone.utc)
    warnings, reports, assignments, tips, changed, legacy = [], [], [], {}, [], []
    refs = git(root, 'for-each-ref', '--format=%(refname:short)', 'refs/remotes/origin/agent/').splitlines()
    refs += ['origin/main']
    for ref in refs:
        sha = git(root, 'rev-parse', ref)
        tips[ref] = sha
        before = previous.get('observed_tips', {}).get(ref)
        if before != sha:
            item = {'ref': ref, 'previous': before, 'current': sha}
            if before:
                item['changed_paths'] = git(root, 'diff', '--name-only', before, sha, check=False).splitlines()
            changed.append(item)
        if ref == 'origin/main':
            continue
        match = BRANCH.fullmatch(ref.removeprefix('origin/'))
        if not match:
            continue
        agent = match.group(1)
        path = f'coordination/agents/{agent}.json'
        if not git(root, 'ls-tree', '--name-only', ref, '--', path):
            old_path = f'coordination/{agent}.md'
            if git(root, 'ls-tree', '--name-only', ref, '--', old_path):
                legacy.append({'agent_id': agent, 'ref': ref, 'sha': sha, 'handoff_path': old_path,
                               'action': 'Read legacy Markdown registration and request structured registration; freshness/ack not inferred.'})
            continue
        try:
            obj = peer_json(root, ref, path)
            if obj.get('agent_id') != agent or obj.get('branch') != ref.removeprefix('origin/'):
                continue  # Ignore records inherited from other task branches.
            if obj.get('state') not in STATES:
                raise ValueError('invalid worker state')
            if not isinstance(obj.get('assignment_acknowledgements'), dict) or not isinstance(obj.get('reviewed_peers'), dict):
                raise ValueError('acknowledgements and reviewed_peers must be objects')
            age = (now - parse_time(obj['updated_utc'])).total_seconds()
            if age < -300:
                raise ValueError('worker timestamp is over five minutes in the future')
            reports.append({'ref': ref, 'sha': sha, 'stale': age > 600, 'age_seconds': max(0, int(age)), 'report': obj})
        except (ValueError, KeyError, TypeError, RuntimeError) as exc:
            warnings.append(f'{ref}: invalid report: {exc}')
    for path in git(root, 'ls-tree', '-r', '--name-only', 'origin/main', '--', 'coordination/assignments').splitlines():
        if not path.endswith('.json'):
            continue
        try:
            obj = peer_json(root, 'origin/main', path)
            identifier(obj['task_id']); identifier(obj['agent_id'])
            if not isinstance(obj.get('revision'), int) or obj['revision'] <= 0:
                raise ValueError('invalid assignment revision')
            if path != f'coordination/assignments/{obj["task_id"]}.json':
                raise ValueError('assignment ID does not match its filename')
            if obj.get('state') not in ['assigned', 'cancel_requested', 'cancelled', 'integrated', 'verified']:
                raise ValueError('invalid assignment state')
            if not isinstance(obj.get('owned_paths'), list) or not obj['owned_paths']:
                raise ValueError('assignment requires owned paths')
            for owned in obj['owned_paths']:
                owned_path(owned)
            matching = [r for r in reports if r['report']['agent_id'] == obj['agent_id'] and r['report']['branch'] == obj['branch']]
            ack = matching[0]['report'].get('assignment_acknowledgements', {}).get(obj['task_id'], {}) if matching else {}
            if not isinstance(ack, dict):
                raise ValueError('invalid worker acknowledgement object')
            acknowledged = ack.get('revision') == obj['revision'] and bool(ack.get('adaptation'))
            if acknowledged:
                ack_sha = ack.get('main_sha', '')
                try:
                    at_ack = peer_json(root, ack_sha, path) if re.fullmatch(r'[0-9a-f]{40}', ack_sha) else {}
                    acknowledged = at_ack == obj
                except (RuntimeError, ValueError):
                    acknowledged = False
            assignments.append({'assignment': obj, 'acknowledged': acknowledged})
        except (ValueError, KeyError, TypeError, RuntimeError) as exc:
            warnings.append(f'{path}: invalid assignment: {exc}')
    for agent in sorted({r['report']['agent_id'] for r in reports}):
        active = [r for r in reports if r['report']['agent_id'] == agent and r['report']['state'] not in ['cancelled', 'integrated']]
        preferred = {a['assignment']['branch'] for a in assignments if a['assignment']['agent_id'] == agent and a['assignment']['state'] == 'assigned'}
        if len(active) > 1 and not (len(preferred) == 1 and any(r['report']['branch'] in preferred for r in active)):
            warnings.append(f'{agent}: multiple active report branches; Commander must resolve ownership')
    for i, left in enumerate(assignments):
        a = left['assignment']
        if a['state'] in ['cancelled', 'integrated', 'verified']:
            continue
        for right in assignments[i+1:]:
            b = right['assignment']
            if b['state'] not in ['cancelled', 'integrated', 'verified']:
                if any(overlaps(x, y) for x in a['owned_paths'] for y in b['owned_paths']):
                    warnings.append(f'ownership overlap: {a["task_id"]} and {b["task_id"]}')
    return {'schema_version': 1, 'checked_utc': stamp(), 'deadline_utc': DEADLINE,
            'remaining_seconds': int((parse_time(DEADLINE)-now).total_seconds()),
            'fetch_ok': True, 'observed_tips': tips, 'changed_refs': changed,
            'reports': reports, 'legacy_handoffs': legacy, 'assignments': assignments, 'warnings': warnings}


def checkpoint(root, emit=True):
    with lock(root) as folder:
        path = folder / 'checkpoint.json'
        previous = json.loads(path.read_text()) if path.exists() else {}
        try:
            git(root, 'fetch', 'origin', '--prune')
        except (RuntimeError, subprocess.TimeoutExpired) as exc:
            previous.update(fetch_ok=False, checked_utc=stamp(), fetch_error=str(exc))
            write_json(path, previous)
            raise RuntimeError('Fetch failed; prior state retained and marked stale. ' + str(exc))
        obj = inspect_fleet(root, previous)
        write_json(path, obj)
    if emit:
        print(json.dumps(obj, indent=2))
    return obj


def watch(root, args):
    if args.interval < 10:
        raise ValueError('watch interval must be at least 10 seconds')
    while not getattr(args, 'until', None) or datetime.now(timezone.utc) < parse_time(args.until):
        try:
            obj = checkpoint(root, emit=False)
            print(json.dumps({'checked_utc': obj['checked_utc'], 'changed_refs': obj['changed_refs'],
                              'reports': len(obj['reports']), 'legacy_handoffs': obj['legacy_handoffs'],
                              'warnings': obj['warnings']}), flush=True)
        except RuntimeError as exc:
            print(str(exc), file=sys.stderr, flush=True)
        remaining = (parse_time(args.until)-datetime.now(timezone.utc)).total_seconds() if getattr(args, 'until', None) else args.interval
        if remaining > 0:
            time.sleep(min(args.interval, remaining))
    print('Deadline reached; watcher stopped. No model, merge, commit, or push was performed.')


def current_git_user_trailer(root):
    """Use the authenticated GitHub identity, not the repository's Cursor bot label."""
    data = json.loads(run(['gh', 'api', 'user', '--jq', '{login: .login, id: .id}'], cwd=root).stdout)
    login, user_id = data.get('login'), data.get('id')
    if not isinstance(login, str) or not re.fullmatch(r'[A-Za-z0-9][A-Za-z0-9-]{0,38}', login):
        raise ValueError('cannot verify current GitHub user for coauthorship')
    if not isinstance(user_id, int) or isinstance(user_id, bool) or user_id <= 0:
        raise ValueError('cannot verify GitHub user ID for noreply coauthor address')
    return f'Co-authored-by: {login} <{user_id}+{login}@users.noreply.github.com>'


def commit_identity(root, coauthor):
    """Honor the published Mac-specific primary-author exception; Cursor executes."""
    match = BRANCH.fullmatch(branch(root))
    record = root / 'coordination' / 'agents' / (match.group(1) + '.json')
    if record.exists():
        obj = json.loads(record.read_text())
        if obj.get('agent_id') == match.group(1) and obj.get('machine') == 'mac-m1max-a':
            return coauthor.removeprefix('Co-authored-by: ')
    return 'Cursor <cursor@flashtex.invalid>'


def publish(root, args):
    name = branch(root)
    if not args.allocation:
        raise ValueError('an authorized Cursor allocation identifier is required')
    if not git(root, 'diff', '--cached', '--name-only'):
        raise ValueError('stage intended changes before publication')
    if git(root, 'diff', '--name-only') or git(root, 'ls-files', '--others', '--exclude-standard'):
        raise ValueError('unstaged/untracked work present; preserve and resolve it before publication')
    git(root, 'diff', '--cached', '--check')
    before = git(root, 'rev-parse', 'HEAD')
    tree = git(root, 'write-tree')
    coauthor = current_git_user_trailer(root)
    direct = bool(getattr(args, 'direct_agent_commit', False))
    expected_identity = commit_identity(root, coauthor)
    if direct and not expected_identity.startswith('jay3332 '):
        if not args.implementation.strip() or any(c in args.implementation for c in '\r\n<>'):
            raise ValueError('direct commit needs a valid actual implementation-agent name')
        expected_identity = args.implementation + ' <agent@flashtex.invalid>'
    prompt = ('Read AGENTS.md. The user requires you, Cursor CLI, to execute this commit. '
              'This is one authorized bounded commit session under allocation ' + args.allocation + '. '
              'No nested model/Claude calls, purchases, branch switching, push, history rewriting, '
              'or file modifications. Review the already staged diff; commit it exactly as staged '
              'with author AND committer ' + expected_identity + '. Execute git commit yourself. '
              'Use this subject: ' + args.message + '\n'
              'Include trailers Implementation-Agent: ' + args.implementation + '\nCommit-Executor: Cursor CLI\n'
              + coauthor + '\n'
              'Verify staged whitespace. Return the SHA. Stop if the staged changes are unsuitable.')
    if direct:
        # Explicit latest user authorization after observed Cursor quota exhaustion.
        # This branch never launches Cursor or misattributes execution to Cursor CLI.
        author_name, author_email = expected_identity.rsplit(' <', 1)
        env = dict(os.environ, GIT_AUTHOR_NAME=author_name, GIT_AUTHOR_EMAIL=author_email[:-1],
                   GIT_COMMITTER_NAME=author_name, GIT_COMMITTER_EMAIL=author_email[:-1])
        message = (args.message + '\n\nImplementation-Agent: ' + args.implementation +
                   '\nCommit-Executor: git via current agent (authorized Cursor-limit fallback)\n' + coauthor + '\n')
        result = subprocess.run(['git', 'commit', '-F', '-'], input=message, cwd=root, env=env,
                                text=True, capture_output=True, timeout=args.timeout)
    else:
        result = run(['cursor-agent', '--print', '--trust', '--auto-review', '--output-format', 'text', prompt],
                     cwd=root, timeout=args.timeout, check=False)
    if result.returncode:
        raise RuntimeError('Cursor failed; inspect Git state before retrying. ' + result.stderr[-1500:])
    after = git(root, 'rev-parse', 'HEAD')
    if after == before or branch(root) != name:
        raise RuntimeError('Cursor did not create the expected commit on this branch; not pushing')
    if git(root, 'show', '-s', '--format=%P', after) != before:
        raise RuntimeError('expected exactly one new, non-merge commit; not pushing')
    if git(root, 'rev-parse', after + '^{tree}') != tree or git(root, 'status', '--porcelain'):
        raise RuntimeError('Cursor changed the staged tree or left work dirty; not pushing')
    ident = git(root, 'show', '-s', '--format=%an <%ae>%n%cn <%ce>', after).splitlines()
    if ident != [expected_identity] * 2:
        raise RuntimeError('unexpected author/committer; not pushing')
    message = git(root, 'show', '-s', '--format=%B', after)
    executor = 'git via current agent (authorized Cursor-limit fallback)' if direct else 'Cursor CLI'
    if 'Commit-Executor: ' + executor not in message.splitlines() or not re.search(r'^Implementation-Agent: .+$', message, re.M):
        raise RuntimeError('missing truthful provenance trailers; not pushing')
    if coauthor not in message.splitlines():
        raise RuntimeError('missing authenticated GitHub user coauthor; not pushing')
    git(root, 'push', '--set-upstream', 'origin', f'HEAD:refs/heads/{name}')
    print(json.dumps({'commit': after, 'branch': name, 'published': True, 'cursor_output': result.stdout[-3000:]}))


def parser():
    p = argparse.ArgumentParser(description=__doc__)
    sub = p.add_subparsers(dest='command', required=True)
    r = sub.add_parser('register'); r.add_argument('--id', required=True); r.add_argument('--machine', required=True)
    r.add_argument('--tool', required=True); r.add_argument('--capability', action='append', default=[])
    r.add_argument('--allocation', default='unallocated')
    r = sub.add_parser('report'); r.add_argument('--id', required=True); r.add_argument('--state', choices=STATES, required=True)
    r.add_argument('--summary', required=True); r.add_argument('--next', required=True)
    r.add_argument('--eta', type=int, nargs=3); r.add_argument('--usage'); r.add_argument('--evidence', default='unknown')
    r.add_argument('--review', action='append', default=[]); r.add_argument('--adaptation')
    r = sub.add_parser('dispatch'); r.add_argument('--task', required=True); r.add_argument('--agent', required=True)
    r.add_argument('--branch', required=True); r.add_argument('--objective', required=True)
    r.add_argument('--acceptance', action='append', required=True); r.add_argument('--path', action='append', required=True)
    r.add_argument('--depends', action='append', default=[]); r.add_argument('--minutes', type=int, required=True)
    r.add_argument('--allocation', required=True)
    r = sub.add_parser('ack'); r.add_argument('--id', required=True); r.add_argument('--task', required=True)
    r.add_argument('--revision', type=int, required=True); r.add_argument('--adaptation', required=True)
    sub.add_parser('checkpoint')
    r = sub.add_parser('watch'); r.add_argument('--interval', type=int, default=60); r.add_argument('--until')
    r = sub.add_parser('publish'); r.add_argument('-m', '--message', required=True)
    r.add_argument('--implementation', required=True); r.add_argument('--allocation', required=True)
    r.add_argument('--timeout', type=int, default=180)
    r.add_argument('--direct-agent-commit', action='store_true', help='explicit user-authorized fallback after Cursor quota exhaustion; truthful direct Git provenance')
    return p


def main():
    args = parser().parse_args()
    try:
        root = root_dir()
        action = {'register': register, 'report': report, 'dispatch': dispatch, 'ack': acknowledge,
                  'watch': watch, 'publish': publish}.get(args.command)
        checkpoint(root) if args.command == 'checkpoint' else action(root, args)
        return 0
    except (RuntimeError, ValueError, KeyError, OSError, subprocess.TimeoutExpired) as exc:
        print(f'coord: {exc}', file=sys.stderr)
        return 1


if __name__ == '__main__':
    sys.exit(main())
