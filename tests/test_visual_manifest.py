"""Synthetic artifacts test validation, never reference-engine/font correctness."""
import copy
import hashlib
import importlib.util
import json
from pathlib import Path
import tempfile
import unittest

ROOT=Path(__file__).resolve().parents[1]
SPEC=importlib.util.spec_from_file_location('visual_manifest',ROOT/'scripts/check_visual_manifest.py')
checker=importlib.util.module_from_spec(SPEC);SPEC.loader.exec_module(checker)
PIL_AVAILABLE=importlib.util.find_spec('PIL') is not None


class VisualManifestTests(unittest.TestCase):
    def setUp(self):
        self.temp=tempfile.TemporaryDirectory();self.addCleanup(self.temp.cleanup);self.root=Path(self.temp.name)
        self.manifest=json.loads((ROOT/'tests/visual-corpus/fixtures/manifest.json').read_text())
        self.manifest['cases']=self.manifest['cases'][:1];self.case=self.manifest['cases'][0]
        source=ROOT/'tests/visual-corpus/fixtures'/self.case['entry_path']
        (self.root/source.name).write_bytes(source.read_bytes());self.path=self.root/'manifest.json'

    def run_check(self):
        self.path.write_text(json.dumps(self.manifest));return checker.check_manifest(self.path)

    def artifact(self,name,data):
        (self.root/name).write_bytes(data);return {'path':name,'sha256':hashlib.sha256(data).hexdigest()}

    def generated(self):
        from PIL import Image
        profile=self.manifest['profiles'][self.case['profile']]
        image=Image.new('RGB',tuple(profile['expected_page_pixels']),'white');image.save(self.root/'page.png')
        page=self.artifact('page.png',(self.root/'page.png').read_bytes())
        page.update(number=1,width=image.width,height=image.height,pixel_sha256=hashlib.sha256(image.convert('RGBA').tobytes()).hexdigest())
        font=self.artifact('font.pfb',b'synthetic font bytes; not an actual TeX font');font['kind']='font_program'
        metric=self.artifact('font.tfm',b'synthetic metric bytes');metric['kind']='metrics'
        self.case['reference_state']='generated'
        self.case['reference']={'source_sha256':self.case['source_sha256'],'engine':profile['reference_engine'],
           'engine_version':'Synthetic validator test only','format_date':'2026-01-01','generation_host':'synthetic-test',
           'generation_time_utc':'2026-09-12T05:00:00Z','package_versions':{p:'test-version' for p in profile['required_packages']},
           'fonts':[{'name':'synthetic','files':[font,metric]}],
           'render':{'dpi':profile['dpi'],'media_box_bp':profile['page_media_box_bp'],'page_pixels':profile['expected_page_pixels'],
                     'color_space':'RGB','background':'white','color_profile':{'mode':'unmanaged','description':'synthetic unmanaged RGB'},
                     'argv_template':profile['raster_argv']},
           'rasterizer':{'name':profile['rasterizer'],'version':'synthetic-test','binary_sha256':'a'*64},
           'pdf':self.artifact('reference.pdf',b'%PDF-1.4\nsynthetic artifact header only\n'),'pages':[page]}

    def test_actual_sources_valid_but_references_pending(self):
        result=checker.check_manifest(ROOT/'tests/visual-corpus/fixtures/manifest.json')
        self.assertEqual(result['status'],'valid');self.assertEqual(result['reference_status'],'pending')
        self.assertEqual(len(result['cases']),8);self.assertIn('pending',result['visual_fidelity'])

    def test_source_hash_drift_invalid(self):
        (self.root/self.case['entry_path']).write_text('changed')
        self.assertEqual(self.run_check()['status'],'invalid')

    def test_relaxed_tolerance_rejected(self):
        self.manifest['profiles'][self.case['profile']]['comparison']['required_max_channel_delta']=1
        with self.assertRaisesRegex(ValueError,'zero tolerance'):self.run_check()

    def test_dpi_geometry_disagreement_rejected(self):
        self.manifest['profiles'][self.case['profile']]['dpi']=72
        with self.assertRaisesRegex(ValueError,'disagree'):self.run_check()

    def test_path_escape_rejected(self):
        self.case['entry_path']='../elsewhere.tex'
        self.assertEqual(self.run_check()['status'],'invalid')

    def test_generated_record_missing_remains_pending(self):
        self.case['reference_state']='generated'
        result=self.run_check();self.assertEqual(result['status'],'valid');self.assertEqual(result['reference_status'],'pending')

    @unittest.skipUnless(PIL_AVAILABLE,'Pillow required to create synthetic PNG')
    def test_artifact_verification_never_claims_fidelity(self):
        self.generated();result=self.run_check()
        self.assertEqual(result['reference_status'],'verified');self.assertIn('pending',result['visual_fidelity'])
        self.assertIn('not independently attested',result['cases'][0]['reference']['scope'])

    @unittest.skipUnless(PIL_AVAILABLE,'Pillow required')
    def test_missing_reference_file_is_pending(self):
        self.generated();(self.root/'page.png').unlink();result=self.run_check()
        self.assertEqual(result['status'],'valid');self.assertEqual(result['reference_status'],'pending')

    @unittest.skipUnless(PIL_AVAILABLE,'Pillow required')
    def test_font_hash_mismatch_invalid(self):
        self.generated();(self.root/'font.pfb').write_bytes(b'changed')
        self.assertEqual(self.run_check()['status'],'invalid')

    @unittest.skipUnless(PIL_AVAILABLE,'Pillow required')
    def test_pixel_hash_mismatch_invalid(self):
        self.generated();self.case['reference']['pages'][0]['pixel_sha256']='0'*64
        self.assertEqual(self.run_check()['status'],'invalid')

    @unittest.skipUnless(PIL_AVAILABLE,'Pillow required')
    def test_stale_source_binding_invalid(self):
        self.generated();self.case['reference']['source_sha256']='0'*64
        self.assertEqual(self.run_check()['status'],'invalid')

    @unittest.skipUnless(PIL_AVAILABLE,'Pillow required')
    def test_unmeasured_engine_version_invalid(self):
        self.generated();self.case['reference']['engine_version']='pending'
        self.assertEqual(self.run_check()['status'],'invalid')

    @unittest.skipUnless(PIL_AVAILABLE,'Pillow required')
    def test_undeclared_png_color_profile_invalid(self):
        from PIL import Image
        self.generated()
        path=self.root/'page.png'
        with Image.open(path) as image:
            image.save(self.root/'profiled.png',icc_profile=b'synthetic test profile')
        data=(self.root/'profiled.png').read_bytes();path.write_bytes(data)
        self.case['reference']['pages'][0]['sha256']=hashlib.sha256(data).hexdigest()
        self.assertEqual(self.run_check()['status'],'invalid')

    def test_schema_pins_exact_pixel_contract(self):
        schema=json.loads((ROOT/'tests/visual-corpus/manifest.schema.json').read_text())
        comparison=schema['$defs']['profile']['properties']['comparison']['properties']
        self.assertEqual(comparison['required_max_channel_delta']['const'],0)
        self.assertEqual(comparison['required_differing_pixel_count']['const'],0)
        self.assertIn('RGBA8',schema['$defs']['page']['properties']['pixel_sha256']['description'])


if __name__=='__main__':unittest.main()
