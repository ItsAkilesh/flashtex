#!/usr/bin/env python3
"""Bounded real-helper replay; temporary project only, no network/provider calls."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import select
import subprocess
import tempfile
import time


class Client:
    def __init__(self, binary, config):
        self.proc = subprocess.Popen([binary, str(config)], stdin=subprocess.PIPE,
                                     stdout=subprocess.PIPE, stderr=subprocess.DEVNULL)
        self.buffer = bytearray()

    def send(self, identity, kind, payload):
        value = dict(protocol_version=1, session_id="benchmark", id=identity,
                     type=kind, payload=payload)
        self.proc.stdin.write(json.dumps(value).encode() + b"\n")
        self.proc.stdin.flush()

    def read(self):
        deadline = time.monotonic() + 15
        while b"\n" not in self.buffer:
            remaining = deadline - time.monotonic()
            if remaining <= 0 or not select.select([self.proc.stdout], [], [], remaining)[0]:
                raise RuntimeError("helper response deadline")
            data = os.read(self.proc.stdout.fileno(), 65536)
            if not data:
                raise RuntimeError("helper EOF")
            self.buffer.extend(data)
            if len(self.buffer) > 17 * 1024 * 1024:
                raise RuntimeError("helper read buffer bound")
        line, _, tail = self.buffer.partition(b"\n")
        self.buffer = bytearray(tail)
        received = time.monotonic()
        return json.loads(line), received

    def stop(self):
        self.proc.kill()
        self.proc.wait(timeout=5)
        self.proc.stdin.close()
        self.proc.stdout.close()


def run(args):
    helper = str(Path(args.helper).resolve())
    compiler = str(Path(args.compiler).resolve())
    prefix = "\\documentclass{article}\n\\begin{document}\n"
    text = prefix + ("Alpha beta gamma delta.\n" * (args.size // 23)) + "\\end{document}\n"
    with tempfile.TemporaryDirectory(prefix="flashtex-helper-bench-") as directory:
        root = Path(directory)
        project = root / "project"
        project.mkdir()
        (root / "ledger").mkdir()
        (project / "main.tex").write_text(text)
        config = root / "config.json"
        config.write_text(json.dumps(dict(session_id="benchmark", project_id="p",
            entry_path="main.tex", project_root=str(project),
            private_ledger_root=str(root / "ledger"), compiler_path=compiler,
            compiler_max_frame_bytes=12 * 1024 * 1024)))
        client = Client(helper, config)
        document = None
        try:
            assert client.read()[0]["type"] == "ready"
            client.send("document", "document", dict(path="main.tex"))
            initial_preview = False
            while document is None or not initial_preview:
                event, _ = client.read()
                if event.get("id") == "document":
                    document = event["payload"]["document"]
                if event.get("payload", {}).get("kind") == "preview":
                    initial_preview = True
            for index in range(args.edits):
                edited = text.replace("Alpha", "Omega" if index % 2 == 0 else "Sigma", 1)
                identity = f"edit-{index}"
                started = time.monotonic()
                client.send(identity, "edit", dict(path="main.tex",
                    expected_revision=document["revision"],
                    expected_sha256=document["source_sha256"], text=edited))
                acknowledgement = preview = None
                while acknowledgement is None or preview is None:
                    event, received = client.read()
                    if event.get("type") == "error":
                        raise RuntimeError(event["payload"]["message"])
                    if event.get("id") == identity:
                        acknowledgement = event["payload"]
                        document = acknowledgement["document"]
                        ack_ms = (received - started) * 1000
                    payload = event.get("payload", {})
                    if payload.get("kind") == "failed":
                        raise RuntimeError(payload["reason"])
                    if payload.get("kind") == "preview":
                        preview = payload
                        wire_ms = (received - started) * 1000
                assert document["text"] == edited
                assert preview["source_versions"] == {"main.tex": document["revision"]}
                result = preview["result"]
                request = dict(protocol_version=1, id=result["id"], type="compile", payload=dict(
                    project_id="p", revision=result["payload"]["revision"],
                    entry_path="main.tex", documents=[dict(path="main.tex", text=edited)]))
                fresh = subprocess.run([compiler], input=json.dumps(request).encode() + b"\n",
                                       capture_output=True, timeout=15, check=True)
                assert json.loads(fresh.stdout) == result
                print(json.dumps(dict(type="sample", index=index, source_bytes=len(edited.encode()),
                    ack_wire_ms=ack_ms, preview_wire_ms=wire_ms,
                    save_and_submit_ms=acknowledgement["save_and_submit_ms"],
                    runtime_total_ms=preview["runtime_total_ms"],
                    controller_total_ms=preview["controller_total_ms"], exact_clean_equal=True)), flush=True)
        finally:
            client.stop()
        # Kill/reopen proves the measured acknowledged edits remain authoritative.
        client = Client(helper, config)
        try:
            assert client.read()[0]["type"] == "ready"
            client.send("reopen", "document", dict(path="main.tex"))
            while True:
                event, _ = client.read()
                if event.get("id") == "reopen":
                    assert event["payload"]["document"] == document
                    break
        finally:
            client.stop()
        print(json.dumps(dict(type="summary", edits=args.edits, exact_reopen=True,
            helper_sha256=hashlib.sha256(Path(helper).read_bytes()).hexdigest(),
            compiler_sha256=hashlib.sha256(Path(compiler).read_bytes()).hexdigest(),
            native_paint_measured=False, lost_reply_retry_measured=False)))


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--helper", required=True)
    parser.add_argument("--compiler", required=True)
    parser.add_argument("--size", type=int, default=5000)
    parser.add_argument("--edits", type=int, default=5)
    args = parser.parse_args()
    if not 100 <= args.size <= 500000 or not 1 <= args.edits <= 20:
        parser.error("size must be100..500000 and edits1..20")
    run(args)
