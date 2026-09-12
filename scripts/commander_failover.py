#!/usr/bin/env python3
"""Read-only Commander death/quiescence witness. Never infers death from silence.

A terminal receipt is evidence for a separately reviewed, serialized authority
claim; this program never launches a model, kills the Commander, or claims main.
An explicitly configured dispatcher can stop only after exact process death.
Linux process identity is PID + boot ID + start ticks. Quota with a live process
remains blocked. The actual hosted quota-to-terminal adapter is not implemented.
"""
import argparse
from datetime import datetime, timezone
import json
import os
import re
from pathlib import Path
import subprocess
import tempfile
import time


def validate_pin(pin):
    for key in ('pid', 'start_ticks'):
        if type(pin.get(key)) is not int or pin[key] <= 0:
            raise ValueError('invalid pinned process ' + key)
    if not isinstance(pin.get('boot_id'), str) or not pin['boot_id']:
        raise ValueError('missing pinned boot identity')


def process_identity(pid, proc=Path('/proc')):
    boot_id = (proc / 'sys/kernel/random/boot_id').read_text().strip()
    if not boot_id:
        raise ValueError('process namespace boot identity unavailable')
    try:
        stat = (proc / str(pid) / 'stat').read_text()
    except FileNotFoundError:
        return None
    end = stat.rfind(')')
    fields = stat[end + 2:].split()
    if end < 0 or len(fields) < 20:
        raise ValueError('unreadable process identity')
    return {'pid': pid, 'start_ticks': int(fields[19]), 'state': fields[0],
            'boot_id': boot_id}


def terminal(pin, observed):
    if observed is None:
        return True
    if observed['boot_id'] != pin['boot_id'] or observed['start_ticks'] != pin['start_ticks']:
        return True  # Original process is gone; a recycled PID is a different process.
    return observed['state'] in ('Z', 'X')


def journal_blockers(common):
    paths = list((common / 'flashtex').glob('*.json'))
    paths += list((common / 'worktrees').glob('*/flashtex/*.json'))
    blocked = []
    for path in paths:
        try:
            if path.stat().st_size > 1048576:
                blocked.append(str(path) + ':oversized')
                continue
            value = json.loads(path.read_text())
            if isinstance(value, dict) and value.get('state') in ('pending', 'in_flight', 'unknown'):
                blocked.append(str(path) + ':unresolved')
        except (OSError, ValueError):
            blocked.append(str(path) + ':unreadable')
    return blocked


def looks_like_publisher(args):
    text = ' '.join(args)
    tokens = re.split(r'[\s;|&()]+', text)
    # Include shell wrappers that may publish after a currently running build.
    return ((any(Path(t.strip('"\'')).name == 'git' for t in tokens)
             and any(t.strip('"\'') in ('push', 'commit', 'merge') for t in tokens))
            or ('coord.py' in text and 'publish' in text)
            or 'dispatch_loop.py' in text
            or ('integrate.py' in text and any(t in text for t in ('promote', 'merge', 'sync')))
            or (args and Path(args[0]).name in ('cursor-agent', 'cursor')))


def publication_processes(proc=Path('/proc')):
    found = []
    for directory in proc.iterdir():
        if not directory.name.isdigit() or int(directory.name) == os.getpid():
            continue
        try:
            if directory.stat().st_uid != os.getuid():
                continue
            args = (directory / 'cmdline').read_bytes().split(b'\0')
            args = [a.decode('utf-8', 'replace') for a in args if a]
            if not args:
                continue
            # Retain only process identity, never command contents/secrets in reports.
            if looks_like_publisher(args):
                identity = process_identity(int(directory.name), proc)
                if identity and identity['state'] not in ('Z', 'X'):
                    found.append(identity)
        except FileNotFoundError:
            continue
        except PermissionError:
            raise RuntimeError('cannot inspect all local publication processes')
    return found


def evaluate(config, authority, observed, services, journals, processes):
    reasons = []
    pin = config['process']
    validate_pin(pin)
    if authority.get('commander_id') != config['predecessor_id'] or authority.get('authority_state') != 'active':
        reasons.append('authority_changed')
    if not terminal(pin, observed):
        reasons.append('exact_commander_process_still_exists')
    if not services or any(s.get('active') not in ('inactive', 'failed') or s.get('pid') != 0 for s in services):
        reasons.append('publisher_services_not_verified_stopped')
    if journals:
        reasons.append('unresolved_publication_journals')
    if processes:
        reasons.append('publication_processes_remain')
    return {'schema_version': 1, 'state': 'terminal_quiescent_observed' if not reasons else 'blocked',
            'predecessor_id': config['predecessor_id'], 'successor_id': config['successor_id'],
            'process_pin': pin, 'reasons': reasons, 'publisher_processes': processes,
            'journal_blockers': journals, 'services': services,
            'observed_utc': datetime.now(timezone.utc).isoformat(),
            'claim_authorized': False,
            'limitations': 'Local observation only. Revalidate authority and independently review witness before a non-force claim. Live-process quota exhaustion is not terminal evidence.'}


def quiesce_after_terminal(config, authority, observed, stop):
    """Stop only the explicitly named dispatcher after exact process death.

    Does not stop the hosting process or arbitrary jobs; remaining jobs/journals
    keep the witness blocked. A changed authority prevents this old monitor acting.
    """
    validate_pin(config['process'])
    if (config.get('allow_dispatcher_stop_after_process_exit') is not True
            or authority.get('commander_id') != config['predecessor_id']
            or authority.get('authority_state') != 'active'
            or not terminal(config['process'], observed)):
        return False
    if config['publisher_services'] != ['flashtex-dispatch.service']:
        raise ValueError('unreviewed service stop target')
    stop('flashtex-dispatch.service')
    return True


