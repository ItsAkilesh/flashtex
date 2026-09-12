#!/usr/bin/env python3
"""Test double for a user-enabled explanation provider command
(`FLASHTEX_ASSISTANT_PROVIDER`): reads one prepared `PromptPayload` JSON on
stdin, writes one response JSON (`context_id`, `explanation`, `edits`) on
stdout, exits. No network, no model. It proposes one edit inside the first
allowed destination (so the helper's `validate` accepts it) and nothing when
the context is explanation-only.

Directives anywhere in the supplied snippet text or user instruction:
`%slowprovider` -> reply after 600 ms; `%hangprovider` -> reply after 30 s;
`%crashprovider` -> exit 4 with no output; `%garbageprovider` -> non-JSON output;
`%hugeprovider` -> a reply above the 64 KiB response limit; `%envprovider` -> the
explanation lists the FLASHTEX_* variable names visible to the provider;
`%badprovider` -> wrong context id; `%outsideprovider` -> an edit outside the
allowed destination (both must be refused by the helper's validation);
`%noeditprovider` -> explanation only.
"""
import json
import sys
import time

payload = json.load(sys.stdin)
snippets = [d["snippet"] for d in payload["diagnostics"] if d.get("snippet")] + payload.get("related", [])
text = "".join(s["text"] for s in snippets) + payload["user_instruction"]
if "%hangprovider" in text:
    time.sleep(30)
if "%slowprovider" in text:
    time.sleep(0.6)
if "%crashprovider" in text:
    sys.exit(4)
if "%garbageprovider" in text:
    print("this is not json")
    sys.exit(0)
if "%hugeprovider" in text:
    sys.stdout.write(json.dumps({"context_id": payload["context_id"], "explanation": "x" * (65 * 1024), "edits": []}))
    sys.exit(0)

context_id = "0" * 64 if "%badprovider" in text else payload["context_id"]


def removed(location):
    for s in snippets:
        sl = s["location"]
        if sl["path"] == location["path"] and location["start_byte"] >= sl["start_byte"] and location["end_byte"] <= sl["end_byte"]:
            data = s["text"].encode("utf-8")
            return data[location["start_byte"] - sl["start_byte"]:location["end_byte"] - sl["start_byte"]].decode("utf-8")
    return None


edits = []
allowed = payload.get("allowed_edits") or []
if "%outsideprovider" in text and snippets:
    sl = snippets[0]["location"]
    loc = {"path": sl["path"], "start_byte": sl["start_byte"], "end_byte": sl["start_byte"] + 1}
    edits.append({"location": loc, "removed_text": removed(loc), "replacement": "X"})
elif allowed and "%noeditprovider" not in text:
    r = allowed[0]
    loc = {"path": r["path"], "start_byte": r["start_byte"], "end_byte": r["end_byte"]}
    old = removed(loc)
    if old is not None:
        edits.append({"location": loc, "removed_text": old, "replacement": "% reviewed: " + old})

explanation = "Fake explanation of %d diagnostic(s) for context %s." % (len(payload["diagnostics"]), payload["context_id"][:8])
if "%envprovider" in text:
    import os
    explanation += " env: " + ",".join(sorted(k for k in os.environ if k.startswith("FLASHTEX_")))
print(json.dumps({"context_id": context_id, "explanation": explanation, "edits": edits}))
