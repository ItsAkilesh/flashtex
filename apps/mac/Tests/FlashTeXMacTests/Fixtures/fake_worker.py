#!/usr/bin/env python3
"""Test double for the Rust worker: runtime v1 JSON Lines over stdin/stdout.

Echoes each `compile` as a `compile_result` whose single text item maps the
first line of the entry document. Not a compiler. Supports a few directives in
the entry text for tests: `%error` -> error envelope, `%garbage` -> non-JSON line.
`%diag:<n>` (anywhere in the entry text, any number of times) -> one error
diagnostic per occurrence, spanning byte `pos+n ..< pos+n+1` where `pos` is the
directive's own UTF-8 byte offset, so a diagnostic after an insertion shifts with
the text the way a real compiler's would. `%slow` at the start -> the reply is
delayed 400 ms (for coalescing tests). Test double only.
"""
import json
import re
import sys
import time

DIAG = re.compile(rb"%diag:(\d+)")

for raw in sys.stdin:
    raw = raw.strip()
    if not raw:
        continue
    env = json.loads(raw)
    if env.get("protocol_version") != 1 or env.get("type") != "compile":
        out = {"protocol_version": 1, "id": env.get("id", "?"), "type": "error",
               "payload": {"message": "unsupported envelope"}}
        print(json.dumps(out), flush=True)
        continue
    p = env["payload"]
    entry = next((d for d in p["documents"] if d["path"] == p["entry_path"]), None)
    text = entry["text"] if entry else ""
    if text.startswith("%error"):
        print(json.dumps({"protocol_version": 1, "id": env["id"], "type": "error",
                          "payload": {"message": "requested failure"}}), flush=True)
        continue
    if text.startswith("%garbage"):
        print("this is not json", flush=True)
        continue
    if text.startswith("%huge"):
        # One complete line larger than the shell's 16 MiB cap.
        sys.stdout.write('{"protocol_version":1,"id":"%s","type":"compile_result","payload":{"x":"' % env["id"])
        sys.stdout.write("x" * (16 * 1024 * 1024 + 64))
        sys.stdout.write('"}}\n')
        sys.stdout.flush()
        continue
    if text.startswith("%trailing"):
        # Partial JSON with no newline, then exit: unterminated bytes at EOF.
        sys.stdout.write('{"protocol_version":1,"id":"%s","type":"compile_result","pay' % env["id"])
        sys.stdout.flush()
        sys.exit(0)
    if text.startswith("%wrongid"):
        # Valid-looking result that answers a request nobody sent.
        env["id"] = "never-sent"
        text = text[len("%wrongid"):]
    if text.startswith("%wrongrev"):
        p["revision"] = p["revision"] + 1000
        text = text[len("%wrongrev"):]
    if text.startswith("%slow"):
        time.sleep(0.4)
    first = text.split("\n", 1)[0]
    end = len(first.encode("utf-8"))
    diagnostics = []
    for m in DIAG.finditer(text.encode("utf-8")):
        at = m.start() + int(m.group(1))
        diagnostics.append({"severity": "error", "message": "fake diagnostic %s" % m.group(1).decode(),
                            "source": {"path": p["entry_path"], "start_byte": at, "end_byte": at + 1},
                            "recovery": "test double: byte skipped"})
    print("fake_worker: compiling revision %d" % p["revision"], file=sys.stderr, flush=True)
    out = {"protocol_version": 1, "id": env["id"], "type": "compile_result",
           "payload": {"project_id": p["project_id"], "revision": p["revision"], "status": "ok",
                       "pages": [{"number": 1, "width_pt": 612, "height_pt": 792,
                                  "items": [{"kind": "text", "text": first, "x_pt": 72,
                                             "baseline_y_pt": 84, "font_size_pt": 12,
                                             "source": {"path": p["entry_path"],
                                                        "start_byte": 0, "end_byte": end}}]}],
                       "diagnostics": diagnostics, "pdf_path": None}}
    print(json.dumps(out), flush=True)
