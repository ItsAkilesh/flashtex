#!/usr/bin/env python3
"""Deterministic local provider command for the assistant recovery tests
(AssistantRecoveryTests.swift). Like `fake_assistant_provider.py` it reads one
prepared `PromptPayload` on stdin and answers one response JSON (`context_id`,
`explanation`, one edit inside the first allowed destination) on stdout — no
network, no model, no credentials — but its behavior comes from a CONTROL FILE
(JSON) named by argv[1] (or `FLASHTEX_TEST_PROVIDER_CONTROL`), not from
directives in the text, so a test can change what the NEXT invocation does
between requests:

  {"delay_s": 0.0,          # sleep before answering
   "ignore_sigterm": false, # keep running through SIGTERM (a stubborn provider)
   "mode": "answer",        # "answer" | "exit" | "garbage" | "stale_context"
   "exit_code": 4,          # for mode "exit"
   "pid_file": "<path>",    # this process writes its pid here at start
   "log_file": "<path>"}    # one line per invocation: <pid> <context_id[:8]>

Every invocation appends to `log_file` so a test can count provider runs and
see which context each answered. Nothing else is written anywhere.
"""
import json
import os
import signal
import sys
import time

control_path = sys.argv[1] if len(sys.argv) > 1 else os.environ.get("FLASHTEX_TEST_PROVIDER_CONTROL")
control = {}
if control_path and os.path.exists(control_path):
    with open(control_path) as f:
        control = json.load(f)

payload = json.load(sys.stdin)
context_id = payload["context_id"]

if control.get("pid_file"):
    with open(control["pid_file"], "w") as f:
        f.write(str(os.getpid()))
if control.get("log_file"):
    with open(control["log_file"], "a") as f:
        f.write("%d %s\n" % (os.getpid(), context_id[:8]))
if control.get("ignore_sigterm"):
    signal.signal(signal.SIGTERM, signal.SIG_IGN)

delay = float(control.get("delay_s", 0))
if delay > 0:
    time.sleep(delay)

mode = control.get("mode", "answer")
if mode == "exit":
    sys.exit(int(control.get("exit_code", 4)))
if mode == "garbage":
    print("not json")
    sys.exit(0)

snippets = [d["snippet"] for d in payload["diagnostics"] if d.get("snippet")] + payload.get("related", [])


def removed(location):
    for s in snippets:
        sl = s["location"]
        if sl["path"] == location["path"] and location["start_byte"] >= sl["start_byte"] and location["end_byte"] <= sl["end_byte"]:
            data = s["text"].encode("utf-8")
            return data[location["start_byte"] - sl["start_byte"]:location["end_byte"] - sl["start_byte"]].decode("utf-8")
    return None


edits = []
allowed = payload.get("allowed_edits") or []
if allowed:
    r = allowed[0]
    loc = {"path": r["path"], "start_byte": r["start_byte"], "end_byte": r["end_byte"]}
    old = removed(loc)
    if old is not None:
        edits.append({"location": loc, "removed_text": old, "replacement": "% reviewed: " + old})

answer_context = "0" * 64 if mode == "stale_context" else context_id
explanation = "Controlled explanation of %d diagnostic(s) for context %s (pid %d)." % (
    len(payload["diagnostics"]), context_id[:8], os.getpid())
print(json.dumps({"context_id": answer_context, "explanation": explanation, "edits": edits}))
