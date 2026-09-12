#!/usr/bin/env python3
"""Concurrent real-helper typing replay; temporary source, bounded run, no provider calls."""
import argparse
from collections import Counter
import hashlib
import json
from pathlib import Path
import subprocess
import tempfile
import threading
import time
from helper_replay import Client, snapshot_after_initial_preview


def run(args):
    helper, compiler = str(Path(args.helper).resolve()), str(Path(args.compiler).resolve())
    text = "\\documentclass{article}\n\\begin{document}\n" + "Alpha beta gamma delta.\n" * (args.size // 23) + "\\end{document}\n"
    with tempfile.TemporaryDirectory(prefix="flashtex-burst-") as directory:
        root = Path(directory)
        (root / "project").mkdir()
        (root / "ledger").mkdir()
        (root / "project/main.tex").write_text(text)
        config = root / "config.json"
        config.write_text(json.dumps(dict(session_id="benchmark", project_id="p", entry_path="main.tex",
            project_root=str(root / "project"), private_ledger_root=str(root / "ledger"),
            compiler_path=compiler, compiler_max_frame_bytes=12 * 1024 * 1024)))
        client = Client(helper, config)
        writer = None
        try:
            before = snapshot_after_initial_preview(client)
            sources = [text.replace("Alpha", f"Edit{i:03}", 1) for i in range(args.edits)]
            starts, finishes, failures = [], [], []
            def send_edits():
                try:
                    previous = before["text"]
                    for index, source in enumerate(sources):
                        started = time.monotonic()
                        starts.append(started)
                        client.send(f"burst-{index}", "edit", dict(path="main.tex",
                            expected_revision=before["revision"] + index,
                            expected_sha256=hashlib.sha256(previous.encode()).hexdigest(), text=source))
                        finishes.append(time.monotonic())
                        previous = source
                        time.sleep(max(0, args.interval_ms / 1000 - (time.monotonic() - started)))
                except Exception as error:
                    failures.append(repr(error))
            writer = threading.Thread(target=send_edits)
            writer.start()
            acknowledged = set()
            counts = Counter()
            latest_ack_revision = before["revision"]
            final = None
            deadline = time.monotonic() + 30
            while len(acknowledged) < args.edits or final is None:
                if time.monotonic() > deadline:
                    raise RuntimeError("burst completion deadline")
                event, received = client.read()
                if event["type"] == "error":
                    raise RuntimeError(event["payload"]["message"])
                identity = event.get("id")
                if identity and identity.startswith("burst-"):
                    index = int(identity.split("-")[1])
                    assert index not in acknowledged
                    document = event["payload"]["document"]
                    assert document["text"] == sources[index]
                    assert document["revision"] == before["revision"] + index + 1
                    assert not event["payload"]["preview_error"]
                    latest_ack_revision = document["revision"]
                    acknowledged.add(index)
                payload = event.get("payload", {})
                kind = payload.get("kind")
                if kind:
                    counts[kind] += 1
                if kind == "failed":
                    raise RuntimeError(payload["reason"])
                if kind == "preview":
                    revision = payload["source_versions"]["main.tex"]
                    assert revision >= latest_ack_revision, "preview older than delivered durable acknowledgement"
                    if revision == before["revision"] + args.edits:
                        final, final_received = payload, received
            writer.join(timeout=5)
            assert not writer.is_alive() and not failures
            result = final["result"]
            request = dict(protocol_version=1, id=result["id"], type="compile", payload=dict(project_id="p",
                revision=result["payload"]["revision"], entry_path="main.tex",
                documents=[dict(path="main.tex", text=sources[-1])]))
            clean = subprocess.run([compiler], input=json.dumps(request).encode() + b"\n",
                                   capture_output=True, timeout=15, check=True)
            assert json.loads(clean.stdout) == result
        finally:
            client.stop()
            if writer is not None:
                writer.join(timeout=5)
        client = Client(helper, config)
        try:
            reopened = snapshot_after_initial_preview(client)
            assert reopened["text"] == sources[-1]
            assert reopened["revision"] == before["revision"] + args.edits
        finally:
            client.stop()
        print(json.dumps(dict(source_bytes=len(sources[-1].encode()), edits=args.edits,
            intended_interval_ms=args.interval_ms, acknowledged=len(acknowledged), updates=dict(counts),
            actual_intervals_ms=[(b-a)*1000 for a,b in zip(starts, starts[1:])],
            send_duration_ms=[(b-a)*1000 for a,b in zip(starts, finishes)],
            final_preview_after_last_send_start_ms=(final_received-starts[-1])*1000,
            exact_clean_final=True, exact_durable_reopen=True, native_paint_measured=False,
            helper_sha256=hashlib.sha256(Path(helper).read_bytes()).hexdigest(),
            compiler_sha256=hashlib.sha256(Path(compiler).read_bytes()).hexdigest())))


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--helper", required=True)
    parser.add_argument("--compiler", required=True)
    parser.add_argument("--size", type=int, default=50000)
    parser.add_argument("--edits", type=int, default=20)
    parser.add_argument("--interval-ms", type=int, default=30)
    args = parser.parse_args()
    if not 100 <= args.size <= 500000 or not 1 <= args.edits <= 50 or not 1 <= args.interval_ms <= 100:
        parser.error("size100..500000, edits1..50, interval-ms1..100 required")
    run(args)
