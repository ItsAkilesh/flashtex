#!/usr/bin/env python3
"""Build clean MacTeX oracles for exact recorded edits of positive fixtures."""
import argparse
import json
import os
from pathlib import Path
import sys
import tempfile

import incremental
import reference


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--only', action='append', required=True, metavar='CASE_ID')
    parser.add_argument('--output', type=Path, required=True)
    parser.add_argument('--texbin', type=Path, default=Path('/Library/TeX/texbin'))
    parser.add_argument('--timeout', type=int, default=30)
    parser.add_argument('--render', action='store_true')
    args = parser.parse_args()
    if args.timeout <= 0:
        parser.error('--timeout must be positive')
    manifest = json.loads((reference.CORPUS / 'manifest.json').read_text())
    reference.validate(manifest)
    cases = {c['id']: c for c in manifest['cases']}
    edits = json.loads((reference.CORPUS / 'edits.json').read_text())['edits']
    by_case = {e['case']: e for e in edits}
    unknown = set(args.only) - by_case.keys()
    if unknown:
        parser.error('unknown edit case: ' + ', '.join(sorted(unknown)))
    selected = list(dict.fromkeys(args.only))
    if any(cases[name]['expect'] != 'success' for name in selected):
        parser.error('negative-fixture repair needs an explicit edited expectation profile')
    prepared = [(cases[name], by_case[name], incremental.sequence(cases[name], by_case[name], 'legacy')[1])
                for name in selected]
    out = args.output.resolve()
    out.mkdir(parents=True, exist_ok=False)
    env = os.environ.copy()
    env.update(SOURCE_DATE_EPOCH='1789171200', FORCE_SOURCE_DATE='1', TZ='UTC',
               openin_any='p', openout_any='p')
    cache = out / 'tex-cache'
    cache.mkdir()
    env['TEXMFVAR'] = str(cache)
    env['TEXMFCACHE'] = str(cache)
    env['PATH'] = str(args.texbin) + os.pathsep + env.get('PATH', '')
    original_corpus = reference.CORPUS
    report = {'manifest_sha256': reference.sha(original_corpus / 'manifest.json'),
              'edits_sha256': reference.sha(original_corpus / 'edits.json'),
              'source_date_epoch': env['SOURCE_DATE_EPOCH'], 'auxiliary_state': 'clean',
              'results': []}
    with tempfile.TemporaryDirectory(prefix='flashtex-edited-sources-') as temporary:
        corpus = Path(temporary)
        for case, edit, request in prepared:
            directory = corpus / 'cases' / case['id']
            for document in request['payload']['documents']:
                path = directory / document['path']
                path.parent.mkdir(parents=True, exist_ok=True)
                path.write_bytes(document['text'].encode('utf-8'))
        try:
            reference.CORPUS = corpus
            for case, edit, request in prepared:
                result = reference.build(case, out, args, env)
                result['original_source_hashes'] = {
                    n: reference.sha(original_corpus / 'cases' / case['id'] / n) for n in case['files']}
                result['edit'] = edit
                result['auxiliary_state'] = 'clean'
                (out / case['id'] / 'reference.json').write_text(json.dumps(result, indent=2) + '\n')
                report['results'].append(result)
                (out / 'report.json').write_text(json.dumps(report, indent=2) + '\n')
                print(case['id'] + ': ' + result['status'], flush=True)
        finally:
            reference.CORPUS = original_corpus
    return int(any(r['status'] != 'reference-built'
                   or (args.render and r.get('raster_status') != 'rendered')
                   for r in report['results']))


if __name__ == '__main__':
    sys.exit(main())
