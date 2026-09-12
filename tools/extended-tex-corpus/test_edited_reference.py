import json
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest

import incremental
import reference


class EditedReferenceTests(unittest.TestCase):
    def test_edited_sources_pdfs_and_original_provenance_match(self):
        index = json.loads((reference.CORPUS / 'edited-reference-index.json').read_text())
        self.assertEqual(index['manifest_sha256'], reference.sha(reference.CORPUS / 'manifest.json'))
        self.assertEqual(index['edits_sha256'], reference.sha(reference.CORPUS / 'edits.json'))
        self.assertEqual(index['auxiliary_state'], 'clean')
        cases = {c['id']: c for c in json.loads((reference.CORPUS / 'manifest.json').read_text())['cases']}
        edits = {e['case']: e for e in json.loads((reference.CORPUS / 'edits.json').read_text())['edits']}
        self.assertEqual({r['id'] for r in index['results']}, {n for n in edits if n.startswith('hw1-')})
        for result in index['results']:
            name = result['id']
            self.assertEqual(result['edit'], edits[name])
            self.assertEqual(result['status'], 'reference-built')
            self.assertEqual(result['raster_status'], 'rendered')
            self.assertEqual(result['warnings'], [])
            self.assertEqual(len(result['pages']), 1)
            directory = reference.CORPUS / 'edited-references' / name
            request = incremental.sequence(cases[name], edits[name], 'legacy')[1]
            for document in request['payload']['documents']:
                path = document['path']
                self.assertEqual((directory / path).read_bytes(), document['text'].encode('utf-8'))
                self.assertEqual(reference.sha(directory / path), result['source_hashes'][path])
                self.assertEqual(reference.sha(reference.CORPUS / 'cases' / name / path),
                                 result['original_source_hashes'][path])
            self.assertEqual(reference.sha(directory / 'main.pdf'), result['pdf_sha256'])
            self.assertEqual(json.loads((directory / 'reference.json').read_text()),
                             {k: v for k, v in result.items()})

    def test_unknown_or_negative_profile_is_rejected_before_output_or_tex(self):
        for name, message in [('not-a-case', b'unknown edit case'),
                              ('error-double-superscript', b'explicit edited expectation profile')]:
            with self.subTest(case=name), tempfile.TemporaryDirectory() as directory:
                out = Path(directory) / 'must-not-exist'
                proc = subprocess.run([sys.executable, str(Path(__file__).with_name('edited_reference.py')),
                                       '--only', name, '--output', str(out)], capture_output=True, timeout=5)
                self.assertNotEqual(proc.returncode, 0)
                self.assertIn(message, proc.stderr)
                self.assertEqual(proc.stdout, b'')
                self.assertFalse(out.exists())


if __name__ == '__main__':
    unittest.main()
