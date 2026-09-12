#!/usr/bin/env python3
"""Read-only fleet observations. Never a launcher, dispatcher, or authority claim."""
import argparse
from datetime import datetime, timezone
import json
from pathlib import Path
import re
import subprocess


SERVICES = ('flashtex-dispatch.service', 'flashtex-coordination-watch.service')
IDENTIFIER = re.compile(r'[A-Za-z0-9][A-Za-z0-9_.-]*\Z')
BRANCH = re.compile(r'agent/[A-Za-z0-9_.-]+/[A-Za-z0-9_./-]+\Z')
SHA = re.compile(r'[0-9a-f]{40,64}\Z')
COMPLETED = ('ready_for_integration', 'completed', 'complete', 'integrated', 'verified')


def utc(value):
    parsed = datetime.fromisoformat(value.replace('Z', '+00:00'))
    if parsed.tzinfo is None:
        raise ValueError('timestamp needs a timezone')
    return parsed.astimezone(timezone.utc)


def freshness(value, now, limit):
    try:
        age = (now - utc(value)).total_seconds()
    except (ValueError, TypeError, AttributeError):
        return 'unknown'
    return 'future' if age < -60 else 'stale' if age > limit else 'fresh'


def queue_remaining(queue):
    index, steps = queue.get('next_index'), queue.get('steps')
    if type(index) is not int or not isinstance(steps, list) or not 0 <= index <= len(steps):
        return None
    if any(not isinstance(step, dict) or not isinstance(step.get('objective'), str)
           or not step['objective'].strip() for step in steps[index:]):
        return None
    return len(steps) - index


def command(argv, *, cwd, timeout=30):
    return subprocess.run(argv, cwd=cwd, capture_output=True, text=True,
                          timeout=timeout, check=False)


