import copy
import base64
import struct
import zlib
import importlib.util
import io
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest
from unittest import mock
import ast
import time

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

    def test_line_limit_excludes_lf_and_accepts_complete_final_record_at_eof(self):
        record = json.dumps(REQUEST).encode()
        for suffix in (b'', b'\n'):
            validator = runtime.Validator()
            errors = runtime.validate_stream(io.BytesIO(record + suffix), validator,
                                             max_line_bytes=len(record))
            self.assertEqual(errors, [])
            self.assertIn(REQUEST['id'], validator.requests)
        errors = runtime.validate_stream(io.BytesIO(record + b' \n'), runtime.Validator(),
                                         max_line_bytes=len(record))
        self.assertIn('exceeds', errors[0]['message'])

    def test_truncated_eof_record_is_reported_not_silently_dropped(self):
        errors = runtime.validate_stream(io.BytesIO(b'{"protocol_version":1,'), runtime.Validator())
        self.assertEqual(len(errors), 1)
        self.assertEqual(errors[0]['line'], 1)
        errors = runtime.validate_stream(io.BytesIO(b'x' * 31), runtime.Validator(), max_line_bytes=10)
        self.assertEqual(len(errors), 1)
        self.assertIn('exceeds', errors[0]['message'])

    @unittest.skipUnless(os.environ.get('FLASHTEX_COMPILER_BIN'),
                         'set FLASHTEX_COMPILER_BIN to a reviewed built compiler executable')
    def test_real_compiler_transcript_utf8_recovery_and_final_eof(self):
        # Optional integration gate: no Cargo download/build or peer modification.
        # The executable must already be built from the peer revision under review.
        requests = []
        for revision, text in enumerate(('Hello FlashTeX.', 'aé𝛼z',
                                         r'Good \unknown{oops}.'), 1):
            request, _ = pair(revision)
            request['payload']['documents'][0]['text'] = text
            requests.append(request)
        data = '\n'.join(json.dumps(request, ensure_ascii=False) for request in requests)
        result = subprocess.run([os.environ['FLASHTEX_COMPILER_BIN']], input=data.encode('utf-8'),
                                capture_output=True, timeout=10)
        self.assertEqual(result.returncode, 0, result.stderr)
        replies = [json.loads(line) for line in result.stdout.splitlines()]
        self.assertEqual(len(replies), len(requests))
        validator, errors = self.validate(requests + replies)
        self.assertEqual(errors, [])
        self.assertEqual([r['preview'] for r in validator.results],
                         ['stale_ignore', 'stale_ignore', 'current'])
        self.assertEqual(validator.results[0]['status'], 'ok')
        self.assertEqual(validator.results[-1]['status'], 'recovered')
        self.assertTrue(replies[-1]['payload']['diagnostics'])

    def run_probe(self, body, **kwargs):
        with tempfile.TemporaryDirectory() as directory:
            executable = Path(directory) / 'compiler with spaces'
            executable.write_text('#!' + sys.executable + '\n' + body)
            executable.chmod(0o700)
            return runtime.probe(str(executable), (json.dumps(REQUEST) + '\n').encode(), **kwargs)

    def test_probe_success_and_stderr_is_not_protocol(self):
        response = json.dumps(RESPONSE)
        validator, errors, info = self.run_probe(
            'import sys\nsys.stdin.read()\nprint(' + repr(response) + ')\nprint("diagnostic",file=sys.stderr)\n')
        self.assertEqual(errors, [])
        self.assertEqual(validator.results[0]['preview'], 'current')
        self.assertEqual(info['stderr'], 'diagnostic\n')

    def test_probe_timeout_kills_child_and_reports_failure(self):
        _, errors, info = self.run_probe('import time\ntime.sleep(20)\n', timeout=0.1)
        self.assertIn('timed out', errors[0]['message'])
        self.assertNotEqual(info['returncode'], 0)

    def test_probe_nonzero_exit_and_premature_eof(self):
        validator, errors, info = self.run_probe('import sys\nsys.stdin.read()\nsys.stderr.write("broken")\nsys.exit(7)\n')
        self.assertEqual(info['returncode'], 7)
        self.assertEqual(info['stderr'], 'broken')
        self.assertIn('nonzero', errors[0]['message'])
        self.assertEqual(set(validator.requests) - validator.completed, {REQUEST['id']})
        validator, errors, _ = self.run_probe('import sys\nsys.stdin.read()\n')
        self.assertEqual(errors, [])
        self.assertEqual(set(validator.requests) - validator.completed, {REQUEST['id']})
        # The CLI final report turns these pending requests into an EOF failure.

    def test_probe_launch_errors_and_non_utf8_stderr(self):
        data = (json.dumps(REQUEST) + '\n').encode()
        _, errors, info = runtime.probe('/no/such/flashtex-executable', data)
        self.assertIn('launch failed', errors[0]['message'])
        self.assertIsNone(info['returncode'])
        with tempfile.TemporaryDirectory() as directory:
            file = Path(directory) / 'not-executable'
            file.write_text('not executable')
            file.chmod(0o600)
            _, errors, _ = runtime.probe(str(file), data)
            self.assertIn('launch failed', errors[0]['message'])
        _, errors, info = self.run_probe('import sys\nsys.stdin.read()\nsys.stderr.buffer.write(bytes([255,254]))\n')
        self.assertEqual(errors, [])
        self.assertEqual(info['stderr'], '\ufffd\ufffd')

    def test_probe_timeout_cleans_forked_pipe_holder(self):
        # Parent exits immediately; its child retains pipes. Timeout must still
        # kill the original process group and return instead of awaiting EOF.
        start = time.monotonic()
        _, errors, info = self.run_probe(
            'import os,time\nif os.fork() == 0:\n time.sleep(20)\nelse:\n os._exit(0)\n', timeout=1.5)
        self.assertLess(time.monotonic() - start, 5)
        self.assertTrue(any('timed out' in e['message'] for e in errors))
        self.assertEqual(info['returncode'], 0)

    def test_probe_setup_failure_reaps_child(self):
        children = []
        original = subprocess.Popen
        def record_child(*args, **kwargs):
            child = original(*args, **kwargs)
            children.append(child)
            return child
        with mock.patch.object(runtime.subprocess, 'Popen', side_effect=record_child):
            with mock.patch.object(runtime.os, 'set_blocking', side_effect=OSError('setup failed')):
                with self.assertRaisesRegex(OSError, 'setup failed'):
                    self.run_probe('import time\ntime.sleep(20)\n')
        self.assertEqual(len(children), 1)
        self.assertIsNotNone(children[0].returncode)
        self.assertTrue(all(stream.closed for stream in (children[0].stdin, children[0].stdout, children[0].stderr)))

    def test_probe_partial_writes_and_backpressure(self):
        # Exercise deterministic short writes rather than relying on pipe capacity.
        write = os.write
        def short_write(fd, data):
            return write(fd, data[:7])
        response = json.dumps(RESPONSE)
        with mock.patch.object(runtime.os, 'write', side_effect=short_write):
            validator, errors, _ = self.run_probe(
                'import sys,json\nrequest=json.loads(sys.stdin.read())\nassert request["id"]==' + repr(REQUEST['id']) +
                '\nsys.stderr.write("x"*100000)\nprint(' + repr(response) + ')\n')
        self.assertEqual(errors, [])
        self.assertEqual(len(validator.results), 1)

    def test_python39_syntax_compatibility(self):
        # Syntax check only; execution on an actual Python3.9 remains a CI gate.
        ast.parse((ROOT / 'scripts/check_runtime.py').read_text(), feature_version=(3, 9))

    def test_probe_cli_fails_on_premature_eof(self):
        with tempfile.TemporaryDirectory() as directory:
            executable = Path(directory) / 'empty-worker'
            executable.write_text('#!' + sys.executable + '\nimport sys\nsys.stdin.read()\n')
            executable.chmod(0o700)
            result = subprocess.run([sys.executable, str(ROOT / 'scripts/check_runtime.py'),
                                     '-', '--compiler', str(executable)],
                                    input=json.dumps(REQUEST) + '\n', text=True, capture_output=True, timeout=5)
        self.assertEqual(result.returncode, 1)
        report = json.loads(result.stdout)
        self.assertEqual(report['pending_request_ids'], [REQUEST['id']])
        self.assertIn('missing terminal', report['errors'][0]['message'])

    def test_probe_truncated_stdout_is_protocol_error(self):
        _, errors, _ = self.run_probe('import sys\nsys.stdin.read()\nsys.stdout.write(\'{"id":\')\n')
        self.assertTrue(errors)
        self.assertEqual(errors[0]['file'], 'compiler.stdout')

    def test_probe_caps_stdout_and_stderr(self):
        _, errors, _ = self.run_probe('import sys\nsys.stdout.write("x"*10000)\n', max_output_bytes=100)
        self.assertTrue(any('exceeds' in e['message'] for e in errors))
        _, errors, info = self.run_probe('import sys\nsys.stdin.read()\nsys.stderr.write("x"*100000)\n')
        self.assertEqual(errors, [])
        self.assertEqual(len(info['stderr']), 65536)
        self.assertTrue(info['stderr_truncated'])

    def test_probe_does_not_launch_on_invalid_input(self):
        _, errors, info = runtime.probe('/must/not/be/executed', b'not json\n')
        self.assertTrue(errors)
        self.assertIsNone(info['returncode'])

    @unittest.skipUnless(os.environ.get('FLASHTEX_COMPILER_BIN'), 'real compiler executable optional')
    def test_live_probe_real_unicode_and_recovery(self):
        requests = []
        for revision, text in enumerate(('aé𝛼z', r'Good \unknown{oops}.'), 1):
            request, _ = pair(revision)
            request['payload']['documents'][0]['text'] = text
            requests.append(request)
        data = '\n'.join(json.dumps(request) for request in requests).encode()
        validator, errors, info = runtime.probe(os.environ['FLASHTEX_COMPILER_BIN'], data)
        self.assertEqual(errors, [])
        self.assertEqual(info['returncode'], 0)
        self.assertEqual([r['status'] for r in validator.results], ['ok', 'recovered'])
        self.assertEqual([r['preview'] for r in validator.results], ['stale_ignore', 'current'])

    def capture_messages(self):
        def chunk(kind, data):
            return struct.pack('>I', len(data)) + kind + data + struct.pack('>I', zlib.crc32(kind + data))
        png = (b'\x89PNG\r\n\x1a\n' + chunk(b'IHDR', struct.pack('>IIBBBBB', 1, 1, 8, 2, 0, 0, 0))
               + chunk(b'IDAT', zlib.compress(b'\x00\xff\x00\x00')) + chunk(b'IEND', b''))
        submit = {'protocol_version': 1, 'id': 'capture-request', 'type': 'capture_submit', 'payload': {
            'capture_id': 'capture1', 'destination_id': 'anchor1', 'base_revision': 1,
            'image': {'mime_type': 'image/png', 'data_base64': base64.b64encode(png).decode()},
            'instructions': 'Transcribe é'}}
        receipt = {'protocol_version': 1, 'id': submit['id'], 'type': 'capture_received', 'payload': {'capture_id': 'capture1'}}
        proposal = {'protocol_version': 1, 'id': submit['id'], 'type': 'capture_proposal', 'payload': {
            'capture_id': 'capture1', 'latex': r'\alpha', 'ambiguities': [], 'required_dependencies': []}}
        return submit, receipt, proposal

    def test_capture_flow_and_async_receipt_order(self):
        submit, receipt, proposal = self.capture_messages()
        for messages in ([submit, receipt, proposal], [submit, proposal, receipt]):
            validator, errors = self.validate(messages)
            self.assertEqual(errors, [])
            self.assertEqual(len(validator.results), 2)
            self.assertEqual(validator.results[-1]['insertion'], 'not_validated_by_runtime_v1')

    def test_capture_retransmission_is_idempotent_but_conflicting_reuse_rejected(self):
        submit, receipt, proposal = self.capture_messages()
        validator, errors = self.validate([submit, submit, receipt, receipt, proposal, proposal])
        self.assertEqual(errors, [])
        self.assertEqual(len(validator.results), 2)
        for field, value in [('destination_id', 'other-anchor'), ('base_revision', 2), ('instructions', 'changed')]:
            changed = copy.deepcopy(submit)
            changed['payload'][field] = value
            _, errors = self.validate([submit, changed])
            self.assertIn('conflicting', errors[0]['message'])
        changed = copy.deepcopy(proposal)
        changed['payload']['latex'] = 'changed'
        _, errors = self.validate([submit, proposal, changed])
        self.assertIn('conflicting duplicate', errors[0]['message'])

    def test_capture_exact_correlation_and_compile_collision(self):
        submit, receipt, _ = self.capture_messages()
        for field in ('id', 'capture_id'):
            changed = copy.deepcopy(receipt)
            (changed if field == 'id' else changed['payload'])[field] = 'unknown'
            _, errors = self.validate([submit, changed])
            self.assertIn('does not match', errors[0]['message'])
        request = copy.deepcopy(REQUEST)
        request['id'] = submit['id']
        for messages in ([submit, request], [request, submit]):
            _, errors = self.validate(messages)
            self.assertIn('collides', errors[0]['message'])

    def test_capture_malformed_and_oversized_image_metadata(self):
        submit, _, _ = self.capture_messages()
        for image in ({'mime_type': 'image/png', 'data_base64': '!!'},
                      {'mime_type': 'image/jpeg', 'data_base64': submit['payload']['image']['data_base64']},
                      {'mime_type': 'image/png', 'data_base64': 'a' * (12 * 1024 * 1024)}):
            changed = copy.deepcopy(submit)
            changed['payload']['image'] = image
            _, errors = self.validate([changed])
            self.assertTrue(errors)
        raw = bytearray(base64.b64decode(submit['payload']['image']['data_base64']))
        raw[-15] ^= 1
        submit['payload']['image']['data_base64'] = base64.b64encode(raw).decode()
        _, errors = self.validate([submit])
        self.assertIn('CRC', errors[0]['message'])

    def test_capture_proposal_fields_and_probe_exclusion(self):
        submit, _, proposal = self.capture_messages()
        proposal['payload']['ambiguities'] = [False]
        _, errors = self.validate([submit, proposal])
        self.assertIn('string', errors[0]['message'])
        _, errors, info = runtime.probe('/must/not/launch', json.dumps(submit).encode())
        self.assertIsNone(info['returncode'])
        self.assertTrue(any('compile requests only' in e['message'] for e in errors))

    def test_capture_receipt_not_proposal_controls_pending_status(self):
        submit, receipt, proposal = self.capture_messages()
        command = [sys.executable, str(ROOT / 'scripts/check_runtime.py'), '-']
        for messages, code in [([submit, proposal], 1), ([submit, receipt], 0)]:
            result = subprocess.run(command, input='\n'.join(json.dumps(m) for m in messages), text=True, capture_output=True)
            self.assertEqual(result.returncode, code)
            if code:
                self.assertEqual(json.loads(result.stdout)['pending_capture_ids'], ['capture1'])

    def test_error_terminal_explicitly_reports_unspecified_schema(self):
        response = {'protocol_version': 1, 'id': REQUEST['id'], 'type': 'error', 'payload': {}}
        validator, errors = self.validate([REQUEST, response])
        self.assertEqual(errors, [])
        self.assertEqual(validator.results[0]['preview'], 'unchanged')
        self.assertIn('not defined', validator.results[0]['note'])


