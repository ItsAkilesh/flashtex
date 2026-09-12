#!/usr/bin/env python3
"""Deterministic, fail-closed revival gate for the Mac standby orchestrator-jaysen-opus.

Consumes the Linux terminal witness published by scripts/commander_failover.py on
the configured witness branch (coordination/failover.json: witness_branch) and the
current authority on origin/main. It NEVER writes the repository, never pushes,
never commits, never dispatches, never calls a model. Its only outputs are a JSON
verdict on stdout and, in --once mode, the exact claim it WOULD prepare, printed
and not applied. `revival_permitted` is a flag for a human/parent review step;
`executed` is always False.

Verdicts: `no_action` (default for every alive, ambiguous, stale, competing or
unreadable state) or `revival_permitted` (explicit terminal witness, pinned
process match, stopped publishers, no journals, no competing claim, main
unchanged since observation, no pending local claim). Absence of evidence is
never evidence of termination.
"""
import argparse
from datetime import datetime, timezone
import json
from pathlib import Path
import re
import subprocess
import sys

SUCCESSOR = 'orchestrator-jaysen-opus'
WITNESS_AUTHOR = 'FlashTeX Witness <witness@flashtex.invalid>'
WITNESS_EXECUTOR = 'Commit-Executor: commander_failover.py deterministic local witness'
RECEIPT_PATH = 'coordination/failover/witness-linux.json'
SHA = re.compile(r'^[0-9a-f]{40}$')


def _pin_ok(pin):
    return (isinstance(pin, dict) and type(pin.get('pid')) is int and pin['pid'] > 0
            and type(pin.get('start_ticks')) is int and pin['start_ticks'] > 0
            and isinstance(pin.get('boot_id'), str) and bool(pin['boot_id']))


