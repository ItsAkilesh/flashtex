#!/usr/bin/env python3
"""Delaying stdio proxy in front of the REAL `flashtex-assistant-context`
helper (ReviewRecoveryTests.swift). The helper is a one-shot JSON transform
that answers in milliseconds, so a test cannot catch a request at a specific
helper stage (probe/prepare/validate/review/approve) to cancel it, kill it or
inject an out-of-order reply there. This proxy reads the one request on
stdin, sleeps for the delay configured for that request's `operation` in a
CONTROL FILE (JSON, argv[2]), then runs the real helper (argv[1]) on the
unchanged request and relays its stdout and exit status byte for byte. No
network, no model, no rewriting.

  {"delay_s": {"prepare": 0.8, "review": 0.8},   # per stage hint (seconds); exact key only
   "pid_file": "<path>",                          # this pid, at start
   "log_file": "<path>"}                          # one line per run: <pid> <operation> <stage-hint>

`stage-hint` is `probe` for a prepare request without `destinations` (the
app's first prepare call) and the operation otherwise, so a test can see the
exact sequence of helper launches. SIGTERM during the sleep ends the proxy
before the helper ever starts (nothing was computed); SIGTERM during the
helper run kills the helper too.
"""
import json
import os
import signal
import subprocess
import sys
import time

helper = sys.argv[1]
control_path = sys.argv[2] if len(sys.argv) > 2 else None
control = {}
if control_path and os.path.exists(control_path):
    with open(control_path) as f:
        control = json.load(f)

raw = sys.stdin.buffer.read()
try:
    req = json.loads(raw)
except ValueError:
    req = {}
op = req.get("operation", "?") if isinstance(req, dict) else "?"
hint = "probe" if op == "prepare" and "destinations" not in req else op

if control.get("pid_file"):
    with open(control["pid_file"], "w") as f:
        f.write(str(os.getpid()))
if control.get("log_file"):
    with open(control["log_file"], "a") as f:
        f.write("%d %s %s\n" % (os.getpid(), op, hint))

delays = control.get("delay_s") or {}
delay = float(delays.get(hint, 0))
if delay > 0:
    time.sleep(delay)

child = subprocess.Popen([helper], stdin=subprocess.PIPE, stdout=subprocess.PIPE)


def on_term(signum, frame):
    try:
        child.kill()
    except OSError:
        pass
    sys.exit(128 + signum)


signal.signal(signal.SIGTERM, on_term)
out, _ = child.communicate(raw)
sys.stdout.buffer.write(out)
sys.stdout.buffer.flush()
sys.exit(child.returncode)
