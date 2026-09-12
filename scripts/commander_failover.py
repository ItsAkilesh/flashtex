#!/usr/bin/env python3
"""Read-only Commander death/quiescence witness. Never infers death from silence.

A terminal receipt is evidence for a separately reviewed, serialized authority
claim; this program never launches a model, kills the Commander, or claims main.
Linux process identity is PID + boot ID + start ticks. Quota with a live process
remains blocked. The actual hosted quota-to-terminal adapter is not implemented.
"""
import argparse
from datetime import datetime, timezone
import json
import os
from pathlib import Path
import subprocess
import time


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
            text = ' '.join(args)
            executable = Path(args[0]).name
            if (('git' in executable and any(a in ('push', 'commit', 'merge') for a in args))
                    or ('coord.py' in text and 'publish' in args)
                    or 'dispatch_loop.py' in text
                    or ('integrate.py' in text and any(a in ('promote', 'merge', 'sync') for a in args))
                    or executable in ('cursor-agent', 'cursor')):
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
    for key in ('pid', 'start_ticks'):
        if type(pin.get(key)) is not int or pin[key] <= 0:
            raise ValueError('invalid pinned process ' + key)
    if not isinstance(pin.get('boot_id'), str) or not pin['boot_id']:
        raise ValueError('missing pinned boot identity')
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


def inspect(config, root):
    def git(*args):
        return subprocess.check_output(['git', *args], cwd=root, text=True, timeout=30).strip()
    authority = json.loads(git('show', 'origin/main:coordination/authority.json'))
    common = Path(git('rev-parse', '--git-common-dir'))
    if not common.is_absolute():
        common = (root / common).resolve()
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
        except (OSError, ValueError, RuntimeError, subprocess.SubprocessError) as exc:
            result = {'state': 'blocked', 'claim_authorized': False, 'error': type(exc).__name__}
        args.output.parent.mkdir(parents=True, exist_ok=True)
        temporary = args.output.with_suffix('.tmp')
        temporary.write_text(json.dumps(result, indent=2) + '\n')
        temporary.replace(args.output)
        print(json.dumps({'state': result['state'], 'claim_authorized': False}), flush=True)
        if not args.watch:
            return 0
        time.sleep(args.interval)


if __name__ == '__main__':
    main()
