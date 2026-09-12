#!/usr/bin/env python3
"""Development-only byte-exact warm/clean runtime-v1 acceptance runner."""
import argparse
import copy
import hashlib
import json
from pathlib import Path
import subprocess
import sys
import time

import reference

PROFILES = {'legacy': None, 'typed': ['rules-v1', 'font-hints-v1']}


def digest(data):
    return hashlib.sha256(data).hexdigest()


def sequence(case, edit, profile):
    """Validate the exact UTF-8 replacement and preserve every project document."""
    documents = []
    for path in case['files']:
        data = (reference.CORPUS / 'cases' / case['id'] / path).read_bytes()
        documents.append({'path': path, 'text': data.decode('utf-8')})
    original = {d['path']: d['text'] for d in documents}
    data = original[edit['path']].encode('utf-8')
    start, end = edit['start_byte'], edit['end_byte']
    old = edit['old'].encode('utf-8')
    if not 0 <= start <= end <= len(data) or data[start:end] != old or data.count(old) != 1:
        raise ValueError('edit no longer matches its unique recorded UTF-8 byte range')
    changed = (data[:start] + edit['new'].encode('utf-8') + data[end:]).decode('utf-8')
    if changed == original[edit['path']]:
        raise ValueError('edit does not change the document')
    requests = []
    for revision, state in enumerate(('original', 'edited', 'undo', 'reapplied'), 1):
        docs = copy.deepcopy(documents)
        if state in ('edited', 'reapplied'):
            next(d for d in docs if d['path'] == edit['path'])['text'] = changed
        payload = {'project_id': 'extended-incremental-' + case['id'],
                   'revision': revision, 'entry_path': case['entry'], 'documents': docs}
        if PROFILES[profile] is not None:
            payload['layout_capabilities'] = PROFILES[profile]
        requests.append({'protocol_version': 1, 'id': case['id'] + '-' + state,
                         'type': 'compile', 'payload': payload})
    return requests


def execute(compiler, requests, directory, timeout):
    """One bounded process. Keep raw output even on crashes, malformed replies or timeouts."""
    directory.mkdir()
    wire = b''.join(json.dumps(r, ensure_ascii=False).encode('utf-8') + b'\n' for r in requests)
    (directory / 'requests.jsonl').write_bytes(wire)
    start = time.monotonic()
    try:
        proc = subprocess.run([str(compiler)], input=wire, capture_output=True, timeout=timeout)
        stdout, stderr = proc.stdout, proc.stderr
        result = {'returncode': proc.returncode, 'timeout': False}
    except subprocess.TimeoutExpired as exc:
        stdout, stderr = exc.stdout or b'', exc.stderr or b''
        result = {'returncode': None, 'timeout': True}
    result.update(seconds=round(time.monotonic() - start, 3),
                  requests_sha256=digest(wire), stdout_sha256=digest(stdout),
                  stderr_sha256=digest(stderr))
    (directory / 'stdout.jsonl').write_bytes(stdout)
    (directory / 'stderr.txt').write_bytes(stderr)
    lines = stdout.splitlines(keepends=True)
    problems = []
    if result['timeout']:
        problems.append('worker timeout')
    if result['returncode'] != 0:
        problems.append('worker did not exit successfully')
    if len(lines) != len(requests):
        problems.append('reply count differs from request count')
    for index, (line, request) in enumerate(zip(lines, requests)):
        try:
            reply = json.loads(line)
            if (reply.get('protocol_version') != 1 or reply.get('type') != 'compile_result'
                    or reply.get('id') != request['id']
                    or reply.get('payload', {}).get('revision') != request['payload']['revision']):
                problems.append('reply envelope/correlation invalid at index ' + str(index))
        except (ValueError, AttributeError, TypeError):
            problems.append('invalid JSON reply at index ' + str(index))
    result['problems'] = problems
    return result, lines


def difference(left, right):
    if left == right:
        return None
    offset = next((i for i, (a, b) in enumerate(zip(left, right)) if a != b), min(len(left), len(right)))
    return {'first_different_byte': offset, 'warm_bytes': len(left), 'clean_bytes': len(right),
            'warm_sha256': digest(left), 'clean_sha256': digest(right)}


