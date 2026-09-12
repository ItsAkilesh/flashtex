import json
from pathlib import Path
import sys
import tempfile
import unittest
from unittest.mock import patch

import incremental as I


class IncrementalGateTests(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.root = Path(self.tmp.name)
        cases = json.loads((I.reference.CORPUS / 'manifest.json').read_text())['cases']
        edits = json.loads((I.reference.CORPUS / 'edits.json').read_text())['edits']
        self.edit = next(e for e in edits if e['case'] == 'project-input-package')
        self.case = next(c for c in cases if c['id'] == self.edit['case'])
        self.requests = I.sequence(self.case, self.edit, 'typed')

    def tearDown(self):
        self.tmp.cleanup()

    def compiler(self, mode):
        path = self.root / ('fake-' + mode)
        path.write_text('#!' + sys.executable + '\n' + '''import json, sys, time
mode = ''' + repr(mode) + '''
for index, line in enumerate(sys.stdin):
    request = json.loads(line)
    if mode == 'timeout': time.sleep(5)
    if mode == 'crash': sys.exit(7)
    if mode == 'drop': continue
    if mode == 'malformed':
        print('not JSON', flush=True)
        continue
    reply = {'protocol_version': 1, 'type': 'compile_result', 'id': request['id'],
             'payload': {'revision': request['payload']['revision'], 'pages': [],
                         'status': 'recovered'}}
    if mode == 'stale' and index > 0: reply['payload']['pages'] = ['stale cached page']
    if mode == 'correlation': reply['id'] = 'different capture'
    print(json.dumps(reply), flush=True)
''')
        path.chmod(0o755)
        return path

    def test_consistent_recovered_reply_passes_without_claiming_support(self):
        result = I.check(self.compiler('good'), self.requests, self.root / 'accept', 2)
        self.assertEqual(result['status'], 'byte-exact')
        self.assertEqual([c['state'] for c in result['comparisons']], ['edited', 'undo', 'reapplied'])
        self.assertTrue(all(c['equal'] for c in result['comparisons']))
        self.assertEqual(len((self.root / 'accept/warm/stdout.jsonl').read_bytes().splitlines()), 4)

    def test_stale_warm_reply_fails_and_preserves_full_evidence(self):
        result = I.check(self.compiler('stale'), self.requests, self.root / 'stale', 2)
        self.assertEqual(result['status'], 'failed')
        self.assertTrue(all(c['difference'] for c in result['comparisons']))
        warm = (self.root / 'stale/warm/stdout.jsonl').read_bytes().splitlines(keepends=True)[1]
        clean = (self.root / 'stale/clean-edited/stdout.jsonl').read_bytes()
        self.assertEqual(result['comparisons'][0]['difference']['warm_sha256'], I.digest(warm))
        self.assertEqual(result['comparisons'][0]['difference']['clean_sha256'], I.digest(clean))

    def test_bad_transport_never_passes_even_if_warm_and_clean_are_identical(self):
        for mode in ('correlation', 'drop', 'malformed', 'crash'):
            with self.subTest(mode=mode):
                result = I.check(self.compiler(mode), self.requests, self.root / mode, 2)
                self.assertEqual(result['status'], 'failed')
                self.assertTrue(result['warm']['problems'])
                self.assertTrue(all(not c['equal'] for c in result['comparisons']))

    def test_timeout_is_bounded_and_retains_input(self):
        result, lines = I.execute(self.compiler('timeout'), self.requests, self.root / 'timed', 0.05)
        self.assertTrue(result['timeout'])
        self.assertIsNone(result['returncode'])
        self.assertEqual(lines, [])
        self.assertEqual(len((self.root / 'timed/requests.jsonl').read_bytes().splitlines()), 4)

    def test_sequence_preserves_included_files_and_undo_bytes(self):
        for request in self.requests:
            self.assertEqual([d['path'] for d in request['payload']['documents']], self.case['files'])
        snapshots = [{d['path']: d['text'] for d in r['payload']['documents']} for r in self.requests]
        self.assertEqual(snapshots[0], snapshots[2])
        self.assertEqual(snapshots[1], snapshots[3])
        self.assertIn('CHANGED CHILD', snapshots[1][self.edit['path']])
        for path in self.case['files']:
            if path != self.edit['path']:
                self.assertTrue(all(s[path] == snapshots[0][path] for s in snapshots))

    def test_utf8_byte_offset_and_whitespace_difference_are_not_normalized(self):
        directory = self.root / 'cases/unicode'
        directory.mkdir(parents=True)
        (directory / 'main.tex').write_text('α before OLD after\n')
        case = {'id': 'unicode', 'entry': 'main.tex', 'files': ['main.tex']}
        edit = {'path': 'main.tex', 'old': 'OLD', 'new': 'β NEW', 'start_byte': 10, 'end_byte': 13}
        with patch.object(I.reference, 'CORPUS', self.root):
            requests = I.sequence(case, edit, 'legacy')
            self.assertEqual(requests[1]['payload']['documents'][0]['text'], 'α before β NEW after\n')
            self.assertNotIn('layout_capabilities', requests[0]['payload'])
            edit['start_byte'] = 9
            with self.assertRaises(ValueError):
                I.sequence(case, edit, 'legacy')
        self.assertEqual(I.difference(b'{"a":1}\n', b'{"a": 1}\n')['first_different_byte'], 5)


if __name__ == '__main__':
    unittest.main()
