#!/usr/bin/env python3
"""Runtime-v1 protocol checks against a built flashtex-compiler binary.

Python 3 standard library only. Launches the worker as a subprocess, sends
`compile` envelopes as JSON Lines on stdin, and asserts on the replies read from
stdout. Exit status is non-zero when any check FAILs. See README.md.

Usage:
    check_protocol.py --compiler <path/to/flashtex-compiler> --repo <repo root>
                      [--latency-requests 20] [--timeout 10] [--json <out.json>]
"""

import argparse
import json
import os
import statistics
import subprocess
import sys
import threading
import time

PASS, FAIL, INFO = "PASS", "FAIL", "INFO"


class Worker:
    """One long-lived worker process speaking JSON Lines on stdin/stdout."""

    def __init__(self, binary, timeout):
        self.timeout = timeout
        self.proc = subprocess.Popen(
            [binary],
            stdin=subprocess.PIPE,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
        )

    def send(self, envelope):
        self.send_raw((json.dumps(envelope, ensure_ascii=False) + "\n").encode("utf-8"))

    def send_raw(self, data):
        self.proc.stdin.write(data)
        self.proc.stdin.flush()

    def recv(self):
        # readline() blocks; run it on a helper thread so a silent worker
        # produces a timeout FAIL instead of hanging the suite.
        box = {}

        def read():
            box["raw"] = self.proc.stdout.readline()

        t = threading.Thread(target=read, daemon=True)
        t.start()
        t.join(self.timeout)
        if t.is_alive():
            self.proc.kill()
            raise RuntimeError(f"no reply within {self.timeout}s")
        raw = box.get("raw") or b""
        if not raw:
            rc = self.proc.poll()
            err = self.proc.stderr.read().decode("utf-8", "replace") if rc is not None else ""
            raise RuntimeError(f"worker closed stdout (exit={rc}) stderr={err!r}")
        return json.loads(raw.decode("utf-8"))

    def roundtrip(self, envelope):
        t0 = time.perf_counter()
        self.send(envelope)
        reply = self.recv()
        return reply, (time.perf_counter() - t0) * 1000.0

    def close(self):
        try:
            self.proc.stdin.close()
            self.proc.wait(timeout=self.timeout)
        except Exception:
            self.proc.kill()


def compile_envelope(ident, revision, text, path="main.tex", project="demo", version=1):
    return {
        "protocol_version": version,
        "id": ident,
        "type": "compile",
        "payload": {
            "project_id": project,
            "revision": revision,
            "entry_path": path,
            "documents": [{"path": path, "text": text}],
        },
    }


def text_items(result):
    items = []
    for page in result["payload"]["pages"]:
        for item in page["items"]:
            if item.get("kind") == "text":
                items.append(item)
    return items


def signature(items):
    return [(i["text"], i["source"]["path"], i["source"]["start_byte"], i["source"]["end_byte"]) for i in items]


class Report:
    def __init__(self):
        self.rows = []  # (status, name, detail)

    def add(self, status, name, detail=""):
        self.rows.append((status, name, detail))
        print(f"[{status}] {name}" + (f" -- {detail}" if detail else ""))

    @property
    def failed(self):
        return any(s == FAIL for s, _, _ in self.rows)


