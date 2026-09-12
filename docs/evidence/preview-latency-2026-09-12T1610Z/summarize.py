#!/usr/bin/env python3
"""Markdown tables from run.sh raw dirs: bench percentiles per cell and the
per-stage p50 attribution (tools/typing-bench/timeline.py) side by side for
every build of the same route/seed/interval. Usage: summarize.py <raw dir>... """
import glob, json, os, statistics as st, sys
sys.path.insert(0, os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "..", "..", "tools", "typing-bench"))
import timeline

STAGES = ["key->send", "send->v1", "v1->recv", "recv->val", "validate", "preraster", "deliver", "pub->paint", "key->paint"]

def p50(rows, key):
    v = [r[key] for r in rows if key in r]
    return st.median(v) if v else None

def fmt(x, d=0):
    return "—" if x is None else (f"{x:.{d}f}")

cells = {}
for raw in sys.argv[1:]:
    for path in sorted(glob.glob(os.path.join(raw, "*.json"))):
        d = json.load(open(path))
        name = os.path.basename(path)[:-5]
        log = path[:-5] + ".log"
        rows = timeline.parse(log) if os.path.exists(log) else []
        phases = timeline.helper_phases(log) if os.path.exists(log) else {}
        d["_rows"] = rows; d["_phases"] = phases; d["_raw"] = os.path.basename(raw)
        cells[(d["_raw"], d.get("build", "?"), d["route"], d["seed"], int(d["interval_ms"]))] = d

print("| raw | build | route | seed | interval | keystrokes | painted | coalesced | no-redraw paints | p50 | p95 | p99 | max | load before→after |")
print("|---|---|---|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---|")
for key in sorted(cells):
    d = cells[key]; k = d["keystroke_to_paint_ms"]
    print("| %s | %s | %s | %s | %d ms | %d | %d | %d | %d | %s | %s | %s | %s | %.1f→%.1f%s |" % (
        d["_raw"], d["build"], d["route"], d["seed"], d["interval_ms"], d["keystrokes"], d["painted"], d["coalesced"], d["paints_without_redraw"],
        fmt(k.get("p50_ms")), fmt(k.get("p95_ms")), fmt(k.get("p99_ms")), fmt(k.get("max_ms")), d["load_avg_before"], d["load_avg_after"],
        " (load-affected)" if d.get("load_affected") else ""))

print()
print("Per-stage p50 (ms) over painted v2 revisions (timeline.py):")
print()
print("| raw | build | route | seed | interval | painted revs | pages (reused) | " + " | ".join(STAGES) + " | helper parse / serialize |")
print("|---|---|---|---|---:|---:|---|" + "---:|" * len(STAGES) + "---|")
for key in sorted(cells):
    d = cells[key]; rows = d["_rows"]
    if not rows:
        print("| %s | %s | %s | %s | %d ms | 0 | — | " % (d["_raw"], d["build"], d["route"], d["seed"], d["interval_ms"]) + " | ".join("—" for _ in STAGES) + " | — |")
        continue
    pages = p50(rows, "pages")
    reused = [r["reused"] for r in rows if "reused" in r]
    helper = ""
    dt = d["_phases"].get("display_transport", []); oo = d["_phases"].get("optional_output", [])
    if dt:
        helper = "%s / %s" % (fmt(st.median([o["profile"]["parse_ms"] for o in dt if "profile" in o and "parse_ms" in o["profile"]]), 1),
                              fmt(st.median([o["serialization_ms"] for o in oo if "serialization_ms" in o]), 1) if oo else "—")
    print("| %s | %s | %s | %s | %d ms | %d | %s%s | " % (d["_raw"], d["build"], d["route"], d["seed"], d["interval_ms"], len(rows), fmt(pages),
          (" (%s)" % fmt(st.median(reused))) if reused else "") + " | ".join(fmt(p50(rows, s), 1) for s in STAGES) + " | %s |" % (helper or "—"))
