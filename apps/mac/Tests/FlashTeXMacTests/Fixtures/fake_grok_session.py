#!/usr/bin/env python3
"""Test double for `flashtex-assistant-context --provider-session SESSION MODEL`
(crates/assistant-context README "provider commands", session.rs /
provider_session.rs at the grok feature): JSON Lines on stdin/stdout, replies
`{"id":..,"result":{..}}` or `{"id":..,"error":".."}`. Same gates as the real
helper: exactly those two arguments (else usage, exit 2) and a non-empty
`FLASHTEX_GROK_API_KEY` in the environment (else "explicit provider credential
missing", exit 2). The key's VALUE is never written anywhere; the fake reports
only its presence. No network, no model.

`admit` accepts a `prepare` input and computes the same context id as
`fake_assistant_context.py`, so the `validated_proposal` it later returns passes
that double's `validate`/`review`. `poll` reports `running` once, then the
proposal (one edit inside the first destination, or none). Directives anywhere
in the bound source text: `%grokfail` -> state `failed`; `%grokslow` -> running
for three polls; `%grokhang` -> running forever; `%grokexpired` -> `expired`;
`%groksnapshot` -> the explanation lists the FLASHTEX_* names visible plus
whether the key variable was present; `%grokmisplaced` -> the edit's offsets
are shifted by 2 and handed back unvalidated (what a non-reasoning model does;
`fake_assistant_context.py` then refuses the review with "removed source
differs", as the pre-relocation real helper did); `%groknoedit` -> no edits.
"""
import hashlib
import json
import os
import sys

args = sys.argv[1:]
if len(args) != 3 or args[0] != "--provider-session":
    sys.stderr.write("usage: flashtex-assistant-context [--session FRESH_SESSION_ID]\n")
    sys.exit(2)
session, model = args[1], args[2]
if model == "no-grok-feature":  # stands in for a helper built without the feature
    sys.stderr.write("usage: flashtex-assistant-context [--session FRESH_SESSION_ID]\n")
    sys.exit(2)
if not os.environ.get("FLASHTEX_GROK_API_KEY"):
    sys.stderr.write("explicit provider credential missing\n")
    sys.exit(2)

jobs = {}
counter = 0


def sha256(text):
    return hashlib.sha256(text.encode("utf-8")).hexdigest()


def reply(cid, result=None, error=None):
    line = {"id": cid, "result": result} if error is None else {"id": cid, "error": error}
    sys.stdout.write(json.dumps(line) + "\n")
    sys.stdout.flush()


def admit(command):
    global counter
    inp = command["input"]
    if inp.get("operation") != "prepare" or "response" in inp or "current_sources" in inp:
        raise ValueError("admit requires prepare input")
    if not command.get("user_requested") or not command.get("allocation"):
        raise ValueError("explicit bounded usage intent required")
    binding = inp["binding"]
    diags = inp["compiler_result"]["payload"]["diagnostics"]
    selected = inp.get("selected_diagnostics")
    if selected is None:
        selected = list(range(min(16, len(diags))))
    dest = inp.get("destinations")
    context_id = sha256(json.dumps([binding, selected, dest, inp["user_instruction"], inp.get("related_paths") or []], sort_keys=True))
    text = "".join(d["text"] for d in inp["sources"])
    counter += 1
    rid = "%s:%d" % (session, counter)
    jobs[rid] = {"context_id": context_id, "text": text, "dest": dest, "polls": 0, "sources": inp["sources"],
                 "diagnostics": len(selected)}
    return {"type": "provider_admitted", "request_id": rid}


def proposal(job):
    edits = []
    if job["dest"] and "%groknoedit" not in job["text"]:
        r = job["dest"][0]
        doc = next(d for d in job["sources"] if d["path"] == r["path"])
        data = doc["text"].encode("utf-8")
        loc = dict(r)
        if "%grokmisplaced" in job["text"]:
            loc = {"path": r["path"], "start_byte": r["start_byte"] + 2, "end_byte": r["end_byte"] + 2}
        edits.append({"location": loc, "removed_text": data[r["start_byte"]:r["end_byte"]].decode("utf-8"),
                      "replacement": "% grok-reviewed: " + data[r["start_byte"]:r["end_byte"]].decode("utf-8")})
    explanation = "Fake Grok (%s) explanation of %d diagnostic(s)" % (model, job["diagnostics"])
    if "%groksnapshot" in job["text"]:
        names = sorted(k for k in os.environ if k.startswith("FLASHTEX_"))
        explanation += " env: " + ",".join(names) + " key:" + ("present" if os.environ.get("FLASHTEX_GROK_API_KEY") else "absent")
    return {"context_id": job["context_id"], "explanation": explanation, "edits": edits}


def poll(command):
    rid = command["request_id"]
    job = jobs.get(rid)
    if job is None:
        raise ValueError("unknown provider request")
    for d in command["current_sources"]:
        if sha256(d["text"]) != d["source_sha256"]:
            raise ValueError("source snapshot is stale")
    job["polls"] += 1
    text = job["text"]
    if "%grokfail" in text:
        return {"type": "provider_status", "request_id": rid, "state": "failed"}
    if "%grokexpired" in text:
        return {"type": "provider_status", "request_id": rid, "state": "expired"}
    if "%grokhang" in text or ("%grokslow" in text and job["polls"] <= 3) or job["polls"] <= 1:
        return {"type": "provider_status", "request_id": rid, "state": "running" if job["polls"] > 1 else "queued"}
    del jobs[rid]
    return {"type": "validated_proposal", "request_id": rid, "payload": proposal(job), "applied": False}


for raw in sys.stdin:
    raw = raw.strip()
    if not raw:
        continue
    try:
        frame = json.loads(raw)
        cid = frame["id"]
        action = frame["action"]
        if action.get("operation") != "provider":
            raise ValueError("only provider actions are supported by this double")
        command = action["command"]
        op = command.get("operation")
        if op == "admit":
            reply(cid, admit(command))
        elif op == "poll":
            reply(cid, poll(command))
        elif op == "cancel":
            jobs.pop(command["request_id"], None)
            reply(cid, {"type": "provider_cancelled", "request_id": command["request_id"]})
        elif op == "retire":
            jobs.pop(command["request_id"], None)
            reply(cid, {"type": "provider_retired", "request_id": command["request_id"]})
        elif op == "snapshot":
            reply(cid, {"type": "provider_snapshot", "payload": {"executing": 0, "jobs": [], "provider_billing_known": False,
                                                                 "queued": len(jobs), "retained": len(jobs), "scheduler_tasks_started": counter}})
        else:
            raise ValueError("invalid provider command")
    except (ValueError, KeyError) as e:
        reply(frame.get("id") if isinstance(frame, dict) else None, error=str(e) or "invalid provider command")
