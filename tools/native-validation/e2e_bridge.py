#!/usr/bin/env python3
"""Bridge receipt path against the real flashtex-bridge binary (stdlib only).

Sequence over one process on stdin/stdout JSON Lines, with a private store dir:
  document_open -> document_opened
  destination_pin -> destination_pinned
  capture_submit (fixture image) -> capture_received durable:true
  capture_submit (same fixture again) -> same record (duplicate, not a second receipt)
  capture_convert (bridge started WITHOUT --enable-grok) -> error provider_disabled
  capture_reject -> capture_rejected; capture_status -> rejected:true
Optionally `--also-fixture label=path` submits other fixture files in fresh
stores and records whether the bridge accepts them (INFO/FAIL for fixture owners).

Usage: e2e_bridge.py --bridge <bin> --fixture <capture-submission.json> --store <dir> [--json out]
"""
import argparse
import json
import os
import shutil
import subprocess
import sys

PASS, FAIL, INFO, FINDING = "PASS", "FAIL", "INFO", "FINDING"
rows = []


def add(s, n, d=""):
    rows.append((s, n, d))
    print(f"[{s}] {n}" + (f" -- {d}" if d else ""))


def req(i, t, p):
    return json.dumps({"protocol_version": 1, "id": i, "type": t, "payload": p})


def run_bridge(binary, store, lines, timeout=30):
    shutil.rmtree(store, ignore_errors=True)
    os.makedirs(store)
    env = dict(os.environ)
    env.pop("XAI_API_KEY", None)  # never let a real key reach the provider path
    p = subprocess.run([binary, "--store", store], input=("\n".join(lines) + "\n").encode("utf-8"),
                       capture_output=True, timeout=timeout, env=env)
    replies = []
    for l in p.stdout.decode("utf-8", "replace").splitlines():
        try:
            replies.append(json.loads(l))
        except json.JSONDecodeError:
            replies.append({"unparseable": l[:200]})
    return p.returncode, replies, p.stderr.decode("utf-8", "replace")


def main():
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--bridge", required=True)
    ap.add_argument("--fixture", required=True, help="capture_submit envelope file (one JSON object)")
    ap.add_argument("--store", required=True, help="private journal dir (outside the repo); recreated")
    ap.add_argument("--also-fixture", action="append", default=[], help="label=path: extra fixtures to submit in fresh stores")
    ap.add_argument("--json")
    args = ap.parse_args()

    fixture = json.load(open(args.fixture, encoding="utf-8"))
    fx_line = json.dumps(fixture)
    cid = fixture["payload"]["capture_id"]
    dest = fixture["payload"]["destination_id"]
    rev = fixture["payload"]["base_revision"]
    text = "Hello FlashTeX.\n"
    lines = [
        req("open", "document_open", {"project_id": "p", "path": "main.tex", "revision": rev, "text": text}),
        req("pin", "destination_pin", {"destination_id": dest, "project_id": "p", "path": "main.tex",
                                       "revision": rev, "start_byte": 5, "end_byte": 5}),
        fx_line,
        fx_line,
        req("convert", "capture_convert", {"capture_id": cid}),
        req("reject", "capture_reject", {"capture_id": cid}),
        req("status", "capture_status", {"capture_id": cid}),
    ]
    rc, r, err = run_bridge(args.bridge, args.store, lines)
    add(PASS if rc == 0 else FAIL, "bridge: process exit", f"exit {rc}" + (f" stderr={err[:200]!r}" if err.strip() else ""))
    if len(r) != 7:
        add(FAIL, "bridge: one reply per request", f"expected 7, got {len(r)}: {r}")
        return finish(args)

    def expect(i, name, cond, detail):
        add(PASS if cond else FAIL, name, detail)

    expect(0, "bridge: document_open -> document_opened", r[0].get("type") == "document_opened", json.dumps(r[0])[:200])
    expect(1, "bridge: destination_pin -> destination_pinned", r[1].get("type") == "destination_pinned"
           and r[1]["payload"].get("valid") is not False, json.dumps(r[1])[:200])
    p2 = r[2].get("payload", {})
    expect(2, "bridge: capture_submit -> capture_received durable:true",
           r[2].get("type") == "capture_received" and p2.get("durable") is True and p2.get("capture_id") == cid,
           json.dumps(r[2])[:200])
    expect(3, "bridge: duplicate capture_submit -> same record (no second receipt id, same payload)",
           r[3].get("type") == "capture_received" and r[3].get("payload") == p2, json.dumps(r[3])[:200])
    p4 = r[4].get("payload", {})
    expect(4, "bridge: capture_convert without --enable-grok -> error provider_disabled",
           r[4].get("type") == "error" and p4.get("code") == "provider_disabled", json.dumps(r[4])[:200])
    expect(5, "bridge: capture_reject -> capture_rejected", r[5].get("type") == "capture_rejected", json.dumps(r[5])[:200])
    p6 = r[6].get("payload", {})
    expect(6, "bridge: capture_status -> rejected:true", r[6].get("type") == "capture_status" and p6.get("rejected") is True,
           json.dumps(r[6])[:200])
    files = sorted(os.listdir(args.store))
    add(PASS if any(cid in f for f in files) else FAIL, "bridge: durable journal record on disk", f"{args.store}: {files}")

    for spec in args.also_fixture:
        label, path = spec.split("=", 1)
        fx2 = json.load(open(path, encoding="utf-8"))
        d2, r2 = fx2["payload"]["destination_id"], fx2["payload"]["base_revision"]
        lines2 = [
            req("open", "document_open", {"project_id": "p", "path": "main.tex", "revision": r2, "text": text}),
            req("pin", "destination_pin", {"destination_id": d2, "project_id": "p", "path": "main.tex",
                                           "revision": r2, "start_byte": 5, "end_byte": 5}),
            json.dumps(fx2),
        ]
        rc2, rr, _ = run_bridge(args.bridge, args.store + "-" + label, lines2)
        rep = rr[2] if len(rr) > 2 else {}
        ok = rep.get("type") == "capture_received"
        add(PASS if ok else FINDING, f"fixture {label}: capture_submit accepted by the bridge (fixture owner: Commander)",
            json.dumps(rep)[:220])
    return finish(args)


def finish(args):
    counts = {s: sum(1 for r in rows if r[0] == s) for s in (PASS, FAIL, INFO, FINDING)}
    print(f"\nsummary: {counts[PASS]} PASS, {counts[FAIL]} FAIL, {counts[INFO]} INFO, {counts[FINDING]} FINDING")
    if args.json:
        json.dump({"rows": rows, "counts": counts}, open(args.json, "w"), indent=1)
    return 1 if counts[FAIL] else 0


if __name__ == "__main__":
    sys.exit(main())
