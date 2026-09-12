#!/usr/bin/env python3
"""Diff headline numbers between two run_all reports and two oracle reports.

run_all report: protocol counts line, swift test summary, latency line.
oracle report: headline-table rows (per sample/variant/compiler).
Prints one line per compared field with SAME / CHANGED and exits 0 always
(a change is flagged for a human, not judged here).

Usage: e2e_nonregression.py --prev-run a.md --new-run b.md --prev-oracle c.md --new-oracle d.md [--only fixture-hello] [--json out]
"""
import argparse
import json
import re
import sys


def run_all_fields(path):
    t = open(path, encoding="utf-8").read()
    f = {}
    m = re.search(r"Counts: (\d+) PASS, (\d+) FAIL, (\d+) INFO", t)
    if m:
        f["protocol PASS/FAIL/INFO"] = "/".join(m.groups())
    m = re.search(r"Executed (\d+) tests?, with (?:\d+ tests? skipped and )?(\d+) failures?", t)
    if m:
        f["swift tests executed/failures"] = "/".join(m.groups())
    m = re.search(r"min ([\d.]+) ms, median ([\d.]+) ms, max ([\d.]+) ms", t)
    if m:
        f["protocol latency min/median/max ms"] = "/".join(m.groups())
    for step in ("cargo build --release", "cargo test --release", "swift build", "swift test", "xcodebuild build", "check_protocol.py"):
        m = re.search(r"^\| (PASS|FAIL|SKIPPED) \| [^|]*" + re.escape(step), t, re.M)
        if m:
            f[f"step {step}"] = m.group(1)
    m = re.search(r"Overall: \*\*(\w+)\*\*", t)
    if m:
        f["overall"] = m.group(1)
    for name in ("FAIL", "PASS"):
        pass
    fails = re.findall(r"^\| FAIL \| ([^|]+) \|", t, re.M)
    f["protocol FAIL rows"] = "; ".join(x.strip() for x in fails) or "(none)"
    return f


def oracle_rows(path, only):
    t = open(path, encoding="utf-8").read()
    rows = {}
    in_table = False
    for line in t.splitlines():
        if line.startswith("## Headline table"):
            in_table = True
            continue
        if in_table and line.startswith("## "):
            break
        if in_table and line.startswith("| ") and not line.startswith("| Sample") and not line.startswith("|---"):
            cells = [c.strip() for c in line.strip("|").split("|")]
            if len(cells) < 13:
                continue
            sample, variant, comp = cells[0], cells[1], cells[2]
            if only and sample not in only:
                continue
            rows[f"{sample}/{variant}/{comp}"] = {
                "pages": cells[3], "mediabox": cells[4], "words": cells[5], "seq_equal": cells[6],
                "similarity": cells[7], "mean_dx": cells[8], "max_dx": cells[9], "mean_dy": cells[10],
                "max_dy": cells[11], "line_starts": cells[12]}
    return rows


def main():
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--prev-run")
    ap.add_argument("--new-run")
    ap.add_argument("--prev-oracle")
    ap.add_argument("--new-oracle")
    ap.add_argument("--only", action="append", default=[])
    ap.add_argument("--json")
    args = ap.parse_args()
    out = {"run_all": [], "oracle": []}
    changed = 0
    if args.prev_run and args.new_run:
        a, b = run_all_fields(args.prev_run), run_all_fields(args.new_run)
        for k in sorted(set(a) | set(b)):
            same = a.get(k) == b.get(k)
            # latency is expected to jitter; flag only if median moves by >2x
            if k.startswith("protocol latency") and a.get(k) and b.get(k):
                pm, nm = float(a[k].split("/")[1]), float(b[k].split("/")[1])
                same = nm <= 2 * pm and pm <= 2 * nm
                tag = "SAME(within 2x)" if same else "CHANGED"
            else:
                tag = "SAME" if same else "CHANGED"
            changed += 0 if same else 1
            out["run_all"].append((tag, k, a.get(k), b.get(k)))
            print(f"[{tag}] run_all {k}: prev={a.get(k)} new={b.get(k)}")
    if args.prev_oracle and args.new_oracle:
        a, b = oracle_rows(args.prev_oracle, args.only), oracle_rows(args.new_oracle, args.only)
        for k in sorted(set(a) | set(b)):
            ra, rb = a.get(k), b.get(k)
            if ra is None or rb is None:
                out["oracle"].append(("MISSING", k, ra, rb))
                print(f"[MISSING] oracle {k}: prev={'yes' if ra else 'no'} new={'yes' if rb else 'no'}")
                continue
            diff = {f: (ra[f], rb[f]) for f in ra if ra[f] != rb[f]}
            tag = "SAME" if not diff else "CHANGED"
            changed += 0 if not diff else 1
            out["oracle"].append((tag, k, ra, rb))
            print(f"[{tag}] oracle {k}" + (f": {diff}" if diff else ""))
    print(f"\nnon-regression: {changed} changed field(s)")
    if args.json:
        json.dump(out, open(args.json, "w"), indent=1)
    return 0


if __name__ == "__main__":
    sys.exit(main())
