#!/usr/bin/env python3
"""Test double for the Rust worker: runtime v1 JSON Lines over stdin/stdout.

Echoes each `compile` as a `compile_result` whose single text item maps the
first line of the entry document. Not a compiler. Supports a few directives in
the entry text for tests: `%error` -> error envelope, `%garbage` -> non-JSON line.
"""
import json
import sys

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
    first = text.split("\n", 1)[0]
    end = len(first.encode("utf-8"))
    print("fake_worker: compiling revision %d" % p["revision"], file=sys.stderr, flush=True)
    out = {"protocol_version": 1, "id": env["id"], "type": "compile_result",
           "payload": {"project_id": p["project_id"], "revision": p["revision"], "status": "ok",
                       "pages": [{"number": 1, "width_pt": 612, "height_pt": 792,
                                  "items": [{"kind": "text", "text": first, "x_pt": 72,
                                             "baseline_y_pt": 84, "font_size_pt": 12,
                                             "source": {"path": p["entry_path"],
                                                        "start_byte": 0, "end_byte": end}}]}],
                       "diagnostics": [], "pdf_path": None}}
    print(json.dumps(out), flush=True)
