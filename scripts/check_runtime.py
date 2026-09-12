#!/usr/bin/env python3
"""Validate runtime-v1 compile JSONLines transcripts; no compiler/model execution.

Usage: check_runtime.py transcript.jsonl (or - for stdin). Requests and responses
must appear in observed transport order in one transcript. Use --requests FILE
for a request stream followed by a separate response stream; every response older
than the newest supplied request is then classified as stale for preview purposes.
Exit 1 means invalid protocol, not stale but valid asynchronous output.
"""
import argparse
import json
import math
from pathlib import PurePosixPath, PureWindowsPath
import sys


class Invalid(ValueError):
    pass


def require(condition, message):
    if not condition:
        raise Invalid(message)


def integer(value):
    return isinstance(value, int) and not isinstance(value, bool)


def number(value):
    try:
        return isinstance(value, (int, float)) and not isinstance(value, bool) and math.isfinite(value)
    except OverflowError:
        return False


def string(value, label, nonempty=False):
    require(isinstance(value, str) and (bool(value) or not nonempty), label + ' must be a string' + (' (nonempty)' if nonempty else ''))
    try:
        value.encode('utf-8')
    except UnicodeEncodeError as exc:
        raise Invalid(label + ' contains an invalid Unicode surrogate') from exc


def path(value):
    string(value, 'path', True)
    require(not PurePosixPath(value).is_absolute() and not PureWindowsPath(value).drive
            and not value.startswith('\\') and '..' not in value.replace('\\', '/').split('/')
            and '\x00' not in value, 'path must be project-relative without parent traversal')


def object_value(value, label):
    require(isinstance(value, dict), label + ' must be an object')


def array(value, label):
    require(isinstance(value, list), label + ' must be an array')


def strict_object(pairs):
    result = {}
    for key, value in pairs:
        require(key not in result, 'duplicate JSON key: ' + key)
        result[key] = value
    return result


class Validator:
    def __init__(self):
        self.requests = {}
        self.completed = set()
        self.latest = {}
        self.results = []

    def source(self, value, documents):
        object_value(value, 'source')
        name = value.get('path')
        path(name)
        require(name in documents, 'source.path is absent from the request documents: ' + name)
        start, end = value.get('start_byte'), value.get('end_byte')
        data = documents[name].encode('utf-8')
        require(integer(start) and integer(end) and 0 <= start <= end <= len(data),
                'source byte range must satisfy 0 <= start_byte <= end_byte <= UTF-8 length')
        try:
            data[:start].decode('utf-8')
            data[:end].decode('utf-8')
        except UnicodeDecodeError as exc:
            raise Invalid('source offsets split a UTF-8 character') from exc

    def consume(self, value):
        object_value(value, 'envelope')
        require(integer(value.get('protocol_version')) and value['protocol_version'] == 1,
                'unsupported protocol_version (expected integer 1)')
        ident, kind, payload = value.get('id'), value.get('type'), value.get('payload')
        string(ident, 'id', True)
        object_value(payload, 'payload')
        require(kind in ('compile', 'compile_result', 'error'),
                'unsupported message type for compile transcript validator: ' + str(kind))
        if kind == 'error':
            # runtime-v1 does not yet define the error payload schema.
            require(ident in self.requests, 'error has no matching compile request id')
            require(ident not in self.completed, 'duplicate terminal response id')
            self.completed.add(ident)
            self.results.append({'id': ident, 'type': 'error', 'preview': 'unchanged',
                                 'note': 'error payload schema is not defined by runtime-v1'})
            return
        project, revision = payload.get('project_id'), payload.get('revision')
        string(project, 'project_id', True)
        require(integer(revision) and revision >= 0, 'revision must be a nonnegative integer')
        if kind == 'compile':
            require(ident not in self.requests, 'duplicate request id')
            require(project not in self.latest or revision >= self.latest[project],
                    'compile request revisions must not decrease within a project/session')
            path(payload.get('entry_path'))
            documents = payload.get('documents')
            array(documents, 'documents')
            mapped = {}
            for document in documents:
                object_value(document, 'document')
                name, text = document.get('path'), document.get('text')
                path(name)
                string(text, 'document.text')
                require(name not in mapped, 'duplicate document path: ' + name)
                mapped[name] = text
            require(payload['entry_path'] in mapped, 'entry_path must exist in request documents')
            self.requests[ident] = (project, revision, mapped)
            self.latest[project] = revision
            return
        require(ident in self.requests, 'compile_result has no matching request id')
        require(ident not in self.completed, 'duplicate terminal response id')
        expected_project, expected_revision, documents = self.requests[ident]
        require((project, revision) == (expected_project, expected_revision),
                'compile_result project_id/revision does not match its request id')
        require(payload.get('status') in ('ok', 'recovered', 'failed'), 'invalid compile_result status')
        pages, diagnostics = payload.get('pages'), payload.get('diagnostics')
        array(pages, 'pages')
        array(diagnostics, 'diagnostics')
        seen_pages = set()
        for page in pages:
            object_value(page, 'page')
            n = page.get('number')
            require(integer(n) and n >= 1 and n not in seen_pages, 'page.number must be unique and positive')
            seen_pages.add(n)
            for field in ('width_pt', 'height_pt'):
                require(number(page.get(field)) and page[field] > 0, field + ' must be finite and positive')
            array(page.get('items'), 'page.items')
            for item in page['items']:
                object_value(item, 'item')
                require(item.get('kind') == 'text', 'runtime-v1 supports text items only')
                string(item.get('text'), 'item.text')
                for field in ('x_pt', 'baseline_y_pt', 'font_size_pt'):
                    require(number(item.get(field)), field + ' must be a finite number')
                require(item['font_size_pt'] > 0, 'font_size_pt must be positive')
                self.source(item.get('source'), documents)
        for diagnostic in diagnostics:
            object_value(diagnostic, 'diagnostic')
            require(diagnostic.get('severity') in ('error', 'warning'), 'invalid diagnostic.severity')
            string(diagnostic.get('message'), 'diagnostic.message')
            require('source' in diagnostic and 'recovery' in diagnostic, 'diagnostic needs source and recovery (null allowed)')
            if diagnostic['source'] is not None:
                self.source(diagnostic['source'], documents)
            if diagnostic['recovery'] is not None:
                string(diagnostic['recovery'], 'diagnostic.recovery')
        if payload.get('pdf_path') is not None:
            string(payload['pdf_path'], 'pdf_path', True)
        self.completed.add(ident)
        self.results.append({'id': ident, 'project_id': project, 'revision': revision,
                             'status': payload['status'],
                             'preview': 'stale_ignore' if revision < self.latest[project] else 'current'})


