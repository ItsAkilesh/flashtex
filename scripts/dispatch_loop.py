#!/usr/bin/env python3
"""Advance explicit per-worker queues from published completion reports.

No model is called unless --publish and --allocation are supplied. Reported ready
means ready for review, never integrated, verified, or project complete. Every
queue step preserves the worker/task/branch and its existing ownership scope.
"""
import argparse
from contextlib import contextmanager
from datetime import datetime, timezone
import fcntl
import json
from pathlib import Path
import subprocess
import sys
import time
from types import SimpleNamespace

import coord


def fetch_origin(root):
    """Retry only a read-only fetch ref-CAS race with another linked worktree.

    Never retry publication, auth failures, arbitrary locks, or pending paid calls.
    """
    for attempt in range(3):
        try:
            return coord.git(root, 'fetch', 'origin', '--prune')
        except RuntimeError as exc:
            message = str(exc)
            if (attempt == 2 or "cannot lock ref 'refs/remotes/origin/" not in message
                    or ' but expected ' not in message or ' is at ' not in message):
                raise
            time.sleep(0.1 * (attempt + 1))


@contextmanager
def dispatcher_lock(root):
    folder = coord.local_state(root)
    folder.mkdir(parents=True, exist_ok=True)
    with (folder / 'dispatcher.lock').open('a') as handle:
        try:
            fcntl.flock(handle, fcntl.LOCK_EX | fcntl.LOCK_NB)
        except BlockingIOError as exc:
            raise RuntimeError('another dispatcher owns this worktree') from exc
        yield folder


def main_records(root, folder):
    paths = coord.git(root, 'ls-tree', '-r', '--name-only', 'origin/main', '--', folder).splitlines()
    return {p: coord.peer_json(root, 'origin/main', p) for p in paths if p.endswith('.json')}


class DispatchAuthorizationError(ValueError):
    """Ownership or funding failure must stop before any mutation."""


def list_of_text(value, field):
    if not isinstance(value, list) or not value or any(not isinstance(s, str) or not s.strip() for s in value):
        raise ValueError(field + ' must be a nonempty list of strings')
    return value


