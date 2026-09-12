#!/usr/bin/env python3
"""Actual producer -> helper display replay. No network, native paint or oracle claim."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import subprocess
import tempfile
from helper_replay import Client, snapshot_after_initial_preview


def digest(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def main():
    parser = argparse.ArgumentParser()
    for name in ('helper', 'producer', 'producer-evidence', 'fixture', 'output'):
        parser.add_argument('--' + name, required=True)
    args = parser.parse_args()
    expected = json.loads(Path(args.producer_evidence).read_text())
    assert digest(args.producer) == expected['binary_sha256'], 'producer binary drift'
    for asset in expected['assets']:
        assert digest(asset['path']) == asset['sha256'], 'asset drift: ' + asset['path']
    font_dirs = sorted({str(Path(a['path']).parent) for a in expected['assets'] if a['path'].endswith('.otf')})
    tfm_dirs = sorted({str(Path(a['path']).parent) for a in expected['assets'] if a['path'].endswith('.tfm')})
    assert font_dirs and tfm_dirs
    os.environ['FLASHTEX_FONT_DIRS'] = os.pathsep.join(font_dirs)
    os.environ['FLASHTEX_TFM_DIRS'] = os.pathsep.join(tfm_dirs)
    os.environ.pop('FLASHTEX_MAX_REPLY_BYTES', None)
    fixture = json.loads(Path(args.fixture).read_text())
    source = fixture['payload']['documents'][0]['text']
    output = Path(args.output)
    output.mkdir(parents=True, exist_ok=False)
    cases = []
    with tempfile.TemporaryDirectory() as temporary:
        root = Path(temporary)
        (root / 'project').mkdir()
        (root / 'private').mkdir()
        (root / 'project/main.tex').write_text(source)
        config = root / 'config.json'
        config.write_text(json.dumps(dict(session_id='benchmark', project_id='p', entry_path='main.tex',
            project_root=str(root/'project'), private_ledger_root=str(root/'private'),
            compiler_path=str(Path(args.producer).resolve()), diagnostic_timings=True)))
        client = Client(args.helper, config, capture_diagnostics=True)
        try:
            document = snapshot_after_initial_preview(client)
            assert document['text'] == source
            for step in range(3):
                if step == 0:
                    client.send('enable', 'configure_display_candidates', dict(capability='display-candidates-v1',
                        enabled=True, renderer_support_confirmed=True))
                    reply_id = 'enable'
                else:
                    source = source.replace('Office AV fi.', 'Office AV fi. More text.') if step == 1 else source.replace('More text.', 'More text. Final edit.')
                    reply_id = 'edit-' + str(step)
                    client.send(reply_id, 'edit', dict(path='main.tex', expected_revision=document['revision'],
                        expected_sha256=document['source_sha256'], text=source))
                events = []
                previews = {}
                acknowledged = False
                while True:
                    event, _ = client.read()
                    events.append(event)
                    payload = event.get('payload', {})
                    assert event['type'] != 'error', event
                    assert payload.get('kind') != 'failed', event
                    if event.get('id') == reply_id:
                        assert event['type'] == 'result'
                        acknowledged = True
                        if step:
                            document = payload['document']
                            assert document['text'] == source
                    if payload.get('kind') == 'preview':
                        previews[payload['request_id']] = payload['result']
                    if payload.get('kind') == 'display_candidate':
                        assert acknowledged
                        candidate = payload
                        assert candidate['untrusted'] and not candidate['source_actions_enabled']
                        assert candidate['source_versions'] == {'main.tex': document['revision']}
                        assert candidate['request_id'] in previews, 'candidate preceded matching v1'
                        break
                request = dict(protocol_version=1, type='compile', id=candidate['request_id'], payload=dict(
                    project_id='p', revision=candidate['compile_revision'], entry_path='main.tex',
                    documents=[dict(path='main.tex', text=source)], layout_capabilities=['display-list-v2']))
                direct = subprocess.run([args.producer], input=json.dumps(request)+'\n', text=True,
                    capture_output=True, check=True, timeout=20)
                lines = [json.loads(line) for line in direct.stdout.splitlines()]
                assert len(lines) == 2
                assert lines[0] == previews[candidate['request_id']], 'v1 output changed through helper'
                assert lines[1] == candidate['display_list'], 'display output changed through helper'
                binding = lines[1]['payload']['documents'][0]
                assert binding['sha256'] == hashlib.sha256(source.encode()).hexdigest()
                assert binding['byte_length'] == len(source.encode())
                assert not lines[0]['payload']['diagnostics'], 'actual producer diagnostics'
                (output/f'step-{step}.json').write_text(json.dumps(dict(request=request, events=events, direct=lines), indent=2)+'\n')
                cases.append(dict(step=step, source_revision=document['revision'], compile_revision=candidate['compile_revision'],
                    exact_direct_v1=True, exact_direct_display=True, source_sha256=binding['sha256']))
            diagnostics = client.diagnostics()
        finally:
            client.stop()
        client = Client(args.helper, config)
        try:
            recovered = snapshot_after_initial_preview(client)
            assert recovered == document, 'durable source changed on reopen'
        finally:
            client.stop()
    evidence = dict(cases=cases, exact_reopen=True, helper_sha256=digest(args.helper),
        producer_sha256=digest(args.producer), producer_source_sha=expected['producer_sha'],
        producer_evidence_sha256=digest(args.producer_evidence), fixture_sha256=digest(args.fixture),
        assets=expected['assets'], diagnostics=diagnostics,
        native_rendering='not performed', native_latency='not measured')
    repo = Path(__file__).resolve().parents[3]
    evidence['helper_source_sha'] = subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=repo, text=True).strip()
    evidence['replay_script_sha256'] = digest(__file__)
    evidence['step_artifact_sha256'] = {p.name: digest(p) for p in sorted(output.glob('step-*.json'))}
    (output/'provenance.json').write_text(json.dumps(evidence, indent=2)+'\n')
    print(json.dumps(cases))


if __name__ == '__main__':
    main()
