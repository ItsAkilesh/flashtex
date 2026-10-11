#!/usr/bin/env python3
"""A host's start-up: process start to its HELLO reply (M3's "flashtex-host
size and start-up unchanged", and PROPOSAL.md §5.1's per-binary numbers).

    python3 tools/unicode-m3/host_startup.py --host BIN [--runs 5] [--json OUT] [-- HOST-ARGS...]

Starts the host on a socket of its own, connects as soon as the socket
accepts, sends the client HELLO (display-list-v3, version 3.3), reads the
host's HELLO and sends BYE; the host then exits. Reported per run: wall
time to the HELLO, the host's exit status, and its user-space instructions
retired over the whole run (`perf stat`, when it counts here). Also the
binary's size, its stripped size and its SHA-256.
"""
import argparse
import hashlib
import json
import os
import re
import shutil
import socket
import statistics
import struct
import subprocess
import sys
import tempfile
import time

C_HELLO, BYE = 0x01, 0x04


def frame(kind, body):
    return struct.pack("<I", len(body) + 1) + bytes([kind]) + body


def read_frame(s):
    head = b""
    while len(head) < 5:
        b = s.recv(5 - len(head))
        if not b:
            return None, None
        head += b
    n = struct.unpack("<I", head[:4])[0] - 1
    body = b""
    while len(body) < n:
        b = s.recv(n - len(body))
        if not b:
            break
        body += b
    return head[4], body


def perf_ok():
    if not shutil.which("perf"):
        return False
    r = subprocess.run(["perf", "stat", "-x,", "-e", "instructions:u", "true"],
                       stdout=subprocess.DEVNULL, stderr=subprocess.PIPE, text=True)
    return r.returncode == 0 and re.search(r"^\d+,", r.stderr, re.M) is not None


def one(host, args, tmp, i, perf):
    sock = os.path.join(tmp, "s%d.sock" % i)
    pout = os.path.join(tmp, "p%d.txt" % i)
    cmd = [host, "--socket", sock] + args
    if perf:
        cmd = ["perf", "stat", "-x,", "-e", "instructions:u", "-o", pout, "--"] + cmd
    t0 = time.time()
    p = subprocess.Popen(cmd, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
    s = None
    while time.time() - t0 < 120:
        try:
            s = socket.socket(socket.AF_UNIX)
            s.connect(sock)
            break
        except OSError:
            s.close()
            s = None
            time.sleep(0.001)
    if s is None:
        p.kill()
        return {"error": "no socket"}
    hello = json.dumps({"protocol": "display-list-v3", "version": [3, 3],
                        "client": "tools/unicode-m3/host_startup.py"}).encode()
    s.sendall(frame(C_HELLO, hello))
    k, body = read_frame(s)
    ms = (time.time() - t0) * 1000
    s.sendall(frame(BYE, b"{}"))
    s.close()
    try:
        code = p.wait(timeout=60)
    except subprocess.TimeoutExpired:
        p.terminate()
        code = p.wait()
    ins = None
    if perf and os.path.exists(pout):
        m = re.search(r"^(\d+),[^,]*,instructions", open(pout).read(), re.M)
        ins = int(m.group(1)) if m else None
    return {"hello_ms": round(ms, 2), "kind": k, "exit": code, "instructions": ins,
            "hello": (body or b"")[:300].decode("utf-8", "replace")}


def main(argv=None):
    ap = argparse.ArgumentParser(description=__doc__.split("\n")[0])
    ap.add_argument("--host", required=True)
    ap.add_argument("--runs", type=int, default=5)
    ap.add_argument("--json")
    ap.add_argument("args", nargs="*")
    a = ap.parse_args(argv)
    host = os.path.abspath(a.host)
    tmp = tempfile.mkdtemp(prefix="host-startup-")
    stripped = os.path.join(tmp, "stripped")
    shutil.copy(host, stripped)
    subprocess.run(["strip", stripped], check=False)
    perf = perf_ok()
    one(host, a.args, tmp, 0, False)  # page cache warm
    runs = [one(host, a.args, tmp, i + 1, perf) for i in range(a.runs)]
    ms = [r["hello_ms"] for r in runs if "hello_ms" in r]
    ins = [r["instructions"] for r in runs if r.get("instructions")]
    res = {"host": a.host, "size": os.path.getsize(host), "stripped_size": os.path.getsize(stripped),
           "sha256": hashlib.sha256(open(host, "rb").read()).hexdigest(),
           "hello_ms_median": statistics.median(ms) if ms else None,
           "hello_ms_min": min(ms) if ms else None,
           "instructions_median": int(statistics.median(ins)) if ins else None,
           "instructions_min": min(ins) if ins else None, "runs": runs}
    print(json.dumps({k: v for k, v in res.items() if k != "runs"}, indent=1))
    if a.json:
        with open(a.json, "w") as f:
            json.dump(res, f, indent=1)
    shutil.rmtree(tmp, ignore_errors=True)
    return 0 if ms and all(r.get("exit") == 0 for r in runs) else 1


if __name__ == "__main__":
    sys.exit(main())