def evaluate(config, authority, main_sha, witness_refs, receipt, receipt_commit, local_journal, now=None):
    """Pure decision. Every argument is data already read; nothing here touches Git.

    config: coordination/failover.json from origin/main (None if absent).
    authority: coordination/authority.json from origin/main.
    main_sha: freshly fetched origin/main.
    witness_refs: {branch_name: sha} of remote agent/orchestrator-witness/* heads.
    receipt: parsed RECEIPT_PATH from the configured witness branch, or None.
    receipt_commit: {'sha','parents':[...],'author','message'} of that branch tip, or None.
    local_journal: parsed local claim journal of a previous one-shot run, or None.
    """
    now = now or datetime.now(timezone.utc)
    reasons = []

    def block(reason):
        reasons.append(reason)

    if not isinstance(config, dict):
        block('failover_config_absent')
        return _verdict(reasons, config, receipt, main_sha)
    predecessor = config.get('predecessor_id')
    if config.get('successor_id') != SUCCESSOR:
        block('config_names_different_successor')
    if config.get('mode') != 'read_only_standby':
        block('config_mode_not_read_only_standby')
    if not _pin_ok(config.get('process')):
        block('config_process_pin_invalid')
    if not isinstance(authority, dict):
        block('authority_unreadable')
    else:
        if authority.get('commander_id') == SUCCESSOR:
            block('already_claimed_by_successor')
        elif authority.get('commander_id') != predecessor:
            block('competing_or_newer_claim_on_main')
        if authority.get('authority_state') != 'active':
            block('authority_state_not_active')
    if not isinstance(main_sha, str) or not SHA.match(main_sha):
        block('main_sha_unavailable')

    branch = config.get('witness_branch')
    if not isinstance(branch, str) or not branch.startswith('agent/orchestrator-witness/'):
        block('witness_branch_not_configured')
    elif branch not in (witness_refs or {}):
        block('no_terminal_receipt_published')  # alive or unknown: never inferred as dead
    if receipt is None:
        if 'no_terminal_receipt_published' not in reasons:
            block('receipt_missing_on_witness_branch')
        return _verdict(reasons, config, receipt, main_sha)

    if not isinstance(receipt, dict):
        block('receipt_unreadable')
        return _verdict(reasons, config, receipt, main_sha)
    if receipt.get('state') != 'terminal_quiescent_observed':
        block('receipt_state_not_terminal:' + str(receipt.get('state')))
    if receipt.get('claim_authorized') is not False:
        block('receipt_asserts_claim_authorization')  # the witness never authorizes; a True is tampering/misuse
    if receipt.get('reasons'):
        block('receipt_lists_blockers')
    if receipt.get('predecessor_id') != predecessor or receipt.get('successor_id') != SUCCESSOR:
        block('receipt_identity_mismatch')
    if receipt.get('process_pin') != config.get('process'):
        block('receipt_pin_does_not_match_current_config')  # stale or recycled process pin
    services = receipt.get('services')
    if (not isinstance(services, list) or not services
            or any(not isinstance(s, dict) or s.get('active') not in ('inactive', 'failed') or s.get('pid') != 0
                   for s in services)):
        block('publisher_services_not_verified_stopped')
    if receipt.get('publisher_processes'):
        block('publication_processes_remain')
    if receipt.get('journal_blockers'):
        block('unresolved_publication_journals')
    observed = receipt.get('observed_main_sha')
    if not isinstance(observed, str) or not SHA.match(observed):
        block('receipt_observed_main_invalid')
    elif observed != main_sha:
        block('main_moved_since_terminal_observation')  # someone wrote main after the witness: reobserve
    try:
        observed_utc = datetime.fromisoformat(str(receipt.get('observed_utc')).replace('Z', '+00:00'))
        if observed_utc.tzinfo is None:
            raise ValueError
        if (observed_utc - now).total_seconds() > 60:
            block('receipt_future_dated')
    except (TypeError, ValueError):
        block('receipt_observed_utc_invalid')

    if not isinstance(receipt_commit, dict):
        block('receipt_commit_unreadable')
    else:
        if receipt_commit.get('author') != WITNESS_AUTHOR:
            block('receipt_commit_author_not_witness')
        if WITNESS_EXECUTOR not in (receipt_commit.get('message') or '').splitlines():
            block('receipt_commit_missing_witness_executor_trailer')
        if receipt_commit.get('parents') != [observed]:
            block('receipt_commit_not_on_observed_main')

    if isinstance(local_journal, dict):
        # Any earlier prepared/pending/pushed claim must be reconciled by hand first
        # (lost or uncertain push, or a prepared commit nobody reviewed).
        block('unreconciled_local_claim_journal:' + str(local_journal.get('state')))
    elif local_journal is not None:
        block('local_claim_journal_unreadable')

    return _verdict(reasons, config, receipt, main_sha)


def _verdict(reasons, config, receipt, main_sha):
    permitted = not reasons
    return {'schema_version': 1, 'successor_id': SUCCESSOR,
            'verdict': 'revival_permitted' if permitted else 'no_action',
            'revival_permitted': permitted, 'executed': False, 'reasons': reasons,
            'main_sha': main_sha,
            'receipt_state': receipt.get('state') if isinstance(receipt, dict) else None,
            'witness_branch': config.get('witness_branch') if isinstance(config, dict) else None,
            'evaluated_utc': datetime.now(timezone.utc).isoformat(),
            'limitations': ('Deterministic local evaluation of published evidence only. '
                            'revival_permitted is a review flag, never an executed claim; the claim '
                            'procedure in tools/commander-standby/README.md requires a human/parent '
                            'decision and a fresh reread of authority immediately before any push.')}


def prepared_claim(config, authority, main_sha, receipt_commit_sha, now=None):
    """The authority record a takeover WOULD publish. Returned as data; never written."""
    now = now or datetime.now(timezone.utc)
    claim = dict(authority)
    claim.update({
        'commander_id': SUCCESSOR, 'machine': 'mac-m1max-a', 'authority_state': 'active',
        'claim_mode': 'positive_termination_witness', 'predecessor': config['predecessor_id'],
        'predecessor_main_writes_quiesced': True, 'claim_base_main': main_sha,
        'dispatch_service': 'stopped:' + ','.join(config.get('publisher_services', [])),
        'dispatch_service_pid_at_claim': 0,
        'updated_utc': now.strftime('%Y-%m-%dT%H:%M:%SZ'),
        'agent_handle': 'Claude Code Opus subagent of mac-claude-a',
        'model': 'Claude Opus',
        'handoff_evidence': ('terminal witness commit ' + receipt_commit_sha + ' on ' + config['witness_branch']
                             + ' pins process ' + json.dumps(config['process'], sort_keys=True)
                             + ' terminated with publishers stopped; observed main ' + main_sha),
        'claim_commit_executor': 'git via Claude Code (mac-m1max-a primary author jay3332; Cursor not logged in)',
        'cursor_quota_state': 'not logged in on mac-m1max-a; no purchase',
    })
    return claim