def check_fixture(worker, repo, report):
    req_path = os.path.join(repo, "protocol", "fixtures", "compile-request.json")
    res_path = os.path.join(repo, "protocol", "fixtures", "compile-result.json")
    with open(req_path, encoding="utf-8") as f:
        request = json.load(f)
    with open(res_path, encoding="utf-8") as f:
        expected = json.load(f)

    reply, ms = worker.roundtrip(request)
    if reply.get("type") != "compile_result":
        report.add(FAIL, "fixture: reply type", f"expected compile_result, got {reply!r}")
        return
    if reply.get("id") != request["id"]:
        report.add(FAIL, "fixture: id preserved", f"{reply.get('id')!r} != {request['id']!r}")
    else:
        report.add(PASS, "fixture: id preserved", request["id"])
    for key in ("project_id", "revision", "status"):
        if reply["payload"].get(key) != expected["payload"][key]:
            report.add(FAIL, f"fixture: payload.{key}", f"{reply['payload'].get(key)!r} != {expected['payload'][key]!r}")
        else:
            report.add(PASS, f"fixture: payload.{key}", repr(expected["payload"][key]))

    src = request["payload"]["documents"][0]["text"].encode("utf-8")
    got, want = text_items(reply), text_items(expected)
    got_sig, want_sig = signature(got), signature(want)

    # The fixture is a contract example (owner: Commander, FT-001). Check that
    # its own ranges slice back to its texts before using it as an oracle.
    fixture_bad = [(t, s, e, src[s:e].decode("utf-8", "replace")) for t, _, s, e in want_sig
                   if src[s:e].decode("utf-8", "replace") != t]
    if fixture_bad:
        report.add(FAIL, "fixture: protocol/fixtures/compile-result.json ranges slice to their texts (owner: Commander)",
                   f"(text, start, end, actual slice)={fixture_bad}; request text={src!r}")
    else:
        report.add(PASS, "fixture: protocol/fixtures/compile-result.json ranges slice to their texts", repr(want_sig))

    # Compiler items: every range must slice to its text, and the words joined
    # must equal the fixture's text. Segmentation (one item per word versus one
    # per line) is reported as INFO, not FAIL.
    compiler_bad = [(t, s, e, src[s:e].decode("utf-8", "replace")) for t, _, s, e in got_sig
                    if src[s:e].decode("utf-8", "replace") != t]
    joined_got = " ".join(i["text"] for i in got)
    joined_want = " ".join(i["text"] for i in want)
    if got_sig == want_sig:
        report.add(PASS, "fixture: compiler item texts and source ranges", f"{len(got)} item(s) identical to fixture")
    elif not compiler_bad and joined_got == joined_want:
        report.add(PASS, "fixture: compiler item texts and source ranges",
                   f"joined text {joined_got!r} equals fixture text and every compiler range slices to its text")
        report.add(INFO, "fixture: segmentation differs from fixture",
                   f"fixture items={want_sig} compiler items={got_sig}")
    else:
        report.add(FAIL, "fixture: compiler item texts and source ranges",
                   f"expected text {joined_want!r}, got {joined_got!r}; mis-sliced compiler items={compiler_bad}")
    pos_got = [(i["x_pt"], i["baseline_y_pt"], i["font_size_pt"]) for i in got]
    pos_want = [(i["x_pt"], i["baseline_y_pt"], i["font_size_pt"]) for i in want]
    if pos_got != pos_want:
        report.add(INFO, "fixture: positions differ (allowed)", f"fixture={pos_want} compiler={pos_got}")
    else:
        report.add(INFO, "fixture: positions identical", repr(pos_got))
    page_got = [(p["number"], p["width_pt"], p["height_pt"]) for p in reply["payload"]["pages"]]
    page_want = [(p["number"], p["width_pt"], p["height_pt"]) for p in expected["payload"]["pages"]]
    report.add(PASS if page_got == page_want else INFO, "fixture: page geometry", f"fixture={page_want} compiler={page_got}")
    diags = reply["payload"].get("diagnostics")
    report.add(PASS if diags == [] else FAIL, "fixture: no diagnostics", repr(diags))
    report.add(PASS if reply["payload"].get("pdf_path", "missing") is None else INFO,
               "fixture: pdf_path is null", repr(reply["payload"].get("pdf_path", "missing")))
    report.add(INFO, "fixture: round trip latency", f"{ms:.2f} ms")


