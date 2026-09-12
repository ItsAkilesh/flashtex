#!/usr/bin/env python3
"""TEST DOUBLE for the FlashTeX project-files helper (crates/project-files,
`flashtex-project-files --root DIR`, protocol project-files-v1). NOT the real
helper: it speaks the same JSON Lines protocol (ping / read / status / save with
{outcome: saved|conflict}) with plain Python file I/O — no root binding, no
symlink refusal, no lock — so the Mac shell's handling of *lost and late
replies* can be tested hermetically.

Usage: fake_project_files.py [--mode M] [--delay S] [--exit N] [--once] --root DIR
(`--root` last, as the shell appends it). `--mode` selects the failure injected
for every operation except `ping`:
  answer  (default) reply normally
  hang    read the request, never answer, keep running
  exit    read the request, exit with --exit (default 3) without answering
  late    perform the operation, sleep --delay seconds (default 2), then
          answer (a reply that arrives after the shell's wait expired)
  garbage answer with a line that is not JSON
`--once` injects the failure for the first operation only.
"""
import hashlib
import json
import os
import sys
import time

args = sys.argv[1:]


def opt(name, default):
    return args[args.index(name) + 1] if name in args else default


MODE = opt("--mode", "answer")
ONCE = "--once" in args
DELAY = float(opt("--delay", "2"))
EXIT = int(opt("--exit", "3"))
root = args[args.index("--root") + 1]


def sha(b):
    return hashlib.sha256(b).hexdigest()


def stat(path):
    try:
        with open(path, "rb") as f:
            b = f.read()
    except FileNotFoundError:
        return None
    return b, sha(b), int(os.stat(path).st_mtime * 1000)


def handle(req):
    op = req.get("operation")
    if op == "ping":
        return {"protocol": "project-files-v1", "root": root, "pid": os.getpid()}
    rel = req.get("path")
    if not isinstance(rel, str) or rel.startswith("/") or ".." in rel.split("/"):
        raise ValueError(("invalid_path", "bad path %r" % rel))
    path = os.path.join(root, rel)
    if op == "read":
        s = stat(path)
        if s is None:
            return {"path": rel, "exists": False}
        b, h, m = s
        return {"path": rel, "exists": True, "text": b.decode("utf-8"), "sha256": h, "bytes": len(b), "mtime_unix_ms": m}
    if op == "status":
        exp = req.get("expected_sha256")
        s = stat(path)
        if s is None:
            return {"path": rel, "exists": False, "state": "deleted" if exp else "unchanged"}
        b, h, m = s
        state = "created" if exp is None else ("unchanged" if exp == h else "modified")
        return {"path": rel, "exists": True, "state": state, "sha256": h, "bytes": len(b), "mtime_unix_ms": m}
    if op == "save":
        text = req["text"].encode("utf-8")
        exp = req.get("expected")
        force = bool(req.get("force"))
        s = stat(path)
        if not force:
            kind = None
            if exp == "new" and s is not None:
                kind = "already_exists"
            elif exp not in ("new", "any"):
                if s is None:
                    kind = "deleted_externally"
                elif s[1] != exp:
                    kind = "modified_externally"
            if kind:
                return {"outcome": "conflict", "conflict": {
                    "path": rel, "kind": kind, "ours": None if exp in ("new", "any") else exp,
                    "theirs": s[1] if s else None, "mtime_unix_ms": s[2] if s else None, "size": len(s[0]) if s else None}}
        tmp = path + ".fake-tmp"
        with open(tmp, "wb") as f:
            f.write(text)
            f.flush()
            os.fsync(f.fileno())
        os.replace(tmp, path)
        return {"outcome": "saved", "receipt": {"path": rel, "bytes": len(text), "sha256": sha(text),
                                                "mtime_unix_ms": int(os.stat(path).st_mtime * 1000)}}
    raise ValueError(("unsupported_operation", "unknown operation %r" % op))


def reply(obj):
    sys.stdout.write(json.dumps(obj) + "\n")
    sys.stdout.flush()


injected = False
for line in sys.stdin:
    line = line.strip()
    if not line:
        continue
    try:
        req = json.loads(line)
    except ValueError:
        reply({"id": None, "error": {"code": "invalid_request", "message": "malformed JSON"}})
        continue
    rid = req.get("id")
    mode = MODE if req.get("operation") != "ping" and not (ONCE and injected) else "answer"
    if mode != "answer":
        injected = True
    if mode == "hang":
        while True:
            time.sleep(3600)
    if mode == "exit":
        sys.exit(EXIT)
    try:
        payload = handle(req)
    except ValueError as e:
        code, message = e.args[0]
        reply({"id": rid, "error": {"code": code, "message": message}})
        continue
    if mode == "late":
        time.sleep(DELAY)
    if mode == "garbage":
        sys.stdout.write("this is not json\n")
        sys.stdout.flush()
        continue
    reply({"id": rid, "payload": payload})