def plan_step(root, queue_path, queue, assignments, now, stale_seconds):
    agent = coord.identifier(queue['agent_id'])
    if Path(queue_path).stem != agent:
        raise ValueError('queue filename must match agent_id')
    index = queue.get('next_index', 0)
    steps = queue.get('steps')
    if not isinstance(index, int) or isinstance(index, bool) or index < 0 or not isinstance(steps, list):
        raise ValueError('queue requires steps list and nonnegative next_index')
    if index >= len(steps):
        return None, 'queue exhausted; Commander must add explicit work'
    candidates = [a for a in assignments.values() if a.get('agent_id') == agent and a.get('state') == 'assigned'
                  and (not queue.get('task_id') or queue['task_id'] == a.get('task_id'))]
    if len(candidates) != 1:
        return None, 'requires exactly one matching active assignment'
    current = candidates[0]
    task = coord.identifier(current['task_id'])
    branch = current['branch']
    if not coord.BRANCH.fullmatch(branch) or (not branch.startswith(f'agent/{agent}/')
            and not coord.published_branch_assignment(root, agent, branch)):
        raise ValueError('assignment branch must match its owner')
    revision = current['revision']
    if not isinstance(revision, int) or isinstance(revision, bool) or revision < 1:
        raise ValueError('invalid assignment revision')
    ref = 'origin/' + branch
    ref_result = coord.run(['git', 'rev-parse', '--verify', ref + '^{commit}'], cwd=root, check=False)
    if ref_result.returncode:
        return None, 'worker branch not published'
    sha = ref_result.stdout.strip()
    report_path = f'coordination/agents/{agent}.json'
    if coord.run(['git', 'cat-file', '-e', sha + ':' + report_path], cwd=root, check=False).returncode:
        return None, 'worker structured report not published'
    report = coord.peer_json(root, sha, report_path)
    if report.get('agent_id') != agent or report.get('branch') != branch:
        return None, 'report belongs to another worker or branch'
    if report.get('state') != 'ready_for_integration':
        return None, 'worker state: ' + str(report.get('state', 'unknown'))
    age = (now - coord.parse_time(report['updated_utc'])).total_seconds()
    if age < -60:
        return None, 'completion report future-dated'
    ack = report.get('assignment_acknowledgements', {}).get(task, {})
    if ack.get('revision') != revision or not isinstance(ack.get('adaptation'), str) or not ack['adaptation'].strip():
        return None, 'assignment revision not acknowledged'
    ack_sha = ack.get('main_sha', '')
    if not isinstance(ack_sha, str) or not coord.re.fullmatch(r'[0-9a-f]{40}', ack_sha):
        return None, 'acknowledgement main SHA invalid'
    if coord.run(['git', 'merge-base', '--is-ancestor', ack_sha, 'origin/main'], cwd=root, check=False).returncode:
        return None, 'acknowledgement is not from authoritative main history'
    try:
        ack_assignment = coord.peer_json(root, ack_sha, f'coordination/assignments/{task}.json')
    except (RuntimeError, ValueError):
        return None, 'acknowledgement main SHA lacks this assignment'
    if ack_assignment != current:
        return None, 'acknowledgement does not identify the current assignment'
    supervisor = report.get('supervisor')
    if supervisor is not None and (not isinstance(supervisor, dict) or supervisor.get('assignment') != f'{task}:{revision}'):
        return None, 'supervisor result does not identify this assignment revision'
    archive_path = f'coordination/completions/{agent}/{task}-r{revision}.json'
    if (root / archive_path).exists():
        return None, 'completion already consumed'
    step = steps[index]
    if not isinstance(step, dict) or not isinstance(step.get('objective'), str) or not step['objective'].strip():
        raise ValueError('queue step requires objective')
    acceptance = list_of_text(step.get('acceptance'), 'acceptance')
    paths = [coord.owned_path(p) for p in list_of_text(step.get('owned_paths'), 'owned_paths')]
    # Revisions may narrow ownership; acquiring new paths requires Commander review.
    if any(not any(p == old or p.startswith(old + '/') for old in current['owned_paths']) for p in paths):
        raise DispatchAuthorizationError('queue cannot expand or transfer path ownership')
    minutes = step.get('timebox_minutes')
    if not isinstance(minutes, int) or isinstance(minutes, bool) or minutes <= 0:
        raise ValueError('queue step requires positive integer timebox_minutes')
    allocation = step.get('allocation_id')
    if not isinstance(allocation, str) or not allocation.strip() or allocation == 'unallocated':
        raise DispatchAuthorizationError('queue step requires explicit authorized allocation_id')
    dependencies = step.get('dependencies', [])
    if not isinstance(dependencies, list) or any(not isinstance(dep, str) for dep in dependencies):
        raise ValueError('dependencies must be task ID strings')
    by_task = {a['task_id']: a for a in assignments.values()}
    for dependency in dependencies:
        coord.identifier(dependency)
        if by_task.get(dependency, {}).get('state') not in ['integrated', 'verified']:
            return None, f'dependency {dependency} is not integrated/verified'
    dispatch_args = SimpleNamespace(task=task, agent=agent, branch=branch, objective=step['objective'],
                                    acceptance=acceptance, path=paths, depends=dependencies,
                                    minutes=minutes, allocation=allocation)
    archive = {'schema_version': 1, 'agent_id': agent, 'task_id': task, 'revision': revision,
               'state': 'reported_ready', 'worker_branch': branch, 'worker_report_sha': sha,
               'report_path': report_path, 'report': report, 'assignment': current,
               'archived_utc': coord.stamp(), 'review_required': True,
               'stale_completion': age > stale_seconds, 'report_age_seconds': max(0, int(age))}
    next_queue = dict(queue, next_index=index + 1)
    pointer = {'schema_version': 1, 'agent_id': agent, 'task_id': task, 'revision': revision + 1,
               'branch': branch, 'assignment_path': f'coordination/assignments/{task}.json',
               'previous_report_sha': sha, 'updated_utc': coord.stamp()}
    return {'args': dispatch_args, 'queue_path': queue_path, 'queue': next_queue,
            'archive_path': archive_path, 'archive': archive,
            'pointer_path': f'coordination/next/{agent}.json', 'pointer': pointer}, None


