#!/usr/bin/env python3
"""Capture latency and disconnect/retry against the real flashtex-bridge (stdlib).

Phase A (latency): one bridge process, document_open + destination_pin, then
N capture_submit envelopes with distinct capture_ids; wall time from the write
of each envelope to its complete capture_received line.
Phase B (disconnect/retry): fresh store; submit loop, SIGKILL the bridge after
K submits; record the client-side error; restart the bridge on the same store;
capture_status for the already-received ids must still be durable; resubmit
the interrupted id and the rest -> capture_received. Also records
capture_convert without --enable-grok -> provider_disabled.

Usage: rev5_bridge.py --bridge <bin> --fixture <capture-submission.json> --store <dir>
                      [--submits 10] [--kill-after 5] [--json out]
"""
import argparse
import json
import os
import shutil
import statistics
import subprocess
import sys
import time

PASS, FAIL, INFO = "PASS", "FAIL", "INFO"
rows = []


def add(s, n, d=""):
    rows.append((s, n, d))
    print(f"[{s}] {n}" + (f" -- {d}" if d else ""))


def env_line(i, t, p):
    return (json.dumps({"protocol_version": 1, "id": i, "type": t, "payload": p}) + "\n").encode()


class Bridge:
    def __init__(self, binary, store):
        env = dict(os.environ)
        env.pop("XAI_API_KEY", None)
        self.p = subprocess.Popen([binary, "--store", store], stdin=subprocess.PIPE, stdout=subprocess.PIPE,
                                  stderr=subprocess.PIPE, env=env)

    def call(self, line, timeout=10):
        t0 = time.perf_counter()
        self.p.stdin.write(line)
        self.p.stdin.flush()
        raw = self.p.stdout.readline()
        ms = (time.perf_counter() - t0) * 1000
        if not raw:
            raise RuntimeError(f"bridge closed stdout (exit {self.p.poll()})")
        return json.loads(raw), ms

    def kill(self):
        self.p.kill()
        self.p.wait()

    def close(self):
        try:
            self.p.stdin.close()
            self.p.wait(timeout=5)
        except Exception:
            self.p.kill()


def fresh(store):
    shutil.rmtree(store, ignore_errors=True)
    os.makedirs(store)


def open_and_pin(b, fixture):
    dest, rev = fixture["payload"]["destination_id"], fixture["payload"]["base_revision"]
    r, _ = b.call(env_line("open", "document_open", {"project_id": "p", "path": "main.tex", "revision": rev, "text": "Hello FlashTeX.\n"}))
    assert r.get("type") == "document_opened", r
    r, _ = b.call(env_line("pin", "destination_pin", {"destination_id": dest, "project_id": "p", "path": "main.tex",
                                                       "revision": rev, "start_byte": 5, "end_byte": 5}))
    assert r.get("type") == "destination_pinned", r


def submit_line(fixture, n):
    fx = json.loads(json.dumps(fixture))
    fx["id"] = f"submit-{n}"
    fx["payload"]["capture_id"] = f"rev5-capture-{n}"
    return fx["payload"]["capture_id"], (json.dumps(fx) + "\n").encode()


