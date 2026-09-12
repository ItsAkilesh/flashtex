#!/usr/bin/env python3
"""Prepare and verify concurrent branch integration in an isolated worktree."""
import argparse
import json
from pathlib import Path
import re
import subprocess
import sys

import coord


def prepare(root, args):
    coord.branch(root, 'commander')
    coord.identifier(args.name)
    if not args.ref.startswith('origin/agent/') or not coord.BRANCH.fullmatch(args.ref[7:]):
        raise ValueError('candidate must be a fetched origin/agent/<id>/<task> branch')
    coord.git(root, 'fetch', 'origin', '--prune')
    base = coord.git(root, 'rev-parse', 'origin/main')
    candidate = coord.git(root, 'rev-parse', args.ref + '^{commit}')
    if candidate != args.sha or not re.fullmatch('[0-9a-f]{40}', args.sha):
        raise ValueError('candidate moved; review its current full SHA before preparing')
    if coord.run(['git', 'merge-base', '--is-ancestor', candidate, base], cwd=root, check=False).returncode == 0:
        raise ValueError('candidate already integrated')
    target = Path(args.worktree).resolve()
    if target.exists():
        raise ValueError('worktree destination already exists; preserve it and inspect before retrying')
    name = 'agent/commander/integrate-' + args.name
    coord.git(root, 'worktree', 'add', '-b', name, str(target), base)
    result = coord.run(['git', 'merge', '--no-commit', '--no-ff', candidate], cwd=target, check=False)
    if not coord.git(target, 'rev-parse', '--verify', 'MERGE_HEAD', check=False):
        raise RuntimeError('merge did not start; inspect preserved worktree: ' + result.stderr)
    conflicts = [p for p in coord.git(target, 'diff', '--name-only', '--diff-filter=U', '-z').split('\0') if p]
    if result.returncode and not conflicts:
        raise RuntimeError('merge failed without resolvable conflicts: ' + result.stderr)
    packet = {'schema_version': 1, 'base_sha': base, 'candidate_sha': candidate,
              'candidate_ref': args.ref, 'branch': name, 'conflicts': conflicts,
              'prepared_utc': coord.stamp(), 'state': 'prepared',
              'baseline_tree': None if conflicts else coord.git(target, 'write-tree')}
    # Stage entries permit checking that cleanly merged paths were left untouched.
    packet['clean_entries'] = [line for line in coord.git(target, 'ls-files', '--stage', '-z').split('\0')
                               if line and line.split('\t', 1)[-1] not in conflicts]
    coord.write_json(coord.local_state(target) / 'integration.json', packet)
    print(json.dumps({'worktree': str(target), 'branch': name, 'conflicts': conflicts,
                      'next': 'Review merge inputs, then run finish in this worktree with validation commands.'}))


def verify_commit(root, packet):
    head = coord.git(root, 'rev-parse', 'HEAD')
    if coord.branch(root, 'commander') != packet['branch']:
        raise ValueError('integration branch changed')
    if coord.git(root, 'show', '-s', '--format=%P', head).split() != [packet['base_sha'], packet['candidate_sha']]:
        raise ValueError('expected one merge commit with the pinned two parents')
    if coord.git(root, 'status', '--porcelain'):
        raise ValueError('integration worktree is dirty')
    identity = coord.git(root, 'show', '-s', '--format=%an <%ae>%n%cn <%ce>', head).splitlines()
    if identity != ['Cursor <cursor@flashtex.invalid>'] * 2:
        raise ValueError('merge must have Cursor author and committer')
    message = coord.git(root, 'show', '-s', '--format=%B', head)
    if not re.search(r'^Commit-Executor: Cursor CLI$', message, re.M) or not re.search(r'^Implementation-Agent: .+$', message, re.M):
        raise ValueError('missing merge provenance')
    conflicts = set(packet['conflicts'])
    actual = [line for line in coord.git(root, 'ls-files', '--stage', '-z').split('\0')
              if line and line.split('\t', 1)[-1] not in conflicts]
    if actual != packet['clean_entries']:
        raise ValueError('Cursor changed paths outside the conflict set; review manually, not pushing')
    if packet['baseline_tree'] and coord.git(root, 'rev-parse', head + '^{tree}') != packet['baseline_tree']:
        raise ValueError('clean merge tree unexpectedly changed')
    return head


def finish(root, args):
    with coord.lock(root):
        return finish_locked(root, args)


