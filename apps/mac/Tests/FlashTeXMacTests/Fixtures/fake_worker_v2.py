#!/usr/bin/env python3
"""Test double for a runtime-v1 producer that speaks the negotiated
`display-list-v2` capability (docs/contracts/runtime-v1-display-list-v2.md).
Not a compiler: the compile_result carries one text item for the first line of
the entry text; the display_list sibling line is the rendering-v2 envelope given
as argv[1] (a real `flashtex-render --v2` fixture) rebound to the request: same
`id`, `project_id`, `revision`, and the entry document's revision, SHA-256 and
byte length.

Directives at the start of the entry text (test-only):
  %v2nocap       behave like an old producer: never accept the capability, no line
  %v2failed      status failed, capability accepted, NO display_list line
  %v2stale       the display_list line carries id "never-sent" (stale/unknown)
  %v2mismatch    the display_list payload claims revision + 1000 (correlation mismatch)
  %v2unsolicited emit the line WITHOUT echoing acceptance (protocol violation)
  %v2decline     decline per request: not echoed, warning diagnostic, no line
"""
import hashlib
import json
import sys

CAP = "display-list-v2"
template = json.load(open(sys.argv[1], encoding="utf-8"))

for raw in sys.stdin:
    raw = raw.strip()
    if not raw:
        continue
    env = json.loads(raw)
    if env.get("protocol_version") != 1 or env.get("type") != "compile":
        print(json.dumps({"protocol_version": 1, "id": env.get("id", "?"), "type": "error",
                          "payload": {"message": "unsupported envelope"}}), flush=True)
        continue
    p = env["payload"]
    entry = next((d for d in p["documents"] if d["path"] == p["entry_path"]), None)
    text = entry["text"] if entry else ""
    directive = text.split("\n", 1)[0] if text.startswith("%v2") else ""
    requested = p.get("layout_capabilities") or []
    accept = CAP in requested and directive not in ("%v2nocap", "%v2unsolicited", "%v2decline")
    status = "failed" if directive == "%v2failed" else "ok"
    first = text.split("\n", 1)[0]
    result = {"project_id": p["project_id"], "revision": p["revision"], "status": status,
              "pages": [] if status == "failed" else [{"number": 1, "width_pt": 612, "height_pt": 792, "items": [
                  {"kind": "text", "text": first, "x_pt": 72, "baseline_y_pt": 84, "font_size_pt": 12,
                   "source": {"path": p["entry_path"], "start_byte": 0, "end_byte": len(first.encode("utf-8"))}}]}],
              "diagnostics": [], "pdf_path": None}
    if directive == "%v2failed":
        result["diagnostics"] = [{"severity": "error", "message": "requested failure", "source": None, "recovery": None}]
    if directive == "%v2decline" and CAP in requested:
        result["diagnostics"] = [{"severity": "warning", "message": "display-list-v2 declined: envelope would exceed the line limit (test)", "source": None, "recovery": None}]
    if accept:
        result["layout_capabilities"] = [CAP]
    print(json.dumps({"protocol_version": 1, "id": env["id"], "type": "compile_result", "payload": result}), flush=True)
    emit_line = (accept and status != "failed") or directive == "%v2unsolicited"
    if not emit_line:
        continue
    v2 = json.loads(json.dumps(template))
    v2["id"] = "never-sent" if directive == "%v2stale" else env["id"]
    payload = v2["payload"]
    payload["project_id"] = p["project_id"]
    payload["revision"] = p["revision"] + (1000 if directive == "%v2mismatch" else 0)
    data = text.encode("utf-8")
    for doc in payload["documents"]:
        if doc["path"] == p["entry_path"]:
            doc["revision"] = p["revision"]
            doc["sha256"] = hashlib.sha256(data).hexdigest()
            doc["byte_length"] = len(data)
    print(json.dumps(v2, ensure_ascii=False), flush=True)