class TransferValidationTests(unittest.TestCase):
    def setUp(self):
        self.validator = runtime.TransferValidator()
        self.counter = 0
        self.messages = []

    def exchange(self, kind, request, payload, reply=None):
        self.counter += 1
        ident = 'transfer-' + str(self.counter)
        values = [{'protocol_version': 1, 'id': ident, 'type': kind, 'payload': request},
                  {'protocol_version': 1, 'id': ident, 'type': reply or self.validator.replies[kind], 'payload': payload}]
        for value in values:
            self.validator.consume(value)
            self.messages.append(copy.deepcopy(value))
        return values

    def setup_capture(self):
        self.source = 'aé𝛼z'
        self.exchange('document_open', {'project_id': 'project', 'path': 'main.tex', 'revision': 1, 'text': self.source}, {})
        self.binding = {'project_id': 'project', 'path': 'main.tex', 'revision': 1, 'start_byte': 1, 'end_byte': 3,
                        'source_sha256': __import__('hashlib').sha256(self.source.encode()).hexdigest()}
        self.anchor = {'destination_id': 'anchor1', 'project_id': 'project', 'path': 'main.tex',
                       'pinned_revision': 1, 'current_revision': 1, 'start_byte': 1, 'end_byte': 3, 'valid': True,
                       'binding': self.binding}
        self.exchange('destination_pin', {'destination_id': 'anchor1', **{k: v for k, v in self.binding.items() if k != 'source_sha256'}}, self.anchor)
        self.submit = RuntimeValidationTests().capture_messages()[0]['payload']
        self.exchange('capture_submit', self.submit, {'capture_id': 'capture1', 'durable': True, 'has_proposal': False, 'applied': False})

    def convert(self):
        self.proposal = {'capture_id': 'capture1', 'latex': 'x', 'ambiguities': [], 'required_dependencies': [], 'context_revision': 1}
        self.exchange('capture_convert', {'capture_id': 'capture1', 'supported_features': []}, self.proposal)

    def prepare(self):
        self.edit = {'capture_id': 'capture1', 'edit_id': 'capture-capture1', 'project_id': 'project', 'path': 'main.tex',
                     'expected_revision': 1, 'start_byte': 1, 'end_byte': 3, 'removed_text': 'é', 'replacement': 'x',
                     'document_before_sha256': self.binding['source_sha256']}
        self.exchange('capture_prepare_insert', {'capture_id': 'capture1', 'expected_revision': 1, 'approved': True}, self.edit)

    def test_full_reviewed_flow_and_idempotent_receipts(self):
        self.setup_capture()
        self.convert()
        self.prepare()
        receipt = {'capture_id': 'capture1', 'edit_id': self.edit['edit_id'], 'new_revision': 2}
        self.exchange('capture_applied', receipt, receipt)
        self.exchange('capture_applied', receipt, receipt)
        self.assertEqual(self.validator.documents[('project', 'main.tex')]['text'], 'ax𝛼z')
        self.exchange('capture_status', {'capture_id': 'capture1'}, {
            'capture_id': 'capture1', 'proposal': {k: self.proposal[k] for k in ('latex', 'ambiguities', 'required_dependencies')},
            'prepared': self.edit, 'applied': {'edit_id': receipt['edit_id'], 'new_revision': 2}, 'rejected': False})
        with self.assertRaisesRegex(runtime.Invalid, 'conflicting application'):
            self.exchange('capture_applied', {**receipt, 'new_revision': 3}, {**receipt, 'new_revision': 3})

    def test_stale_context_and_review_revisions_refused(self):
        self.setup_capture()
        with self.assertRaisesRegex(runtime.Invalid, 'context revision'):
            self.exchange('capture_convert', {'capture_id': 'capture1'}, {
                'capture_id': 'capture1', 'latex': 'x', 'ambiguities': [], 'required_dependencies': [], 'context_revision': 0})
        self.convert()
        with self.assertRaisesRegex(runtime.Invalid, 'stale approval'):
            self.exchange('capture_prepare_insert', {'capture_id': 'capture1', 'expected_revision': 0, 'approved': True}, {})

    def test_bad_source_span_and_target_hash_rejected_atomically(self):
        self.setup_capture()
        with self.assertRaisesRegex(runtime.Invalid, 'UTF-8'):
            self.exchange('document_edit', {'project_id': 'project', 'path': 'main.tex', 'base_revision': 1,
                'revision': 2, 'start_byte': 2, 'end_byte': 3, 'replacement': 'x'}, {'revision': 2})
        self.assertEqual(self.validator.documents[('project', 'main.tex')]['revision'], 1)
        self.convert()
        self.prepare()
        changed = {**self.edit, 'document_before_sha256': '0' * 64}
        with self.assertRaisesRegex(runtime.Invalid, 'prepared edit changed'):
            self.exchange('capture_prepare_insert', {'capture_id': 'capture1', 'expected_revision': 1, 'approved': True}, changed)

    def test_correct_error_reply_accepts_refusal_without_state_mutation(self):
        self.setup_capture()
        self.exchange('document_edit', {'project_id': 'project', 'path': 'main.tex', 'base_revision': 0,
            'revision': 2, 'start_byte': 0, 'end_byte': 0, 'replacement': 'x'},
            {'code': 'revision_conflict', 'message': 'stale'}, reply='error')
        self.assertEqual(self.validator.documents[('project', 'main.tex')]['text'], self.source)
        self.exchange('capture_reject', {'capture_id': 'capture1'}, {'capture_id': 'capture1'})
        with self.assertRaisesRegex(runtime.Invalid, 'after rejection'):
            self.convert()

    def test_transfer_rejects_boolean_revisions_and_counts_newline_limit(self):
        with self.assertRaisesRegex(runtime.Invalid, 'u64'):
            self.exchange('document_open', {'project_id': 'project', 'path': 'main.tex', 'revision': True, 'text': 'x'}, {})
        message = {'protocol_version': 1, 'id': 'boundary', 'type': 'capture_status', 'payload': {'capture_id': 'capture1'}}
        raw = json.dumps(message).encode()
        errors = runtime.validate_stream(io.BytesIO(raw + b'\n'), runtime.TransferValidator(), max_line_bytes=len(raw))
        self.assertIn('exceeds', errors[0]['message'])
        errors = runtime.validate_stream(io.BytesIO(raw + b'\n'), runtime.TransferValidator(), max_line_bytes=len(raw) + 1)
        self.assertEqual(errors, [])

    def test_reply_ids_duplicates_and_cross_type_conflicts(self):
        pair = self.exchange('document_open', {'project_id': 'project', 'path': 'main.tex', 'revision': 1, 'text': 'x'}, {})
        self.validator.consume(pair[1])
        with self.assertRaises(runtime.Invalid):
            self.validator.consume({**pair[1], 'payload': {'unexpected': True}})
        with self.assertRaisesRegex(runtime.Invalid, 'no matching'):
            self.validator.consume({**pair[1], 'id': 'unknown'})
        with self.assertRaisesRegex(runtime.Invalid, 'reply type'):
            self.validator.consume({**pair[1], 'type': 'capture_received'})

    def test_rebase_and_intersecting_edit_invalidation(self):
        self.setup_capture()
        self.exchange('document_edit', {'project_id': 'project', 'path': 'main.tex', 'base_revision': 1,
            'revision': 2, 'start_byte': 0, 'end_byte': 0, 'replacement': 'β'}, {'revision': 2})
        self.assertEqual(self.validator.anchors['anchor1']['start_byte'], 3)
        self.exchange('document_edit', {'project_id': 'project', 'path': 'main.tex', 'base_revision': 2,
            'revision': 3, 'start_byte': 3, 'end_byte': 5, 'replacement': 'x'}, {'revision': 3})
        with self.assertRaisesRegex(runtime.Invalid, 'invalid destination'):
            self.convert()

    def test_restart_retains_journal_but_requires_document_resynchronization(self):
        self.setup_capture()
        self.convert()
        self.prepare()
        self.validator.documents.clear()
        self.validator.anchors.clear()
        self.exchange('document_open', {'project_id': 'project', 'path': 'main.tex', 'revision': 1, 'text': self.source}, {})
        receipt = {'capture_id': 'capture1', 'edit_id': self.edit['edit_id'], 'new_revision': 2}
        self.exchange('capture_applied', receipt, receipt)
        self.validator.documents.clear()
        self.validator.anchors.clear()
        self.exchange('document_open', {'project_id': 'project', 'path': 'main.tex', 'revision': 2, 'text': 'ax𝛼z'}, {})
        self.exchange('capture_applied', receipt, receipt)
        self.assertEqual(self.validator.documents[('project', 'main.tex')]['text'], 'ax𝛼z')

    @unittest.skipUnless(os.environ.get('FLASHTEX_BRIDGE_BIN'), 'set reviewed local bridge executable for real CLI gate')
    def test_real_bridge_receipt_rejection_and_provider_disabled(self):
        self.setup_capture()
        requests = [m for index, m in enumerate(self.messages) if index % 2 == 0]
        requests += [{'protocol_version': 1, 'id': 'convert-disabled', 'type': 'capture_convert', 'payload': {'capture_id': 'capture1'}},
                     {'protocol_version': 1, 'id': 'reject', 'type': 'capture_reject', 'payload': {'capture_id': 'capture1'}}]
        with tempfile.TemporaryDirectory() as directory:
            result = subprocess.run([os.environ['FLASHTEX_BRIDGE_BIN'], '--store', directory],
                input='\n'.join(json.dumps(r) for r in requests) + '\n', text=True, capture_output=True, timeout=10)
        self.assertEqual(result.returncode, 0, result.stderr)
        replies = [json.loads(line) for line in result.stdout.splitlines()]
        self.assertEqual(len(replies), len(requests))
        transcript = [message for pair in zip(requests, replies) for message in pair]
        validator = runtime.TransferValidator()
        errors = runtime.validate_stream(io.BytesIO(('\n'.join(json.dumps(m) for m in transcript)).encode()), validator)
        self.assertEqual(errors, [])
        self.assertEqual(replies[-2]['payload']['code'], 'provider_disabled')
        self.assertTrue(validator.captures['capture1']['rejected'])


if __name__ == '__main__':
    unittest.main()
