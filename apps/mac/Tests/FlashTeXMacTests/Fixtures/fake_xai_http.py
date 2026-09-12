#!/usr/bin/env python3
"""Loopback stand-in for api.x.ai used by the Mac's connection probe tests
(`GrokProbe`, `FLASHTEX_GROK_BASE_URL`): binds 127.0.0.1 on a free port, prints
`PORT <n>` on stdout, then answers `GET /v1/models` by the bearer token:
`good-key` -> 200, `rate-key` -> 429, `slow-key` -> no reply for 8 s,
`server-key` -> 503, anything else -> 401. Any other path -> 404. Also answers
`POST /v1/responses` the way the helper's `grok.rs` expects a completed reply,
for when the helper gains a base-URL override (not yet; see docs/mac/grok-live.md).
Keys are compared, never printed. Exits on stdin EOF.
"""
import json
import sys
import threading
import time
from http.server import BaseHTTPRequestHandler, HTTPServer


class Handler(BaseHTTPRequestHandler):
    def log_message(self, *_):
        pass

    def _key(self):
        auth = self.headers.get("Authorization", "")
        return auth[len("Bearer "):] if auth.startswith("Bearer ") else ""

    def _send(self, status, body):
        data = json.dumps(body).encode("utf-8")
        self.send_response(status)
        self.send_header("Content-Type", "application/json")
        self.send_header("Content-Length", str(len(data)))
        self.end_headers()
        self.wfile.write(data)

    def _status_for_key(self):
        key = self._key()
        if key == "good-key":
            return 200
        if key == "rate-key":
            return 429
        if key == "server-key":
            return 503
        if key == "slow-key":
            time.sleep(8)
            return 200
        return 401

    def do_GET(self):
        if self.path != "/v1/models":
            return self._send(404, {"error": "not found"})
        status = self._status_for_key()
        self._send(status, {"data": [{"id": "grok-4.6"}]} if status == 200 else {"error": "status %d" % status})

    def do_POST(self):
        length = int(self.headers.get("Content-Length", "0"))
        body = self.rfile.read(length) if length else b""
        if self.path != "/v1/responses":
            return self._send(404, {"error": "not found"})
        status = self._status_for_key()
        if status != 200:
            return self._send(status, {"error": "status %d" % status})
        try:
            request = json.loads(body)
            user = json.loads(next(m["content"] for m in request["input"] if m["role"] == "user"))
            context_id = user["context_id"]
        except (ValueError, KeyError, StopIteration):
            return self._send(400, {"error": "bad request"})
        proposal = {"context_id": context_id, "explanation": "Stub explanation", "edits": []}
        self._send(200, {"id": "resp_stub", "status": "completed", "model": request.get("model"),
                         "output": [{"type": "message", "role": "assistant",
                                     "content": [{"type": "output_text", "text": json.dumps(proposal)}]}],
                         "usage": {"input_tokens": 1, "output_tokens": 1}})


server = HTTPServer(("127.0.0.1", 0), Handler)
sys.stdout.write("PORT %d\n" % server.server_address[1])
sys.stdout.flush()
threading.Thread(target=server.serve_forever, daemon=True).start()
sys.stdin.read()
server.shutdown()