HUMAN_CONFIRMATION = 'I verified tools/commander-standby/revival-packet.md and authorize a LOCAL claim commit'


def prepare_local_claim(root, evidence, verdict, human_confirmed, journal_path, now=None):
    """Beyond printing: create the claim commit in a detached temporary worktree. NO PUSH.

    Refuses unless the exact human confirmation phrase is supplied and the verdict is
    revival_permitted. Writes only a temp worktree and the local journal. The push
    to main is a separate manual, non-force step documented in README.md.
    """
    if human_confirmed != HUMAN_CONFIRMATION:
        raise PermissionError('explicit human confirmation phrase required; nothing prepared')
    if not verdict.get('revival_permitted') or verdict.get('executed'):
        raise PermissionError('verdict is not revival_permitted; nothing prepared')
    existing = read_journal(journal_path)
    if existing is not None:
        raise RuntimeError('local claim journal already exists; reconcile it before preparing again')
    base = evidence['main_sha']
    if git(root, 'rev-parse', 'origin/main') != base:
        raise RuntimeError('origin/main moved since evaluation; re-run the gate')
    claim = prepared_claim(evidence['config'], evidence['authority'], base, evidence['receipt_commit']['sha'], now)
    import tempfile
    folder = Path(tempfile.mkdtemp(prefix='flashtex-standby-claim-'))
    journal_path.parent.mkdir(parents=True, exist_ok=True)
    journal_path.write_text(json.dumps({'state': 'pending', 'base': base, 'worktree': str(folder)}) + '\n')
    git(root, 'worktree', 'add', '--detach', str(folder), base)
    target = folder / 'coordination/authority.json'
    target.write_text(json.dumps(claim, indent=2) + '\n')
    git(folder, 'add', '--', 'coordination/authority.json')
    body = ('Claim Commander authority for orchestrator-jaysen-opus after verified terminal witness\n\n'
            'Evidence: ' + claim['handoff_evidence'] + '\n\n'
            'Implementation-Agent: Claude Code Opus subagent orchestrator-jaysen-opus (parent mac-claude-a)\n'
            'Commit-Executor: git via Claude Code (standby_gate.py prepare, human-confirmed)\n')
    git(folder, 'commit', '-m', body)
    sha = git(folder, 'rev-parse', 'HEAD')
    journal_path.write_text(json.dumps({'state': 'prepared_local', 'base': base, 'commit': sha,
                                        'worktree': str(folder), 'pushed': False}) + '\n')
    return {'state': 'prepared_local', 'commit': sha, 'worktree': str(folder), 'base': base, 'pushed': False,
            'manual_push': 'git -C ' + str(folder) + ' push origin HEAD:refs/heads/main   # non-force; rejected => stop'}


# ---------------------------------------------------------------- evidence collection (read-only)

def git(root, *args, check=True):
    result = subprocess.run(['git', *args], cwd=root, capture_output=True, text=True, timeout=60)
    if check and result.returncode:
        raise RuntimeError('git ' + ' '.join(args) + ': ' + result.stderr.strip())
    return result.stdout.strip()


def show_json(root, ref, path):
    result = subprocess.run(['git', 'show', ref + ':' + path], cwd=root, capture_output=True, text=True, timeout=60)
    if result.returncode:
        return None
    try:
        return json.loads(result.stdout)
    except ValueError:
        return 'unreadable'