def inspect(config, root):
    def git(*args):
        return subprocess.check_output(['git', *args], cwd=root, text=True, timeout=30).strip()
    validate_pin(config['process'])
    authority = json.loads(git('show', 'origin/main:coordination/authority.json'))
    control = json.loads(git('show', 'origin/main:coordination/control.json'))
    if control.get('state') in ('stop_requested', 'stopped'):
        return {'state': 'blocked', 'claim_authorized': False, 'reasons': ['explicit_user_stop']}

    common = Path(git('rev-parse', '--git-common-dir'))
    if not common.is_absolute():
        common = (root / common).resolve()
    observed = process_identity(config['process']['pid'])
    quiesce_after_terminal(config, authority, observed,
        lambda unit: subprocess.run(['systemctl', '--user', 'stop', unit],
                                    capture_output=True, timeout=15, check=True))
    services = []
    for unit in config['publisher_services']:
        p = subprocess.run(['systemctl', '--user', 'show', unit, '-p', 'ActiveState', '-p', 'MainPID'],
                           capture_output=True, text=True, timeout=10, check=True)
        fields = dict(line.split('=', 1) for line in p.stdout.splitlines() if '=' in line)
        services.append({'unit': unit, 'active': fields.get('ActiveState'), 'pid': int(fields.get('MainPID', '-1'))})
    result = evaluate(config, authority, process_identity(config['process']['pid']), services,
                      journal_blockers(common), publication_processes())
    result['observed_main_sha'] = git('rev-parse', 'origin/main')
    return result


def publish_terminal_receipt(config, result, root, journal):
    """One deterministic witness-branch commit, never an authority/main write.

    A pending journal is never retried automatically, including uncertain push.
    The designated successor must inspect and reconcile that exact branch.
    """
    if result.get('state') != 'terminal_quiescent_observed':
        return None
    if journal.exists():
        old = json.loads(journal.read_text())
        if old.get('state') == 'published':
            return old.get('commit')
        raise RuntimeError('terminal receipt publication pending; reconcile instead of retrying')
    branch = config['witness_branch']
    if not branch.startswith('agent/orchestrator-witness/') or '..' in branch:
        raise ValueError('invalid witness branch')
    def git(*args, cwd=root):
        return subprocess.check_output(['git', *args], cwd=cwd, text=True, stderr=subprocess.PIPE, timeout=45).strip()
    git('check-ref-format', 'refs/heads/' + branch)
    git('fetch', 'origin')
    base = git('rev-parse', 'origin/main')
    if base != result['observed_main_sha']:
        raise RuntimeError('main changed since terminal observation; reobserve before receipt')
    authority = json.loads(git('show', base + ':coordination/authority.json'))
    if authority.get('commander_id') != config['predecessor_id'] or authority.get('authority_state') != 'active':
        raise RuntimeError('authority changed before witness publication')
    record = {'state': 'pending', 'branch': branch, 'base': base}
    journal.write_text(json.dumps(record) + '\n')
    folder = Path(tempfile.mkdtemp(prefix='flashtex-terminal-witness-'))
    git('worktree', 'add', '--detach', str(folder), base)
    relative = 'coordination/failover/witness-linux.json'
    target = folder / relative
    target.parent.mkdir(parents=True, exist_ok=True)
    target.write_text(json.dumps(result, indent=2) + '\n')
    git('add', '--', relative, cwd=folder)
    body = ('Observe terminal Commander and stopped publication jobs\n\n'
            'Implementation-Agent: Codex Astra\n'
            'Commit-Executor: commander_failover.py deterministic local witness\n'
            'Co-authored-by: ' + config['witness_coauthor'])
    git('-c', 'user.name=FlashTeX Witness', '-c', 'user.email=witness@flashtex.invalid',
        'commit', '-m', body, cwd=folder)
    commit = git('rev-parse', 'HEAD', cwd=folder)
    record.update(commit=commit, worktree=str(folder))
    journal.write_text(json.dumps(record) + '\n')
    git('push', 'origin', 'HEAD:refs/heads/' + branch, cwd=folder)
    record['state'] = 'published'
    journal.write_text(json.dumps(record) + '\n')
    return commit


def write_observation(output, result):
    """A full disk must not terminate the read-only watcher or replace its last receipt."""
    try:
        output.parent.mkdir(parents=True, exist_ok=True)
        temporary = output.with_suffix('.tmp')
        temporary.write_text(json.dumps(result, indent=2) + '\n')
        temporary.replace(output)
    except OSError as exc:
        return {'state': 'observation_write_failed', 'claim_authorized': False,
                'error': type(exc).__name__, 'errno': exc.errno}
    return {'state': result['state'], 'claim_authorized': False}


def main():
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument('--config', type=Path, required=True)
    p.add_argument('--repo', type=Path, required=True)
    p.add_argument('--output', type=Path, required=True)
    p.add_argument('--watch', action='store_true')
    p.add_argument('--interval', type=float, default=30)
    args = p.parse_args()
    if args.interval < 5:
        p.error('interval must be at least five seconds')
    config = json.loads(args.config.read_text())
    while True:
        try:
            result = inspect(config, args.repo)
            if config.get('publish_terminal_receipt') is True:
                receipt = publish_terminal_receipt(config, result, args.repo,
                                                   args.output.with_suffix('.publication.json'))
                if receipt:
                    result['witness_commit'] = receipt
        except (OSError, ValueError, RuntimeError, subprocess.SubprocessError) as exc:
            result = {'state': 'blocked', 'claim_authorized': False, 'error': type(exc).__name__}
        print(json.dumps(write_observation(args.output, result)), flush=True)
        if not args.watch:
            return 0
        time.sleep(args.interval)


if __name__ == '__main__':
    main()