def require_authority(root, args, ref='origin/main'):
    expected = getattr(args, 'commander_id', None)
    if not expected:
        raise ValueError('explicit --commander-id required; never infer authority from a process name')
    authority = coord.peer_json(root, ref, 'coordination/authority.json')
    if authority.get('authority_state') != 'active' or authority.get('commander_id') != expected:
        raise ValueError('Commander authority changed; dispatcher must stop without mutation')


def scan_once(root, args):
    """Prepare all eligible next revisions, optionally publish once. No retries."""
    root = Path(root)
    with dispatcher_lock(root) as folder:
        journal_path = folder / 'dispatcher-publication.json'
        if journal_path.exists() and json.loads(journal_path.read_text()).get('state') == 'pending':
            raise RuntimeError('previous publication unresolved; inspect Git and publication journal before recovery')
        coord.branch(root, 'commander')
        if coord.git(root, 'status', '--porcelain'):
            raise RuntimeError('Commander worktree is dirty; preserving work and stopping')
        fetch_origin(root)
        baseline = coord.git(root, 'rev-parse', 'origin/main')
        require_authority(root, args, baseline)
        if coord.git(root, 'rev-parse', 'HEAD') != baseline:
            if coord.run(['git', 'merge-base', '--is-ancestor', 'HEAD', baseline], cwd=root, check=False).returncode:
                raise RuntimeError('Commander HEAD must exactly match current origin/main or be a clean ancestor before dispatch')
            coord.git(root, 'merge', '--ff-only', 'origin/main')
        control = {}
        control_path = 'coordination/control.json'
        if not coord.run(['git', 'cat-file', '-e', baseline + ':' + control_path], cwd=root, check=False).returncode:
            control = coord.peer_json(root, baseline, control_path)
            if control.get('state') == 'user_stopped':
                return {'prepared': [], 'skipped': [], 'published': False, 'paths': [],
                        'baseline_main': baseline, 'stopped': True, 'reason': control['state']}
        assignments = main_records(root, 'coordination/assignments')
        queues = main_records(root, 'coordination/queues')
        plans, skipped = [], []
        now = datetime.now(timezone.utc)
        for path, queue in sorted(queues.items()):
            if queue.get('agent_id') in control.get('paused_agents', []):
                skipped.append({'queue': path, 'reason': 'worker paused by explicit user staffing limit'})
                continue
            try:
                plan, reason = plan_step(root, path, queue, assignments, now, args.stale_seconds)
            except DispatchAuthorizationError:
                raise
            except (ValueError, KeyError, TypeError) as exc:
                skipped.append({'queue': path, 'reason': 'invalid queue or worker record: ' + str(exc), 'needs_commander_review': True})
                continue
            if plan:
                plans.append(plan)
            else:
                skipped.append({'queue': path, 'reason': reason})
        # Validate inter-task ownership before any mutation. coord.dispatch checks it again.
        for plan in plans:
            task_args = plan['args']
            for other in assignments.values():
                if other['task_id'] != task_args.task and other.get('state') not in ['cancelled', 'integrated', 'verified']:
                    if any(coord.overlaps(a, b) for a in task_args.path for b in other['owned_paths']):
                        raise ValueError('next task overlaps active ownership: ' + other['task_id'])
        paths = []
        for plan in plans:
            coord.dispatch(root, plan['args'])
            for path_key, value_key in [('queue_path', 'queue'), ('archive_path', 'archive'), ('pointer_path', 'pointer')]:
                coord.write_json(root / plan[path_key], plan[value_key])
                paths.append(plan[path_key])
            paths.append(f'coordination/assignments/{plan["args"].task}.json')
        result = {'prepared': [p['args'].task for p in plans], 'skipped': skipped,
                  'published': False, 'baseline_main': baseline, 'paths': paths}
        if not paths or not args.publish:
            if paths:
                result['action'] = 'Review and publish prepared files; they are not yet authoritative. Watch stops here.'
            return result
        if not args.allocation:
            raise ValueError('--publish requires a separately authorized Cursor --allocation')
        coord.git(root, 'add', '--', *paths)
        # Persist BEFORE paid invocation. A crash or ambiguous failure cannot trigger paid retry.
        coord.write_json(journal_path, dict(state='pending', baseline_main=baseline,
                                           paths=paths, started_utc=coord.stamp()))
        require_authority(root, args, baseline)
        coord.publish(root, SimpleNamespace(allocation=args.allocation, implementation='Codex Astra dispatcher',
                      direct_agent_commit=getattr(args, 'direct_agent_commit', False),
                      message='coord: dispatch next queued worker assignments', timeout=args.timeout))
        fetch_origin(root)
        if coord.git(root, 'rev-parse', 'origin/main') != baseline:
            raise RuntimeError('main changed during publication; task branch preserved, integration required')
        if coord.run(['git', 'merge-base', '--is-ancestor', baseline, 'HEAD'], cwd=root, check=False).returncode:
            raise RuntimeError('published revision is not a descendant of baseline main')
        require_authority(root, args)
        coord.git(root, 'push', 'origin', 'HEAD:refs/heads/main')
        after = coord.git(root, 'rev-parse', 'HEAD')
        coord.write_json(journal_path, dict(state='published', baseline_main=baseline,
                                           commit=after, paths=paths, published_utc=coord.stamp()))
        result.update(published=True, commit=after)
        return result


