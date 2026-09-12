import copy
import importlib.util
import io
import json
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[1]
SPEC = importlib.util.spec_from_file_location('check_runtime', ROOT / 'scripts/check_runtime.py')
runtime = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(runtime)
REQUEST = json.loads((ROOT / 'protocol/fixtures/compile-request.json').read_text())
RESPONSE = json.loads((ROOT / 'protocol/fixtures/compile-result.json').read_text())


def pair(revision=1, project='demo'):
    request, response = copy.deepcopy(REQUEST), copy.deepcopy(RESPONSE)
    for value in (request, response):
        value['id'] = f'{project}-{revision}'
        value['payload']['revision'] = revision
        value['payload']['project_id'] = project
    return request, response


class RuntimeValidationTests(unittest.TestCase):
    def validate(self, messages):
        validator = runtime.Validator()
        stream = io.BytesIO(('\n'.join(json.dumps(message) for message in messages) + '\n').encode())
        return validator, runtime.validate_stream(stream, validator)

    def test_real_contract_fixtures(self):
        validator, errors = self.validate([REQUEST, RESPONSE])
        self.assertEqual(errors, [])
        self.assertEqual(validator.results[0]['preview'], 'current')

    def test_out_of_order_responses_are_valid_and_old_result_is_stale(self):
        request1, response1 = pair(1)
        request2, response2 = pair(2)
        validator, errors = self.validate([request1, request2, response2, response1])
        self.assertEqual(errors, [])
        self.assertEqual([r['preview'] for r in validator.results], ['current', 'stale_ignore'])

    def test_new_request_makes_old_response_stale_even_before_new_response(self):
        request1, response1 = pair(1)
        request2, _ = pair(2)
        validator, errors = self.validate([request1, request2, response1])
        self.assertEqual(errors, [])
        self.assertEqual(validator.results[0]['preview'], 'stale_ignore')

    def test_revision_order_is_scoped_to_project_and_same_revision_retry_allowed(self):
        request1, _ = pair(9)
        request2, _ = pair(1, 'other')
        retry = copy.deepcopy(request1)
        retry['id'] += '-retry'
        _, errors = self.validate([request1, request2, retry])
        self.assertEqual(errors, [])
        old_request, _ = pair(8)
        _, errors = self.validate([request1, old_request])
        self.assertIn('must not decrease', errors[0]['message'])

    def test_response_must_correlate_exact_request_not_latest(self):
        for field, value in [('revision', 2), ('project_id', 'other')]:
            response = copy.deepcopy(RESPONSE)
            response['payload'][field] = value
            _, errors = self.validate([REQUEST, response])
            self.assertIn('does not match', errors[0]['message'])
        _, errors = self.validate([RESPONSE])
        self.assertIn('no matching', errors[0]['message'])

    def test_utf8_offsets_not_codepoint_offsets(self):
        request, response = pair()
        request['payload']['documents'][0]['text'] = 'aé𝛼z'
        source = response['payload']['pages'][0]['items'][0]['source']
        for start, end, valid in [(1, 3, True), (3, 7, True), (8, 8, True),
                                  (2, 3, False), (3, 4, False), (0, 9, False),
                                  (-1, 1, False), (3, 1, False), (True, 3, False)]:
            source.update(start_byte=start, end_byte=end)
            _, errors = self.validate([request, response])
            self.assertEqual(not errors, valid, (start, end, errors))

    def test_paths_and_missing_document_are_rejected(self):
        for name in ['/etc/main.tex', '../main.tex', 'dir/../main.tex', 'C:\\main.tex', '\\\\host\\file', 'x\x00.tex']:
            request = copy.deepcopy(REQUEST)
            request['payload']['entry_path'] = name
            _, errors = self.validate([request])
            self.assertTrue(errors, name)
        response = copy.deepcopy(RESPONSE)
        response['payload']['pages'][0]['items'][0]['source']['path'] = 'missing.tex'
        _, errors = self.validate([REQUEST, response])
        self.assertIn('absent', errors[0]['message'])

    def test_malformed_payload_is_actionable_and_does_not_crash(self):
        mutations = [lambda p: p.update(pages={}), lambda p: p.update(diagnostics=None),
                     lambda p: p.update(status='perfect'), lambda p: p.update(revision=True),
                     lambda p: p['pages'][0].update(width_pt=-1),
                     lambda p: p['pages'][0]['items'][0].update(font_size_pt=0),
                     lambda p: p['pages'][0]['items'][0].update(source=[]),
                     lambda p: p['pages'][0]['items'][0].update(x_pt=float('nan')),
                     lambda p: p.update(pdf_path=3),
                     lambda p: p['pages'][0].update(width_pt=10 ** 1000),
                     lambda p: p['pages'][0]['items'][0].update(text='\ud800')]
        for mutate in mutations:
            response = copy.deepcopy(RESPONSE)
            mutate(response['payload'])
            _, errors = self.validate([REQUEST, response])
            self.assertTrue(errors)
            self.assertEqual(errors[0]['line'], 2)

    def test_recovery_diagnostic_source_uses_same_utf8_checks(self):
        request, response = pair()
        response['payload']['status'] = 'recovered'
        response['payload']['diagnostics'] = [{'severity': 'error', 'message': 'Unknown command',
            'source': None, 'recovery': 'Rendered literal text'}]
        _, errors = self.validate([request, response])
        self.assertEqual(errors, [])
        response['payload']['diagnostics'][0]['source'] = {'path': 'main.tex', 'start_byte': 0, 'end_byte': 999}
        _, errors = self.validate([request, response])
        self.assertIn('byte range', errors[0]['message'])

    def test_duplicate_requests_responses_and_json_keys(self):
        for messages in [[REQUEST, REQUEST], [REQUEST, RESPONSE, RESPONSE]]:
            _, errors = self.validate(messages)
            self.assertIn('duplicate', errors[0]['message'])
        errors = runtime.validate_stream(io.BytesIO(b'{"id":"a","id":"b"}\n'), runtime.Validator())
        self.assertIn('duplicate JSON key', errors[0]['message'])

    def test_oversized_and_invalid_utf8_records_recover_for_next_line(self):
        validator = runtime.Validator()
        raw = b'x' * 2000 + b'\n' + b'\xff\n' + json.dumps(REQUEST).encode() + b'\n'
        errors = runtime.validate_stream(io.BytesIO(raw), validator, max_line_bytes=1000)
        self.assertEqual(len(errors), 2)
        self.assertEqual(errors[1]['line'], 2)
        self.assertIn(REQUEST['id'], validator.requests)

    def test_cli_subprocess_input_and_pending_policy(self):
        command = [sys.executable, str(ROOT / 'scripts/check_runtime.py'), '-']
        transcript = json.dumps(REQUEST) + '\n' + json.dumps(RESPONSE) + '\n'
        result = subprocess.run(command, input=transcript, text=True, capture_output=True)
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertTrue(json.loads(result.stdout)['valid'])
        for flags, code in [([], 1), (['--allow-pending'], 0)]:
            result = subprocess.run(command + flags, input=json.dumps(REQUEST) + '\n', text=True, capture_output=True)
            self.assertEqual(result.returncode, code)
            self.assertEqual(json.loads(result.stdout)['pending_request_ids'], [REQUEST['id']])

    def test_cli_separate_request_and_response_files(self):
        with tempfile.TemporaryDirectory() as temp:
            request_file = Path(temp) / 'requests.jsonl'
            request_file.write_text(json.dumps(REQUEST) + '\n')
            result = subprocess.run([sys.executable, str(ROOT / 'scripts/check_runtime.py'),
                                     '--requests', str(request_file), '-'],
                                    input=json.dumps(RESPONSE) + '\n', text=True, capture_output=True)
        self.assertEqual(result.returncode, 0, result.stdout + result.stderr)

    def test_error_terminal_explicitly_reports_unspecified_schema(self):
        response = {'protocol_version': 1, 'id': REQUEST['id'], 'type': 'error', 'payload': {}}
        validator, errors = self.validate([REQUEST, response])
        self.assertEqual(errors, [])
        self.assertEqual(validator.results[0]['preview'], 'unchanged')
        self.assertIn('not defined', validator.results[0]['note'])


if __name__ == '__main__':
    unittest.main()