def collect(root, *, runner=command, fetch=True, ref='origin/main', gh_output=None,
            dispatcher=None):
    """Pin main and each assigned worker tip; runner and gh JSON are injectable.

    Fetch updates remote-tracking metadata only. No checkout, writes to tracked
    files, model execution, issue changes, or remote process checks occur.
    """
    result = {'schema_version': 1, 'assignments': {}, 'reports': {}, 'next': {},
              'queues': {}, 'errors': [], 'services': {}, 'report_sources': {},
              'completions': {}, 'completion_sources': {}, 'ack_sources': {},
              'dispatcher': dispatcher, 'fetch': 'skipped', 'authority_ref': ref}

    def run(argv):
        try:
            response = runner(argv, cwd=root, timeout=30)
            if response.returncode:
                result['errors'].append({'operation': argv, 'reason': 'exit ' + str(response.returncode)})
                return None
            return response.stdout
        except (OSError, subprocess.TimeoutExpired) as exc:
            result['errors'].append({'operation': argv, 'reason': type(exc).__name__})
            return None

    def decode(raw, label):
        try:
            value = json.loads(raw)
            if not isinstance(value, dict):
                raise ValueError('expected object')
            return value
        except (ValueError, TypeError):
            result['errors'].append({'operation': label, 'reason': 'invalid JSON object'})
            return {}

    if fetch:
        result['fetch'] = 'ok' if run(['git', 'fetch', 'origin', '--prune']) is not None else 'failed'
    pinned = run(['git', 'rev-parse', '--verify', '--end-of-options', ref + '^{commit}'])
    if pinned is not None:
        pinned = pinned.strip()
        if not re.fullmatch('[0-9a-f]{40,64}', pinned):
            result['errors'].append({'operation': 'pin main', 'reason': 'invalid commit SHA'})
            pinned = None
    result['main_sha'] = pinned
    if pinned:
        paths = run(['git', 'ls-tree', '-r', '--name-only', pinned, '--', 'coordination'])
        for path in sorted((paths or '').splitlines()):
            parts = path.split('/')
            if len(parts) == 3 and parts[1] in ('assignments', 'agents', 'next', 'queues') and path.endswith('.json'):
                value = decode(run(['git', 'show', pinned + ':' + path]), path)
                bucket = 'reports' if parts[1] == 'agents' else parts[1]
                result[bucket][Path(path).stem] = value
            elif path in ('coordination/control.json', 'coordination/authority.json'):
                result[Path(path).stem] = decode(run(['git', 'show', pinned + ':' + path]), path)
            elif len(parts) == 4 and parts[1] == 'completions' and path.endswith('.json'):
                result['completions'][path] = decode(run(['git', 'show', pinned + ':' + path]), path)
        for assignment in result['assignments'].values():
            agent, branch = assignment.get('agent_id'), assignment.get('branch')
            if not isinstance(agent, str) or not IDENTIFIER.fullmatch(agent) or not isinstance(branch, str) or not BRANCH.fullmatch(branch) or not branch.startswith('agent/' + agent + '/'):
                result['errors'].append({'operation': 'worker report', 'reason': 'invalid agent or branch'})
                continue
            tip = run(['git', 'rev-parse', '--verify', '--end-of-options', 'origin/' + branch + '^{commit}'])
            if tip and re.fullmatch('[0-9a-f]{40,64}', tip.strip()):
                tip = tip.strip()
                raw = run(['git', 'show', tip + ':coordination/agents/' + agent + '.json'])
                if raw is not None:
                    result['reports'][agent] = decode(raw, branch)
                    result['report_sources'][agent] = {'branch': branch, 'sha': tip}
        # Historical publication and ACK links are inspected from Git, not inferred
        # from a comment, a PID, a report's code_revision, or an archive's claim.
        for path, archive in result['completions'].items():
            agent, sha = archive.get('agent_id'), archive.get('worker_report_sha')
            if isinstance(agent, str) and IDENTIFIER.fullmatch(agent) and isinstance(sha, str) and SHA.fullmatch(sha):
                raw = run(['git', 'show', sha + ':coordination/agents/' + agent + '.json'])
                if raw is not None:
                    result['completion_sources'][path] = {'sha': sha, 'report': decode(raw, path)}
        for task, assignment in result['assignments'].items():
            agent = assignment.get('agent_id')
            if not isinstance(agent, str) or not IDENTIFIER.fullmatch(task):
                continue
            report = result['reports'].get(agent, {})
            acks = report.get('assignment_acknowledgements', {})
            ack = acks.get(task, {}) if isinstance(acks, dict) else {}
            sha = ack.get('main_sha') if isinstance(ack, dict) else None
            if isinstance(sha, str) and SHA.fullmatch(sha):
                common = run(['git', 'merge-base', sha, pinned])
                raw = run(['git', 'show', sha + ':coordination/assignments/' + task + '.json'])
                result['ack_sources'][agent + '/' + task] = {
                    'main_sha': sha, 'ancestor_of_main': common is not None and common.strip() == sha,
                    'assignment': decode(raw, 'ACK ' + task) if raw is not None else None}

    raw_issues = gh_output if gh_output is not None else run(
        ['gh', 'issue', 'list', '--state', 'open', '--limit', '1000',
         '--json', 'number,title,url,state,updatedAt,labels,body'])
    try:
        issues = json.loads(raw_issues)
        if not isinstance(issues, list) or any(not isinstance(i, dict) for i in issues):
            raise ValueError('expected issue objects')
        result['issues'] = issues
        if len(issues) >= 1000:
            result['errors'].append({'operation': 'issues', 'reason': 'listing may be truncated at 1000'})
    except (ValueError, TypeError):
        result['issues'] = []
        result['errors'].append({'operation': 'issues', 'reason': 'issue snapshot unavailable or invalid'})
    for service in SERVICES:
        raw = run(['systemctl', '--user', 'show', service, '--no-pager',
                   '--property=LoadState,ActiveState,SubState,MainPID'])
        result['services'][service] = dict(line.split('=', 1) for line in (raw or '').splitlines() if '=' in line)
    return result