def check_unicode(worker, report):
    text = "naïve café — 😀 end\n"
    data = text.encode("utf-8")
    reply, _ = worker.roundtrip(compile_envelope("unicode-1", 1, text))
    if reply.get("type") != "compile_result":
        report.add(FAIL, "unicode: reply type", repr(reply))
        return
    items = text_items(reply)
    if not items:
        report.add(FAIL, "unicode: items emitted", "no text items")
        return
    bad = []
    for item in items:
        s, e = item["source"]["start_byte"], item["source"]["end_byte"]
        if not (0 <= s < e <= len(data)):
            bad.append((item["text"], s, e, "out of range"))
            continue
        try:
            sliced = data[s:e].decode("utf-8")
        except UnicodeDecodeError as exc:
            bad.append((item["text"], s, e, f"not a scalar boundary: {exc}"))
            continue
        if sliced != item["text"]:
            bad.append((item["text"], s, e, f"slice={sliced!r}"))
    if bad:
        report.add(FAIL, "unicode: every span slices back to its text", repr(bad))
    else:
        report.add(PASS, "unicode: every span slices back to its text",
                   f"{len(items)} items: {[i['text'] for i in items]} "
                   f"spans={[(i['source']['start_byte'], i['source']['end_byte']) for i in items]}")
    expected_words = text.split()
    if [i["text"] for i in items] == expected_words:
        report.add(PASS, "unicode: all words present in order", repr(expected_words))
    else:
        report.add(INFO, "unicode: item texts differ from whitespace split",
                   f"items={[i['text'] for i in items]} words={expected_words}")
    # Byte offsets must be UTF-8, not code points or UTF-16 units: the last
    # word "end" starts after 4-byte and 3-byte scalars, so its start byte is
    # larger than its code-point index.
    last = items[-1]
    cp_index = text.index(last["text"])
    if last["source"]["start_byte"] > cp_index:
        report.add(PASS, "unicode: offsets are UTF-8 bytes, not code points",
                   f"{last['text']!r} start_byte={last['source']['start_byte']} > code point index {cp_index}")
    else:
        report.add(FAIL, "unicode: offsets are UTF-8 bytes, not code points",
                   f"{last['text']!r} start_byte={last['source']['start_byte']} code point index {cp_index}")


def check_revision_order(worker, report):
    worker.send(compile_envelope("rev-2", 2, "Second revision text\n"))
    worker.send(compile_envelope("rev-1", 1, "First revision text\n"))
    r_a = worker.recv()
    r_b = worker.recv()
    pairs = {r.get("id"): r for r in (r_a, r_b)}
    for ident, rev in (("rev-2", 2), ("rev-1", 1)):
        r = pairs.get(ident)
        if r is None or r.get("type") != "compile_result" or r["payload"].get("revision") != rev:
            report.add(FAIL, f"revision: {ident} echoes revision {rev}", repr(r))
        else:
            report.add(PASS, f"revision: {ident} echoes revision {rev}", f"revision={r['payload']['revision']}")
    report.add(INFO, "revision: reply order", f"{[r_a.get('id'), r_b.get('id')]} "
               "(worker is sequential; the UI stale guard is covered by swift test WorkerClientTests/ShellModelTests)")


def check_source_editing(worker, report):
    a, _ = worker.roundtrip(compile_envelope("edit-a", 3, "Alpha beta gamma.\n"))
    b, _ = worker.roundtrip(compile_envelope("edit-b", 4, "Alpha beta gamma delta.\n\nNew paragraph here.\n"))
    ia, ib = signature(text_items(a)), signature(text_items(b))
    if ia and ib and ia != ib:
        report.add(PASS, "editing: different sources produce different items", f"{len(ia)} vs {len(ib)} items")
    else:
        report.add(FAIL, "editing: different sources produce different items", f"a={ia} b={ib}")
    c, _ = worker.roundtrip(compile_envelope("edit-c", 5, "Alpha beta gamma.\n"))
    ic = signature(text_items(c))
    if ic == ia:
        report.add(PASS, "editing: same source is deterministic", f"{len(ic)} items identical to first compile")
    else:
        report.add(FAIL, "editing: same source is deterministic", f"a={ia} c={ic}")
    d, _ = worker.roundtrip(compile_envelope("edit-d", 6, "Text with \\unknowncmd{x} and $math$.\n"))
    diags = d["payload"].get("diagnostics", []) if d.get("type") == "compile_result" else None
    if diags:
        report.add(PASS, "editing: unsupported input is diagnosed, not silent",
                   f"status={d['payload'].get('status')} diagnostics={[x['message'] for x in diags]}")
    else:
        report.add(FAIL, "editing: unsupported input is diagnosed, not silent", repr(d))


