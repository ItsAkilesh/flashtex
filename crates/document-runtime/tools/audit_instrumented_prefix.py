#!/usr/bin/env python3
"""Audit pinned failed-run artifacts only; never execute helper or producer."""
import base64
import collections
import gzip
import hashlib
import json
import pathlib
import subprocess

COMMIT = subprocess.check_output(['git', 'rev-parse', '9f4a3105'], text=True).strip()
BASE = 'crates/preview-controller/benchmarks/typing-burst-instrumented-failed/'
def blob(path):
    return subprocess.check_output(['git', 'show', COMMIT + ':' + BASE + path])
def sha(raw):
    return hashlib.sha256(raw).hexdigest()
archive = json.loads(blob('archive.json'))
data = {}
for name, entry in archive.items():
    compressed = blob(entry['path'])
    assert sha(compressed) == entry['sha256']
    raw = gzip.decompress(compressed)
    assert sha(raw) == entry['uncompressed_sha256']
    data[name] = raw
failure = json.loads(blob('failure.json'))
assert failure['status'] == 'failed_post_measurement_reopen' and failure['exit_code'] == 1
assert not {'provenance.json', 'clean.input.jsonl', 'clean.output.jsonl'} & data.keys()
for name, expected in failure['scripts'].items():
    actual = subprocess.check_output(['git', 'show', COMMIT + ':crates/preview-controller/examples/' + name])
    assert sha(actual) == expected
commands = list(map(json.loads, data['requests.jsonl'].splitlines()))
assert len(commands) == 20
config = json.loads(data['sender/sender-config.json'])
assert b''.join(base64.b64decode(s, validate=True) for s in config['frames']) == data['requests.jsonl']
states = [data['initial.tex']]
for i, request in enumerate(commands):
    command = request['payload']['command']
    assert command['expected_revision'] == i + 1
    assert command['expected_sha256'] == sha(states[-1])
    assert command['command_id'] == f'typing-{i}'
    edit, = command['edits']
    start, end = edit['start_byte'], edit['end_byte']
    assert states[-1][start:end].decode() == edit['removed_text']
    states.append(states[-1][:start] + edit['replacement'].encode() + states[-1][end:])
    assert len(states[-1]) == 50000
assert states[-1] == data['final.tex']
lines = data['events.jsonl'].splitlines(keepends=True)
events = list(map(json.loads, lines))
histories = []
acks = []
current = []
for event in events:
    p = event.get('payload', {})
    if (event.get('id') or '').startswith('edit-'):
        i = int(event['id'][5:]); h = p['history']
        assert h['document']['revision'] == h['command_revision'] == i + 2
        assert h['document']['source_sha256'] == sha(states[i + 1])
        acks.append(i)
    if p.get('kind') == 'completed_snapshot':
        histories.append(p)
    if p.get('kind') == 'preview':
        assert p['source_versions']['main.tex'] >= max(acks) + 2
        current.append(p)
assert acks == list(range(20)) and len(current) == 1
assert current[0]['source_versions']['main.tex'] == 21
assert current[0] == json.loads(data['final-preview.json'])
pairs = []
partial = []
for name in data:
    if not name.startswith('producer/') or not name.endswith('.input.jsonl'):
        continue
    requests = list(map(json.loads, data[name].splitlines()))
    output = data[name.replace('.input.', '.output.')]
    if not output.endswith(b'\n'):
        assert len(requests) == 1 and requests[0]['payload']['documents'][0]['text'].encode() == states[-1]
        assert b'\n' not in output and len(output) == failure['reopen_output_captured_bytes'] == 94208
        partial.append({'path': name.replace('.input.', '.output.'), 'bytes': len(output), 'sha256': sha(output), 'complete_jsonl_frames': 0})
        continue
    replies = list(map(json.loads, output.splitlines()))
    assert len(requests) == len(replies)
    for request, reply in zip(requests, replies):
        assert request['id'] == reply['id']
        revision = request['payload']['revision']
        assert reply['payload']['revision'] == revision
        expected = states[min(revision, 21) - 1]
        assert request['payload']['documents'] == [{'path': 'main.tex', 'text': expected.decode()}]
        pairs.append((request, reply))
assert len(partial) == 1
for h in histories + current:
    matched = [(q, r) for q, r in pairs if q['id'] == h['request_id'] and r == h['result']]
    assert len(matched) == 1
    if h in histories:
        revision = h['source_versions']['main.tex']
        assert h['is_current'] is False and h['source_actions_enabled'] is False
        assert h['source_binding_token'] == f'burst-{revision - 2}'
        assert h['compile_revision'] == matched[0][0]['payload']['revision'] < h['current_compile_revision']
        assert matched[0][0]['payload']['documents'][0]['text'].encode() == states[revision - 1]
receiver = json.loads(data['receiver-timings.json'])
assert receiver['dropped'] == 0
records = receiver['records']
diag = json.loads(data['diagnostics.json'])
frames = collections.defaultdict(list)
for r in diag:
    if r['phase'] == 'output_frame':
        assert set(r) == {'phase', 'sequence', 'compile_revision', 'class', 'bytes', 'admission_attempt_ms', 'at_ms', 'outcome'}
        frames[r['sequence']].append(r)
assert len(frames) == len(records) == 49
ordered = []
for sequence, lifecycle in frames.items():
    assert collections.Counter(r['outcome'] for r in lifecycle) == collections.Counter(['admitted', 'dequeued', 'write_started', 'write_finished'])
    for key in ['sequence', 'compile_revision', 'class', 'bytes', 'admission_attempt_ms']:
        assert all(r[key] == lifecycle[0][key] for r in lifecycle)
    by = {r['outcome']: r for r in lifecycle}
    assert by['dequeued']['at_ms'] <= by['write_started']['at_ms'] <= by['write_finished']['at_ms']
    ordered.append(by['write_started'])
ordered.sort(key=lambda r: r['at_ms'])
for i, (frame, record) in enumerate(zip(ordered, records), 1):
    assert record['sequence'] == i and record['bytes'] == frame['bytes']
    assert record['read_started'] <= record['frame_received'] <= record['decode_started'] <= record['decode_finished']
prefix = len(records) - len(lines)
assert prefix == 3
for line, event, frame in zip(lines, events, ordered[prefix:]):
    assert len(line) == frame['bytes']
    if event.get('payload', {}).get('kind') == 'completed_snapshot':
        assert frame['class'] == 'optional'
        assert frame['compile_revision'] == event['payload']['compile_revision']
assert sum(r['class'] == 'optional' for r in ordered) == len(histories) == 10
summary = {'capture_commit': COMMIT, 'capture_path': BASE, 'artifact_hashes_verified': len(data), 'overall_status': failure['status'], 'source_edits_and_acks': 20, 'current_revisions': [21], 'historical_revisions': [h['compile_revision'] for h in histories], 'matched_writer_receiver_frames': len(records), 'telemetry_dropped': receiver['dropped'], 'startup_prefix_frames': prefix, 'reopen_partial': partial, 'missing_acceptance': failure['missing_acceptance'], 'qualification': 'Artifact-only prefix audit. Original complete producer results match retained source; failed reopen, absent clean comparison and normal provenance remain unverified. No cross-clock subtraction, native claim, rerun or failure-cause inference.'}
out = pathlib.Path('crates/document-runtime/benchmarks/instrumented-prefix-review')
out.mkdir(exist_ok=True)
(out / 'audit.json').write_text(json.dumps(summary, indent=2) + '\n')
print(json.dumps(summary, indent=2))