def parser():
    p = argparse.ArgumentParser(description=__doc__)
    mode = p.add_mutually_exclusive_group()
    mode.add_argument('--once', action='store_true', help='one scan (default)')
    mode.add_argument('--watch', action='store_true', help='scan until interrupted; no project deadline cutoff')
    p.add_argument('--interval', type=int, default=30)
    p.add_argument('--stale-seconds', type=int, default=600)
    p.add_argument('--publish', action='store_true', help='actual Cursor commit, then guarded main push')
    p.add_argument('--commander-id', required=True, help='exact active authority owner; stale owner fails closed')
    p.add_argument('--direct-agent-commit', action='store_true', help='authorized Cursor-limit fallback; no Cursor call')
    p.add_argument('--allocation', help='authorized Cursor publication allocation, never an invented balance')
    p.add_argument('--timeout', type=int, default=180)
    return p


def main():
    args = parser().parse_args()
    try:
        if args.interval < 10 or args.stale_seconds <= 0 or args.timeout <= 0:
            raise ValueError('interval >=10, stale-seconds >0, timeout >0 required')
        if args.publish and not args.allocation:
            raise ValueError('--publish requires authorized Cursor --allocation')
        root = coord.root_dir()
        while True:
            result = scan_once(root, args)
            print(json.dumps(result, indent=2), flush=True)
            if result.get('stopped') or not args.watch or (result['prepared'] and not result['published']):
                return 0
            time.sleep(args.interval)
    except (RuntimeError, ValueError, KeyError, TypeError, OSError, subprocess.TimeoutExpired) as exc:
        print('dispatch_loop: ' + str(exc), file=sys.stderr)
        return 1


if __name__ == '__main__':
    sys.exit(main())
