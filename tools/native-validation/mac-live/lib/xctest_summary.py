#!/usr/bin/env python3
"""Summarises a `swift test` log as JSON: per test case passed / failed /
skipped with the skip reason, plus the totals line. Stdlib only.

Usage: xctest_summary.py --log <swift-test.log> --exit <code> --out <json>
"""
import argparse
import json
import re

CASE = re.compile(r"Test Case '-\[(\S+) (\S+)\]' (passed|failed|skipped)(?: \((\d+\.\d+) seconds\))?")
SKIP = re.compile(r"(\S+\.swift):(\d+): -\[(\S+) (\S+)\] : Test skipped(?: - (.*))?")
FAIL = re.compile(r"(\S+\.swift):(\d+): error: -\[(\S+) (\S+)\] : (.*)")
TOTAL = re.compile(r"Executed (\d+) tests?, with (\d+) tests? skipped and (\d+) failures? \((\d+) unexpected\) in ([\d.]+)")


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--log", required=True)
    ap.add_argument("--exit", type=int, required=True)
    ap.add_argument("--out", required=True)
    a = ap.parse_args()
    cases, skips, fails = {}, {}, {}
    total = None
    for line in open(a.log, encoding="utf-8", errors="replace"):
        m = CASE.search(line)
        if m:
            cases["%s/%s" % (m.group(1).split(".")[-1], m.group(2))] = {"result": m.group(3), "seconds": float(m.group(4)) if m.group(4) else None}
            continue
        m = SKIP.search(line)
        if m:
            skips["%s/%s" % (m.group(3).split(".")[-1], m.group(4))] = (m.group(5) or "").strip()
            continue
        m = FAIL.search(line)
        if m:
            fails.setdefault("%s/%s" % (m.group(3).split(".")[-1], m.group(4)), []).append("%s:%s: %s" % (m.group(1).split("/")[-1], m.group(2), m.group(5).strip()[:300]))
            continue
        m = TOTAL.search(line)
        if m:
            total = {"executed": int(m.group(1)), "skipped": int(m.group(2)), "failures": int(m.group(3)), "unexpected": int(m.group(4)), "seconds": float(m.group(5))}
    for k, v in cases.items():
        if k in skips:
            v["skip_reason"] = skips[k]
        if k in fails:
            v["failures"] = fails[k]
    out = {"exit_code": a.exit, "cases": cases, "totals": total,
           "passed": sum(1 for v in cases.values() if v["result"] == "passed"),
           "failed": sum(1 for v in cases.values() if v["result"] == "failed"),
           "skipped": sum(1 for v in cases.values() if v["result"] == "skipped")}
    json.dump(out, open(a.out, "w"), indent=1, sort_keys=True)
    print("xctest: %d passed, %d failed, %d skipped (exit %d)" % (out["passed"], out["failed"], out["skipped"], a.exit))
    for k, v in sorted(cases.items()):
        if v["result"] != "passed":
            print("    %s %s -- %s" % (v["result"].upper(), k, v.get("skip_reason") or "; ".join(v.get("failures", []))[:200]))


if __name__ == "__main__":
    main()