def validate_stream(stream, validator, name='<stdin>', max_line_bytes=8 * 1024 * 1024):
    errors = []
    line_number = 0
    while True:
        # Bounded readline avoids allocating an arbitrarily large transport line.
        raw = stream.readline(max_line_bytes + 1)
        if not raw:
            break
        line_number += 1
        try:
            require(len(raw) <= max_line_bytes, 'JSON line exceeds --max-line-bytes')
            value = json.loads(raw.decode('utf-8'), object_pairs_hook=strict_object,
                               parse_constant=lambda token: (_ for _ in ()).throw(Invalid('non-finite JSON number: ' + token)))
            validator.consume(value)
        except (Invalid, ValueError, UnicodeError, RecursionError) as exc:
            errors.append({'file': name, 'line': line_number, 'message': str(exc)})
            if len(raw) > max_line_bytes:
                # Consume the oversized record in bounded chunks, then recover.
                while raw and not raw.endswith(b'\n'):
                    raw = stream.readline(max_line_bytes + 1)
    return errors


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('transcript', help='JSONLines file, or - for stdin')
    parser.add_argument('--requests', help='optional request-only JSONLines prefix')
    parser.add_argument('--allow-pending', action='store_true', help='allow requests without terminal responses in a partial capture')
    parser.add_argument('--max-line-bytes', type=int, default=8 * 1024 * 1024)
    args = parser.parse_args(argv)
    if args.max_line_bytes < 1:
        parser.error('--max-line-bytes must be positive')
    validator, errors = Validator(), []
    for filename in ([args.requests] if args.requests else []) + [args.transcript]:
        try:
            if filename == '-':
                errors.extend(validate_stream(sys.stdin.buffer, validator, filename, args.max_line_bytes))
            else:
                with open(filename, 'rb') as stream:
                    errors.extend(validate_stream(stream, validator, filename, args.max_line_bytes))
        except OSError as exc:
            errors.append({'file': filename, 'message': str(exc)})
    pending = sorted(set(validator.requests) - validator.completed)
    if pending and not args.allow_pending:
        errors.append({'message': 'missing terminal responses', 'request_ids': pending})
    print(json.dumps({'valid': not errors, 'errors': errors, 'responses': validator.results,
                      'pending_request_ids': pending}, indent=2))
    return 1 if errors else 0


if __name__ == '__main__':
    sys.exit(main())