def collect(root, remote='origin', fetch=True):
    if fetch:
        git(root, 'fetch', '--prune', remote)
    main_sha = git(root, 'rev-parse', remote + '/main')
    config = show_json(root, remote + '/main', 'coordination/failover.json')
    authority = show_json(root, remote + '/main', 'coordination/authority.json')
    refs = {}
    for line in git(root, 'ls-remote', '--heads', remote, 'refs/heads/agent/orchestrator-witness/*').splitlines():
        sha, ref = line.split('\t', 1)
        refs[ref[len('refs/heads/'):]] = sha
    receipt = receipt_commit = None
    branch = config.get('witness_branch') if isinstance(config, dict) else None
    if branch in refs:
        tip = refs[branch]
        receipt = show_json(root, tip, RECEIPT_PATH)
        meta = git(root, 'show', '-s', '--format=%H%n%P%n%an <%ae>%n%B', tip, check=False)
        if meta:
            lines = meta.split('\n', 3)
            receipt_commit = {'sha': lines[0], 'parents': lines[1].split(), 'author': lines[2],
                              'message': lines[3] if len(lines) > 3 else ''}
    return {'main_sha': main_sha, 'config': config, 'authority': authority, 'witness_refs': refs,
            'receipt': receipt, 'receipt_commit': receipt_commit}


def read_journal(path):
    if path is None or not path.exists():
        return None
    try:
        return json.loads(path.read_text())
    except ValueError:
        return 'unreadable'


def main():
    p = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    p.add_argument('--repo', type=Path, default=Path(__file__).resolve().parents[2])
    p.add_argument('--remote', default='origin')
    p.add_argument('--no-fetch', action='store_true')
    p.add_argument('--journal', type=Path, help='local claim journal of a previous one-shot run, if any')
    p.add_argument('--once', action='store_true',
                   help='one guarded invocation: evaluate, and if permitted print the claim that WOULD be '
                        'prepared. Nothing is written, committed or pushed.')
    p.add_argument('--prepare', action='store_true',
                   help='with --once and --human-confirmed: create the claim commit in a detached temporary '
                        'worktree (LOCAL ONLY, never pushed). Refused without the exact confirmation phrase.')
    p.add_argument('--human-confirmed', default=None, metavar='PHRASE',
                   help='exact phrase: ' + json.dumps(HUMAN_CONFIRMATION))
    args = p.parse_args()
    if args.prepare and not args.once:
        p.error('--prepare requires --once')
    if args.prepare and not args.journal:
        p.error('--prepare requires --journal so the local claim is journaled and never duplicated')
    try:
        evidence = collect(args.repo, args.remote, fetch=not args.no_fetch)
    except (RuntimeError, OSError, subprocess.SubprocessError) as exc:
        print(json.dumps({'verdict': 'no_action', 'revival_permitted': False, 'executed': False,
                          'reasons': ['evidence_collection_failed:' + type(exc).__name__]}, indent=2))
        return 0
    verdict = evaluate(evidence['config'], evidence['authority'], evidence['main_sha'], evidence['witness_refs'],
                       evidence['receipt'], evidence['receipt_commit'], read_journal(args.journal))
    if args.once and verdict['revival_permitted']:
        verdict['would_prepare_claim'] = prepared_claim(evidence['config'], evidence['authority'],
                                                        evidence['main_sha'], evidence['receipt_commit']['sha'])
        verdict['next_manual_steps'] = [
            'Re-run this gate immediately before acting; if the verdict changes, stop.',
            'Follow tools/commander-standby/README.md steps 1-7 by hand; fill revival-packet.md first.',
            'Push non-force only; a rejected push means another claim won: stop all global writes.']
        if args.prepare:
            try:
                verdict['prepared_local_claim'] = prepare_local_claim(
                    args.repo, evidence, verdict, args.human_confirmed, args.journal)
            except (PermissionError, RuntimeError, OSError, subprocess.SubprocessError) as exc:
                verdict['prepare_refused'] = type(exc).__name__ + ': ' + str(exc)
    elif args.prepare:
        verdict['prepare_refused'] = 'verdict is no_action; nothing prepared'
    print(json.dumps(verdict, indent=2))
    return 0


if __name__ == '__main__':
    sys.exit(main())