def finish_locked(root, args):
    path = coord.local_state(root) / 'integration.json'
    packet = json.loads(path.read_text())
    commands = [json.loads(value) for value in args.check]
    if any(not isinstance(c, list) or not c or any(not isinstance(x, str) or not x for x in c) for c in commands):
        raise ValueError('--check must be a nonempty JSON argv array; no implicit shell')
    if coord.branch(root, 'commander') != packet['branch']:
        raise ValueError('run finish in the prepared integration worktree')
    if packet['state'] == 'prepared':
        if not args.allocation:
            raise ValueError('Cursor resolution/commit requires an authorized allocation')
        if coord.git(root, 'rev-parse', 'HEAD') != packet['base_sha']:
            raise ValueError('HEAD changed before Cursor execution; inspect the worktree')
        if coord.git(root, 'rev-parse', 'MERGE_HEAD') != packet['candidate_sha']:
            raise ValueError('merge input changed')
        packet.update(state='cursor_started', allocation=args.allocation, started_utc=coord.stamp())
        coord.write_json(path, packet)  # Persist before spending; retry must inspect the outcome.
        prompt = ('Read AGENTS.md and relevant contracts. This is one bounded integration resolution '
                  'and commit session authorized under ' + args.allocation + '. Resolve the merge '
                  'conflicts by preserving intended behavior from both parents. Only edit conflict '
                  'paths: ' + json.dumps(packet['conflicts']) + '. If intent is ambiguous or other '
                  'paths need editing, stop and report. Never blanket-select ours/theirs. Do not '
                  'change tests to hide failures. No nested agents, other paid calls, push, branch '
                  'switching, or history rewriting. Stage resolved paths and execute git commit '
                  'yourself with author AND committer Cursor <cursor@flashtex.invalid>, subject '
                  'integration: combine reviewed worker changes, and truthful trailers '
                  'Implementation-Agent: Cursor (merge resolution) and Commit-Executor: Cursor CLI. '
                  'The harness runs validation after your commit. Return commit SHA.')
        result = coord.run(['cursor-agent', '--print', '--trust', '--auto-review', '--output-format', 'text', prompt],
                           cwd=root, timeout=args.timeout, check=False)
        packet['cursor_returncode'] = result.returncode
        coord.write_json(path, packet)
        if result.returncode:
            raise RuntimeError('Cursor failed; worktree preserved. Inspect before rerunning finish: ' + result.stderr[-1000:])
    # A timeout/failure never automatically starts another paid session. An existing
    # valid merge can be checked/resumed; unresolved work requires active inspection.
    head = verify_commit(root, packet)
    packet.update(state='validating', merge_sha=head, checks=[])
    coord.write_json(path, packet)
    for command in commands:
        result = coord.run(command, cwd=root, timeout=args.check_timeout, check=False)
        packet['checks'].append({'argv': command, 'exit_code': result.returncode,
                                 'output_tail': (result.stdout + result.stderr)[-4000:]})
        coord.write_json(path, packet)
        if result.returncode:
            raise RuntimeError('Validation failed; nothing pushed. Evidence in ' + str(path))
    if verify_commit(root, packet) != head:
        raise ValueError('validation changed the integration commit')
    coord.git(root, 'push', '--set-upstream', 'origin', f"HEAD:refs/heads/{packet['branch']}")
    packet.update(state='verified_branch_published', completed_utc=coord.stamp())
    coord.write_json(path, packet)
    print(json.dumps({'commit': head, 'branch': packet['branch'], 'checks': packet['checks'],
                      'main_updated': False, 'evidence': str(path)}))



def promote(root, args):
    with coord.lock(root):
        path = coord.local_state(root) / 'integration.json'
        packet = json.loads(path.read_text())
        if packet['state'] != 'verified_branch_published':
            raise ValueError('integration must pass validation and publish its branch first')
        head = verify_commit(root, packet)
        if head != args.sha or head != packet.get('merge_sha'):
            raise ValueError('review the exact validated merge SHA before promotion')
        coord.git(root, 'fetch', 'origin', '--prune')
        if coord.git(root, 'rev-parse', 'origin/main') != packet['base_sha']:
            raise ValueError('main advanced; prepare and validate a new combined integration')
        if coord.git(root, 'rev-parse', 'origin/' + packet['branch']) != head:
            raise ValueError('published integration branch moved; review again')
        coord.git(root, 'push', 'origin', 'HEAD:refs/heads/main')
        packet.update(state='integrated', promoted_utc=coord.stamp())
        coord.write_json(path, packet)
        print(json.dumps({'main_commit': head, 'integrated': True}))


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    sub = parser.add_subparsers(dest='action', required=True)
    p = sub.add_parser('prepare')
    p.add_argument('--ref', required=True); p.add_argument('--sha', required=True)
    p.add_argument('--name', required=True); p.add_argument('--worktree', required=True)
    p = sub.add_parser('finish')
    p.add_argument('--allocation'); p.add_argument('--timeout', type=int, default=180)
    p.add_argument('--check', action='append', required=True)
    p.add_argument('--check-timeout', type=int, default=180)
    p = sub.add_parser('promote'); p.add_argument('--sha', required=True)
    args = parser.parse_args()
    try:
        {'prepare': prepare, 'finish': finish, 'promote': promote}[args.action](coord.root_dir(), args)
        return 0
    except (ValueError, RuntimeError, OSError, KeyError, subprocess.TimeoutExpired) as exc:
        print('integrate: ' + str(exc), file=sys.stderr)
        return 1


if __name__ == '__main__':
    sys.exit(main())