def analyze(snapshot, *, now, stale_seconds=600, resource_seconds=1800):
    """Pure deterministic analysis: every remote process remains unverified."""
    findings, workers = [], []

    def flag(code, subject, detail):
        findings.append({'code': code, 'subject': subject, 'detail': detail})

    def objects(name):
        value = snapshot.get(name, {})
        if not isinstance(value, dict):
            flag('invalid_snapshot', name, 'Expected a mapping of JSON objects.')
            return {}
        valid = {}
        for key, item in value.items():
            if not isinstance(item, dict):
                flag('invalid_record', str(key), name + ' entry is not an object.')
            else:
                valid[key] = item
        return valid

    reports, assignments = objects('reports'), objects('assignments')
    pointers, queues = objects('next'), objects('queues')
    completions, completion_sources = objects('completions'), objects('completion_sources')
    ack_sources = objects('ack_sources')
    report_sources = objects('report_sources')
    if not assignments:
        flag('assignment_inventory_empty', 'fleet', 'No assignments observed; absence does not establish healthy or completed work.')
    if not reports:
        flag('report_inventory_empty', 'fleet', 'No structured worker/resource reports observed.')
    for error in snapshot.get('errors', []):
        flag('collection_error', 'collection', json.dumps(error, sort_keys=True))
    if snapshot.get('fetch') != 'ok':
        flag('remote_freshness_unverified', 'git', 'Fetch failed or was skipped; pinned observations may be stale.')
    if snapshot.get('authority_ref') != 'origin/main':
        flag('authority_ref_unverified', 'git', 'Snapshot is not identified as origin/main; a task-branch record cannot prove publication on main.')
    active = {}
    for task, assignment in sorted(assignments.items()):
        if assignment.get('state') not in ('assigned', 'accepted', 'in_progress', 'blocked', 'ready_for_integration'):
            continue
        agent = assignment.get('agent_id')
        revision = assignment.get('revision')
        if not isinstance(agent, str) or not IDENTIFIER.fullmatch(agent) or type(revision) is not int or revision < 1 or assignment.get('task_id') != task:
            flag('invalid_assignment', task, 'Matching task ID, valid agent ID and positive integer revision required.')
            continue
        active.setdefault(agent, []).append(task)
        report = reports.get(agent, {})
        acknowledgements = report.get('assignment_acknowledgements', {})
        ack = acknowledgements.get(task, {}) if isinstance(acknowledgements, dict) else {}
        ack = ack if isinstance(ack, dict) else {}
        acknowledged = (type(ack.get('revision')) is int and ack['revision'] == revision
                        and isinstance(ack.get('adaptation'), str) and bool(ack['adaptation'].strip()))
        ack_source = ack_sources.get(agent + '/' + task, {})
        ack_verified = (acknowledged and isinstance(ack.get('main_sha'), str)
                        and bool(SHA.fullmatch(ack['main_sha']))
                        and ack_source.get('main_sha') == ack['main_sha']
                        and ack_source.get('ancestor_of_main') is True
                        and ack_source.get('assignment') == assignment)
        if not acknowledged:
            flag('ack_required', agent, task + ' revision ' + str(revision) + ' needs an exact ACK with adaptation.')
        elif not ack_verified:
            flag('ack_history_unverified', agent, 'Exact revision is reported, but ACK main history and assignment contents do not verify it.')
        if report.get('agent_id') != agent or report.get('branch') != assignment.get('branch'):
            flag('report_identity_mismatch', agent, 'Report missing or does not match assigned agent and branch.')
        age = freshness(report.get('updated_utc'), now, stale_seconds)
        if age != 'fresh':
            flag('report_' + age, agent, 'Report timestamp is ' + age + '; this does not prove a stopped process.')
        pointer = pointers.get(agent)
        pointer_matches = pointer is not None and all(pointer.get(key) == assignment.get(key) for key in ('task_id', 'revision', 'branch', 'agent_id'))
        if pointer is None:
            flag('next_missing', agent, 'No explicit next-task pointer.')
        elif any(pointer.get(key) != assignment.get(key) for key in ('task_id', 'revision', 'branch', 'agent_id')):
            flag('next_mismatch', agent, 'Next pointer differs from the active assignment.')
        if report.get('state') == 'blocked':
            flag('worker_blocked', agent, 'Worker reports a blocker; inspect recovery evidence.')
        state = report.get('state', 'unknown')
        if state in COMPLETED and acknowledged:
            flag('pending_completion_dispatch', agent, task + ' current revision is reported complete; publication/review and next dispatch need evidence, not a running-state assumption.')
        if not acknowledged and revision > 1:
            flag('next_ack_pending', agent, task + ' published revision ' + str(revision) + ' has not been acknowledged by the worker.')
        queue = queues.get(agent, {})
        depth = queue_remaining(queue) if queue.get('agent_id') == agent else None
        sequence = {
            'previous_completion': {'status': 'first_assignment' if revision == 1 else 'missing', 'revision': revision - 1 if revision > 1 else None},
            'publication': {'status': 'first_assignment' if revision == 1 else 'missing_or_mismatched', 'main_sha': snapshot.get('main_sha')},
            'next_ack': {'status': 'verified_history' if ack_verified else 'reported_only' if acknowledged else 'missing_or_mismatched', 'main_sha': ack.get('main_sha')},
            'current_completion': {'reported': state in COMPLETED, 'matches_current_revision': acknowledged,
                                   'report_sha': report_sources.get(agent, {}).get('sha'),
                                   'updated_utc': report.get('updated_utc')}}
        if revision > 1:
            path = 'coordination/completions/' + agent + '/' + task + '-r' + str(revision - 1) + '.json'
            archive = completions.get(path, {})
            source = completion_sources.get(path, {})
            old_assignment, old_report = archive.get('assignment'), archive.get('report')
            old_assignment = old_assignment if isinstance(old_assignment, dict) else {}
            old_report = old_report if isinstance(old_report, dict) else {}
            old_acks = old_report.get('assignment_acknowledgements', {})
            old_ack = old_acks.get(task, {}) if isinstance(old_acks, dict) else {}
            old_ack = old_ack if isinstance(old_ack, dict) else {}
            source_sha = archive.get('worker_report_sha')
            valid = (archive.get('agent_id') == agent and archive.get('task_id') == task
                     and type(archive.get('revision')) is int and archive['revision'] == revision - 1
                     and archive.get('state') == 'reported_ready'
                     and archive.get('worker_branch') == assignment.get('branch')
                     and all(old_assignment.get(k) == v for k, v in {'agent_id': agent, 'task_id': task, 'revision': revision - 1, 'branch': assignment.get('branch')}.items())
                     and old_report.get('agent_id') == agent and old_report.get('branch') == assignment.get('branch')
                     and old_report.get('state') in COMPLETED and type(old_ack.get('revision')) is int
                     and old_ack['revision'] == revision - 1 and isinstance(old_ack.get('adaptation'), str)
                     and bool(old_ack['adaptation'].strip())
                     and isinstance(source_sha, str) and bool(SHA.fullmatch(source_sha))
                     and source.get('sha') == source_sha and source.get('report') == old_report)
            sequence['previous_completion'].update(status='recorded_valid' if valid else 'invalid' if archive else 'missing', report_sha=source_sha)
            linked = (valid and pointer_matches and pointer.get('previous_report_sha') == source_sha
                      and snapshot.get('authority_ref') == 'origin/main'
                      and isinstance(snapshot.get('main_sha'), str) and bool(SHA.fullmatch(snapshot['main_sha'])))
            if linked:
                sequence['publication']['status'] = 'linked_on_main'
            else:
                flag('completion_publication_link_unverified', agent, 'Previous completion source, archived report, next pointer and pinned main do not form a verified link.')
        supervisor = report.get('supervisor', {})
        supervisor = supervisor if isinstance(supervisor, dict) else {}
        usage = report.get('claude_usage', {})
        usage = usage if isinstance(usage, dict) else {}
        history = {'reported': bool(supervisor), 'assignment': supervisor.get('assignment'),
                   'pid': supervisor.get('pid'), 'cycle': supervisor.get('cycle'),
                   'last_completed_utc': supervisor.get('last_completed_utc'),
                   'state': supervisor.get('state'), 'remote_process_verified': False,
                   'quota_availability': 'unknown', 'usage_reported_utc': usage.get('reported_utc'),
                   'usage_freshness': freshness(usage.get('reported_utc'), now, resource_seconds),
                   'reported_remaining_quota': usage.get('remaining_quota', 'unknown')}
        if supervisor and supervisor.get('assignment') != task + ':' + str(revision):
            flag('supervisor_assignment_mismatch', agent, 'Historical supervisor cycle belongs to a different task/revision.')
        if supervisor.get('state') in ('blocked', 'model_failed', 'model_running', 'pending'):
            flag('supervisor_needs_review', agent, 'Historical supervisor state needs blocker/in-flight reconciliation; no process liveness is inferred.')
        activity = ('awaiting_next_ack' if not acknowledged else 'completion_pending_dispatch' if state in COMPLETED
                    else 'blocked' if state == 'blocked' else 'reported_in_progress' if state == 'in_progress' else 'unverified')
        workers.append({'agent_id': agent, 'task_id': task, 'revision': revision,
                        'acknowledged': acknowledged, 'acknowledgement_verified': ack_verified,
                        'report_freshness': age, 'reported_state': state, 'activity': activity,
                        'queue_depth': {'current_assignments': 1, 'remaining_followups': depth,
                                        'target_followups': 2, 'coverage_satisfied': depth is not None and depth >= 2},
                        'handoff_sequence': sequence, 'supervisor_history': history,
                        'remote_process_verified': False})
    for agent, tasks in sorted(active.items()):
        if len(tasks) > 1:
            flag('multiple_active_assignments', agent, ', '.join(sorted(tasks)))
    for agent, report in sorted(reports.items()):
        process = report.get('process_evidence', {})
        process = process if isinstance(process, dict) else {}
        pid = process.get('pid')
        session = process.get('session')
        identity = (type(pid) is int and pid > 0) or (isinstance(session, str) and bool(session.strip()))
        process_age = freshness(process.get('observed_utc'), now, stale_seconds)
        evidence = (identity and process.get('machine') == report.get('machine')
                    and bool(report.get('machine')) and process_age == 'fresh'
                    and process.get('state') == 'running')
        if not evidence:
            flag('activity_unproven', agent, 'Requires fresh machine-bound PID/session evidence and exact ACK; comments, summaries and task rows are insufficient.')
        else:
            flag('remote_activity_reported_only', agent, 'Fresh PID/session reported; this checker has not verified that remote process.')
        resource = report.get('resource_evidence', {})
        resource = resource if isinstance(resource, dict) else {}
        resource_age = freshness(resource.get('observed_utc'), now, resource_seconds)
        if resource_age != 'fresh':
            flag('resource_' + resource_age, agent, 'Provider/resource timestamp is ' + resource_age + '; report heartbeat is not quota evidence.')
        if resource.get('status') not in ('available', 'exhausted', 'blocked'):
            flag('resource_unknown', agent, 'Resource availability is unknown; subscription price is not remaining quota.')
        elif resource['status'] != 'available':
            flag('resource_' + resource['status'], agent, 'Reported resources do not establish eligibility for further work.')
        if process.get('in_flight') not in (None, False, 'none'):
            flag('in_flight_unresolved', agent, 'Reported in-flight work requires reconciliation before retry or reassignment.')
    for agent, queue in sorted(queues.items()):
        depth = queue_remaining(queue)
        if depth is None:
            flag('queue_invalid', agent, 'Queue needs an index from zero through its length and nonempty objectives on remaining steps.')
        else:
            if depth == 0:
                flag('queue_exhausted', agent, 'No queued next step; this is not project completion or proof of worker inactivity.')
            if depth < 2:
                flag('queue_low_water', agent, str(depth) + ' follow-ups remain; the target is current assignment plus two follow-ups.')
        if queue.get('agent_id') != agent:
            flag('queue_identity_mismatch', agent, 'Queue filename and agent ID differ.')
    for agent in sorted(set(active) - set(queues)):
        flag('queue_missing', agent, 'No explicit follow-on queue exists.')
    for agent in sorted(set(pointers) - set(active)):
        flag('next_orphaned', agent, 'Pointer has no matching active assignment.')
    issues = snapshot.get('issues', [])
    if not isinstance(issues, list):
        flag('invalid_snapshot', 'issues', 'Expected a list.')
        issues = []
    for issue in issues:
        if not isinstance(issue, dict):
            flag('invalid_record', 'issue', 'Expected an issue object.')
            continue
        labels = issue.get('labels', [])
        labels = labels if isinstance(labels, list) else []
        names = [label.get('name', '') if isinstance(label, dict) else str(label) for label in labels]
        title = str(issue.get('title', ''))
        if str(issue.get('state', 'OPEN')).upper() == 'OPEN' and ('recovery' in title.lower() or any('recovery' in str(n).lower() for n in names)):
            flag('recovery_open', '#' + str(issue.get('number', '?')), title + '; comments do not prove a worker launch or a verified fix.')
    for service, state in sorted(objects('services').items()):
        if state.get('ActiveState') != 'active' or state.get('LoadState') != 'loaded' or not str(state.get('MainPID', '')).isdigit() or int(state.get('MainPID', '0')) <= 0:
            flag('local_service_unverified', service, 'Local service snapshot does not show a loaded active unit with a PID.')
    if not snapshot.get('services'):
        flag('local_services_missing', 'services', 'No local service snapshot; remote services are never inferred.')
    dispatcher = snapshot.get('dispatcher')
    if dispatcher is None:
        flag('dispatcher_journal_unknown', 'dispatcher', 'No optional dispatcher journal supplied; service activity alone does not prove useful dispatch.')
    elif not isinstance(dispatcher, dict):
        flag('invalid_record', 'dispatcher', 'Dispatcher journal must be an object.')
    elif dispatcher.get('state', dispatcher.get('phase')) not in ('idle', 'complete', 'published'):
        flag('dispatcher_needs_review', 'dispatcher', 'Journal state ' + str(dispatcher.get('state', dispatcher.get('phase', 'unknown'))) + ' requires review; no retry is performed.')
    findings.sort(key=lambda f: (f['subject'], f['code'], f['detail']))
    return {'schema_version': 1, 'observed_utc': now.isoformat().replace('+00:00', 'Z'),
            'main_sha': snapshot.get('main_sha'), 'authority': snapshot.get('authority'),
            'control': snapshot.get('control'), 'workers': workers, 'findings': findings,
            'report_sources': report_sources, 'authority_ref': snapshot.get('authority_ref'),
            'local_services': snapshot.get('services', {}),
            'actionable': any(f['code'] != 'remote_activity_reported_only' for f in findings),
            'scope': 'Read-only observations; no remote process, authority transfer, resource grant or project completion is verified.'}


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--root', type=Path, default=Path.cwd())
    parser.add_argument('--no-fetch', action='store_true')
    parser.add_argument('--ref', default='origin/main')
    parser.add_argument('--snapshot', type=Path, help='Analyze saved JSON instead of running any commands')
    parser.add_argument('--gh-json', type=Path, help='Inject saved gh issue-list JSON')
    parser.add_argument('--dispatcher-json', type=Path, help='Optional saved local dispatcher journal')
    parser.add_argument('--now', help='Timezone-qualified ISO timestamp for reproducible analysis')
    parser.add_argument('--stale-seconds', type=int, default=600)
    parser.add_argument('--resource-seconds', type=int, default=1800)
    args = parser.parse_args(argv)
    try:
        if min(args.stale_seconds, args.resource_seconds) < 1:
            raise ValueError('freshness limits must be positive')
        snapshot = json.loads(args.snapshot.read_text()) if args.snapshot else collect(
            args.root, fetch=not args.no_fetch, ref=args.ref,
            gh_output=args.gh_json.read_text() if args.gh_json else None,
            dispatcher=json.loads(args.dispatcher_json.read_text()) if args.dispatcher_json else None)
        if not isinstance(snapshot, dict):
            raise ValueError('snapshot must be an object')
        report = analyze(snapshot, now=utc(args.now) if args.now else datetime.now(timezone.utc),
                         stale_seconds=args.stale_seconds, resource_seconds=args.resource_seconds)
        print(json.dumps(report, indent=2, sort_keys=True))
        return 1 if report['actionable'] else 0
    except (ValueError, OSError, TypeError) as exc:
        print(json.dumps({'error': str(exc)}, sort_keys=True))
        return 2


if __name__ == '__main__':
    raise SystemExit(main())