def check(compiler, requests, directory, timeout):
    directory.mkdir()
    warm, replies = execute(compiler, requests, directory / 'warm', timeout)
    result = {'warm': warm, 'comparisons': []}
    for index in range(1, len(requests)):
        state = ('original', 'edited', 'undo', 'reapplied')[index]
        clean, clean_replies = execute(compiler, [requests[index]], directory / ('clean-' + state), timeout)
        comparison = {'state': state, 'clean': clean, 'equal': False}
        if not warm['problems'] and not clean['problems']:
            comparison['difference'] = difference(replies[index], clean_replies[0])
            comparison['equal'] = comparison['difference'] is None
        result['comparisons'].append(comparison)
    result['status'] = 'byte-exact' if all(c['equal'] for c in result['comparisons']) else 'failed'
    (directory / 'result.json').write_text(json.dumps(result, indent=2) + '\n')
    return result


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--compiler', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    parser.add_argument('--only', action='append', default=[])
    parser.add_argument('--profile', choices=PROFILES, action='append')
    parser.add_argument('--timeout', type=float, default=20)
    parser.add_argument('--compiler-source-revision', default='unknown',
                        help='caller-supplied provenance; never inferred from artifact or checkout')
    args = parser.parse_args()
    if args.timeout <= 0:
        parser.error('--timeout must be positive')
    compiler = args.compiler.resolve()
    if not compiler.is_file():
        parser.error('compiler executable does not exist')
    manifest = json.loads((reference.CORPUS / 'manifest.json').read_text())
    reference.validate(manifest)
    cases = {c['id']: c for c in manifest['cases']}
    edits = json.loads((reference.CORPUS / 'edits.json').read_text())['edits']
    unknown = set(args.only) - {e['case'] for e in edits}
    if unknown:
        parser.error('unknown edit case: ' + ', '.join(sorted(unknown)))
    selected = [e for e in edits if not args.only or e['case'] in args.only]
    profiles = list(dict.fromkeys(args.profile or PROFILES))
    # Validate all selections before launching any workers or creating output.
    prepared = [(e, profile, sequence(cases[e['case']], e, profile))
                for e in selected for profile in profiles]
    out = args.output.resolve()
    out.mkdir(parents=True, exist_ok=False)
    compiler_hash = reference.sha(compiler)
    report = {'compiler': str(compiler), 'compiler_sha256': compiler_hash,
              'compiler_source_revision_caller_supplied': args.compiler_source_revision,
              'manifest_sha256': reference.sha(reference.CORPUS / 'manifest.json'),
              'edits_sha256': reference.sha(reference.CORPUS / 'edits.json'),
              'comparison': 'complete raw reply bytes, including JSONL terminator; no normalization',
              'scope': 'warm/clean consistency only; no TeX support, PDF parity or latency claim',
              'timeout_seconds_per_worker': args.timeout, 'results': []}
    for edit, profile, requests in prepared:
        name = edit['case'] + '--' + profile
        try:
            result = check(compiler, requests, out / name, args.timeout)
        except OSError as exc:
            result = {'status': 'failed', 'launch_error': str(exc)}
        result.update(case=edit['case'], profile=profile, purpose=edit['purpose'],
                      source_hashes={n: reference.sha(reference.CORPUS / 'cases' / edit['case'] / n)
                                     for n in cases[edit['case']]['files']})
        report['results'].append(result)
        (out / 'report.json').write_text(json.dumps(report, indent=2) + '\n')
        print(name + ': ' + result['status'], flush=True)
    report['compiler_sha256_after_run'] = reference.sha(compiler)
    report['compiler_unchanged'] = compiler_hash == report['compiler_sha256_after_run']
    (out / 'report.json').write_text(json.dumps(report, indent=2) + '\n')
    return int(not report['compiler_unchanged'] or any(r['status'] != 'byte-exact' for r in report['results']))


if __name__ == '__main__':
    sys.exit(main())