def check_errors(worker, report):
    r, _ = worker.roundtrip(compile_envelope("v2", 1, "x\n", version=2))
    if r.get("type") == "error" and r.get("id") == "v2":
        report.add(PASS, "error: protocol_version 2 rejected",
                   f"code={r['payload'].get('code')} message={r['payload'].get('message')!r}")
    else:
        report.add(FAIL, "error: protocol_version 2 rejected", repr(r))
    r, _ = worker.roundtrip({"protocol_version": 1, "id": "unk", "type": "frobnicate", "payload": {}})
    if r.get("type") == "error" and r.get("id") == "unk":
        report.add(PASS, "error: unknown type rejected",
                   f"code={r['payload'].get('code')} message={r['payload'].get('message')!r}")
    else:
        report.add(FAIL, "error: unknown type rejected", repr(r))
    r, _ = worker.roundtrip(compile_envelope("trav", 1, "x\n", path="../evil.tex"))
    if r.get("type") == "error":
        report.add(PASS, "error: parent traversal path rejected", f"code={r['payload'].get('code')}")
    else:
        report.add(INFO, "error: parent traversal path not rejected as error envelope", repr(r)[:300])
    worker.send_raw(b"this is not json\n")
    r = worker.recv()
    if r.get("type") == "error":
        report.add(PASS, "error: malformed JSON rejected", f"code={r['payload'].get('code')}")
    else:
        report.add(FAIL, "error: malformed JSON rejected", repr(r))
    # The worker must still be alive and usable after the error paths.
    r, _ = worker.roundtrip(compile_envelope("after-errors", 7, "Still alive.\n"))
    report.add(PASS if r.get("type") == "compile_result" else FAIL,
               "error: worker keeps serving after error envelopes", r.get("type", repr(r)))


def check_latency(worker, report, count):
    text = ("\\section{Latency}\nThe quick brown fox jumps over the lazy dog. " * 8
            + "\n\nSecond paragraph with naïve café text.\n")
    samples = []
    for n in range(count):
        r, ms = worker.roundtrip(compile_envelope(f"lat-{n}", 100 + n, text))
        if r.get("type") != "compile_result":
            report.add(FAIL, "latency: compile_result reply", repr(r))
            return None
        samples.append(ms)
    stats = {
        "count": count,
        "min_ms": min(samples),
        "median_ms": statistics.median(samples),
        "max_ms": max(samples),
        "document_bytes": len(text.encode("utf-8")),
    }
    report.add(INFO, "latency: edit-to-result over %d requests" % count,
               "min=%.2f ms median=%.2f ms max=%.2f ms (doc %d bytes; subprocess pipe round trip, no UI)" % (
                   stats["min_ms"], stats["median_ms"], stats["max_ms"], stats["document_bytes"]))
    return stats


def main():
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--compiler", required=True, help="path to the built flashtex-compiler binary")
    ap.add_argument("--repo", required=True, help="repository root containing protocol/fixtures")
    ap.add_argument("--latency-requests", type=int, default=20)
    ap.add_argument("--timeout", type=float, default=10.0)
    ap.add_argument("--json", help="write machine-readable results here")
    args = ap.parse_args()

    if not os.access(args.compiler, os.X_OK):
        print(f"[FAIL] compiler binary not executable: {args.compiler}")
        return 2

    report = Report()
    worker = Worker(args.compiler, args.timeout)
    latency = None
    try:
        check_fixture(worker, args.repo, report)
        check_unicode(worker, report)
        check_revision_order(worker, report)
        check_source_editing(worker, report)
        check_errors(worker, report)
        latency = check_latency(worker, report, args.latency_requests)
    except Exception as exc:  # any transport failure is a FAIL, not a crash
        report.add(FAIL, "transport", f"{type(exc).__name__}: {exc}")
    finally:
        worker.close()

    counts = {s: sum(1 for r in report.rows if r[0] == s) for s in (PASS, FAIL, INFO)}
    print(f"\nsummary: {counts[PASS]} PASS, {counts[FAIL]} FAIL, {counts[INFO]} INFO")
    if args.json:
        with open(args.json, "w", encoding="utf-8") as f:
            json.dump({"compiler": args.compiler, "rows": report.rows, "counts": counts, "latency": latency}, f, indent=2)
    return 1 if report.failed else 0


if __name__ == "__main__":
    sys.exit(main())
