#!/usr/bin/env python3
"""Compiler round-trip latency over one document via the CLI path (stdlib).

Sends `compile` envelopes for the given document N times to one long-lived
flashtex-compiler process and measures send->complete-reply wall time.
This is the transport the Mac app uses (JSON Lines over stdin/stdout) but
without the app's 250 ms debounce/coalescing or UI work.

Usage: e2e_latency.py --compiler <bin> --document <file.tex> [--count 20] [--json out]
Prints: LATENCY ms: n=.. min=.. median=.. max=.. bytes=..
"""
import argparse
import json
import statistics
import subprocess
import sys
import time


def main():
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--compiler", required=True)
    ap.add_argument("--document", required=True)
    ap.add_argument("--count", type=int, default=20)
    ap.add_argument("--json")
    args = ap.parse_args()
    text = open(args.document, encoding="utf-8").read()
    proc = subprocess.Popen([args.compiler], stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=subprocess.PIPE)
    samples = []
    statuses = set()
    pages = None
    try:
        for n in range(args.count):
            env = {"protocol_version": 1, "id": f"lat-{n}", "type": "compile",
                   "payload": {"project_id": "e2e", "revision": n + 1, "entry_path": "main.tex",
                               "documents": [{"path": "main.tex", "text": text}]}}
            line = (json.dumps(env, ensure_ascii=False) + "\n").encode("utf-8")
            t0 = time.perf_counter()
            proc.stdin.write(line)
            proc.stdin.flush()
            raw = proc.stdout.readline()
            samples.append((time.perf_counter() - t0) * 1000.0)
            reply = json.loads(raw)
            if reply.get("type") != "compile_result":
                print(f"unexpected reply: {raw[:200]!r}")
                return 1
            statuses.add(reply["payload"]["status"])
            pages = len(reply["payload"]["pages"])
    finally:
        proc.stdin.close()
        proc.wait(timeout=10)
    stats = {"count": len(samples), "min_ms": round(min(samples), 3), "median_ms": round(statistics.median(samples), 3),
             "max_ms": round(max(samples), 3), "document_bytes": len(text.encode("utf-8")),
             "statuses": sorted(statuses), "pages": pages, "samples_ms": [round(s, 3) for s in samples]}
    print("LATENCY ms: n=%d min=%.3f median=%.3f max=%.3f bytes=%d pages=%s status=%s" % (
        stats["count"], stats["min_ms"], stats["median_ms"], stats["max_ms"], stats["document_bytes"], pages, ",".join(stats["statuses"])))
    if args.json:
        json.dump(stats, open(args.json, "w"), indent=1)
    return 0


if __name__ == "__main__":
    sys.exit(main())
