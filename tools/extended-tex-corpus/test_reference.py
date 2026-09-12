import importlib.util
import json
from pathlib import Path
import subprocess
import sys
import unittest

SPEC=importlib.util.spec_from_file_location('reference', Path(__file__).with_name('reference.py'))
R=importlib.util.module_from_spec(SPEC); SPEC.loader.exec_module(R)

class CorpusTests(unittest.TestCase):
    def test_integrity_and_engine_profiles(self):
        m=json.loads((R.CORPUS/'manifest.json').read_text()); R.validate(m)
        self.assertEqual({c['engine'] for c in m['cases']},{'pdftex','pdflatex','lualatex','xelatex'})
        self.assertTrue(all(c['diagnostic'] for c in m['cases'] if c['expect']=='error'))

    def test_multifile_request_preserves_sources(self):
        p=subprocess.run([sys.executable,str(Path(R.__file__)),'--emit-request','project-input-package'],capture_output=True,text=True,check=True)
        request=json.loads(p.stdout)
        docs={d['path']:d['text'] for d in request['payload']['documents']}
        self.assertIn('localstyle.sty',docs)
        self.assertIn('chapters/two.tex',docs)
        for name,source in docs.items():
            self.assertEqual(source,(R.CORPUS/'cases/project-input-package'/name).read_text())
        self.assertEqual(request['payload']['entry_path'],'main.tex')

    def test_reference_hashes_match_sources_and_pdfs(self):
        index=json.loads((R.CORPUS/'reference-index.json').read_text())
        self.assertEqual(index['manifest_sha256'],R.sha(R.CORPUS/'manifest.json'))
        manifest=json.loads((R.CORPUS/'manifest.json').read_text())
        self.assertEqual({c['id'] for c in manifest['cases']},{r['id'] for r in index['results']})
        for result in index['results']:
            for name,digest in result['source_hashes'].items():
                self.assertEqual(R.sha(R.CORPUS/'cases'/result['id']/name),digest)
            if result['status']=='reference-built':
                self.assertEqual(R.sha(R.CORPUS/'references'/result['id']/'main.pdf'),result['pdf_sha256'])

    def test_edit_byte_ranges_and_unique_replacements(self):
        edits=json.loads((R.CORPUS/'edits.json').read_text())['edits']
        for edit in edits:
            data=(R.CORPUS/'cases'/edit['case']/edit['path']).read_bytes()
            self.assertEqual(data.count(edit['old'].encode()),1)
            self.assertEqual(data[edit['start_byte']:edit['end_byte']],edit['old'].encode())
            changed=data[:edit['start_byte']]+edit['new'].encode()+data[edit['end_byte']:]
            changed.decode('utf-8')
            self.assertNotEqual(data,changed)

    def test_binary_asset_request_rejected_explicitly(self):
        p=subprocess.run([sys.executable,str(Path(R.__file__)),'--emit-request','graphics-included-raster'],capture_output=True)
        self.assertNotEqual(p.returncode,0)
        self.assertIn(b'binary assets require a project-files adapter',p.stderr)
        self.assertEqual(p.stdout,b'')

    def test_unknown_case_rejected(self):
        p=subprocess.run([sys.executable,str(Path(R.__file__)),'--emit-request','nonexistent'],capture_output=True)
        self.assertNotEqual(p.returncode,0)
        self.assertEqual(p.stdout,b'')

    def test_bounded_child_timeout(self):
        result=R.run([sys.executable,'-c','import time; time.sleep(5)'],Path.cwd(),{},0.05)
        self.assertTrue(result['timeout'])
        self.assertIsNone(result['returncode'])

if __name__=='__main__': unittest.main()