def main():
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--bridge", required=True)
    ap.add_argument("--fixture", required=True)
    ap.add_argument("--store", required=True)
    ap.add_argument("--submits", type=int, default=10)
    ap.add_argument("--kill-after", type=int, default=5)
    ap.add_argument("--json")
    args = ap.parse_args()
    fixture = json.load(open(args.fixture, encoding="utf-8"))
    out = {}

    # ---- Phase A: latency
    store_a = args.store + "-latency"
    fresh(store_a)
    b = Bridge(args.bridge, store_a)
    try:
        open_and_pin(b, fixture)
        samples = []
        for n in range(args.submits):
            cid, line = submit_line(fixture, n)
            r, ms = b.call(line)
            if r.get("type") != "capture_received" or r["payload"].get("durable") is not True:
                add(FAIL, f"capture {cid}: capture_received durable:true", json.dumps(r)[:200])
                break
            samples.append(ms)
        if len(samples) == args.submits:
            out["capture_latency_ms"] = {"n": len(samples), "min": round(min(samples), 3), "median": round(statistics.median(samples), 3),
                                         "max": round(max(samples), 3), "samples": [round(s, 3) for s in samples]}
            add(PASS, f"capture latency: {args.submits} x capture_submit -> capture_received (durable, fsync in the store)",
                "min %.3f / median %.3f / max %.3f ms (first %.3f ms)" % (min(samples), statistics.median(samples), max(samples), samples[0]))
        r, _ = b.call(env_line("convert", "capture_convert", {"capture_id": "rev5-capture-0"}))
        ok = r.get("type") == "error" and r["payload"].get("code") == "provider_disabled"
        add(PASS if ok else FAIL, "provider gap: capture_convert without --enable-grok -> provider_disabled", json.dumps(r)[:200])
        out["provider"] = r
    finally:
        b.close()

    # ---- Phase B: disconnect / retry
    store_b = args.store + "-disconnect"
    fresh(store_b)
    b = Bridge(args.bridge, store_b)
    open_and_pin(b, fixture)
    received = []
    error_text = None
    interrupted = None
    for n in range(args.submits):
        cid, line = submit_line(fixture, n)
        if n == args.kill_after:
            b.kill()  # SIGKILL mid-loop, before this submit's reply
            interrupted = cid
            try:
                b.call(line)
            except (BrokenPipeError, RuntimeError, ValueError, OSError) as exc:
                error_text = f"{type(exc).__name__}: {exc}"
            break
        r, _ = b.call(line)
        if r.get("type") == "capture_received":
            received.append(cid)
    add(PASS if error_text else FAIL, f"disconnect: SIGKILL of flashtex-bridge after {len(received)} receipts surfaces a client error on the next submit",
        error_text or "no error observed")
    out["disconnect_error"] = error_text
    # restart on the same store
    b2 = Bridge(args.bridge, store_b)
    try:
        open_and_pin(b2, fixture)
        durable = 0
        for cid in received:
            r, _ = b2.call(env_line(f"status-{cid}", "capture_status", {"capture_id": cid}))
            if r.get("type") == "capture_status":
                durable += 1
        add(PASS if durable == len(received) else FAIL, "retry: receipts survive the bridge crash (capture_status after restart on the same store)",
            f"{durable}/{len(received)} durable")
        # resubmit the interrupted id and the remaining ones
        resubmitted = 0
        for n in range(args.kill_after, args.submits):
            cid, line = submit_line(fixture, n)
            r, _ = b2.call(line)
            if r.get("type") == "capture_received" and r["payload"].get("durable") is True:
                resubmitted += 1
        add(PASS if resubmitted == args.submits - args.kill_after else FAIL,
            f"retry: resubmit of the interrupted id ({interrupted}) and the rest after relaunch -> capture_received",
            f"{resubmitted}/{args.submits - args.kill_after} received")
        r, _ = b2.call(env_line("dup", "capture_submit", json.loads(submit_line(fixture, 0)[1])["payload"]))
        add(PASS if r.get("type") == "capture_received" else FAIL, "retry: duplicate of a pre-crash id after restart returns the same durable record", json.dumps(r)[:160])
    finally:
        b2.close()
    files = sorted(f for f in os.listdir(store_b) if f.endswith(".json"))
    add(INFO, "journal files after restart", f"{len(files)}: {files[:12]}")

    counts = {s: sum(1 for r in rows if r[0] == s) for s in (PASS, FAIL, INFO)}
    print(f"\nsummary: {counts[PASS]} PASS, {counts[FAIL]} FAIL, {counts[INFO]} INFO")
    out["rows"], out["counts"] = rows, counts
    if args.json:
        json.dump(out, open(args.json, "w"), indent=1)
    return 1 if counts[FAIL] else 0


if __name__ == "__main__":
    sys.exit(main())
