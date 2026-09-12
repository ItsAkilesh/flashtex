"""Classify typing-bench paints as historical vs current from the app log.

Usage: analyze.py <bench dir> [<bench dir> ...]
Each bench dir holds typing-bench-<UTC>/compiler-<seed>-<ms>ms.json and
compiler-<seed>-<ms>ms.log (FLASHTEX_LOG). Prints one JSON summary per run
and a markdown table row.
"""
import glob
import json
import os
import re
import sys

APPLIED_H = re.compile(r"compile: applied historical revision (\d+) at (\d+)")
APPLIED_C = re.compile(r"compile: applied revision (\d+) at (\d+)")
PAINT = re.compile(r"paint: revision (\d+) at (\d+) \(covers (\d+) keystrokes")
HIST_PAINTED = re.compile(r"historical: painted (\S+) as revision (\d+)")
HIST_DROP = re.compile(r"historical: (refused|dropped|rejected)")


def pct(values, p):
    if not values:
        return None
    s = sorted(values)
    k = max(0, min(len(s) - 1, int(round(p / 100.0 * len(s) + 0.5)) - 1))
    return s[k]


def fmt(v):
    return "—" if v is None else ("%.0f" % v if v >= 10 else "%.1f" % v)


def analyze(json_path, log_path):
    d = json.load(open(json_path))
    hist, cur, paints = set(), set(), []
    hist_frames, hist_drops = 0, 0
    for line in open(log_path, encoding="utf-8", errors="replace"):
        m = APPLIED_H.search(line)
        if m:
            hist.add(int(m.group(1)))
            continue
        m = APPLIED_C.search(line)
        if m:
            cur.add(int(m.group(1)))
            continue
        m = PAINT.search(line)
        if m:
            paints.append((int(m.group(1)), int(m.group(2)), int(m.group(3))))
            continue
        if HIST_PAINTED.search(line):
            hist_frames += 1
        elif HIST_DROP.search(line):
            hist_drops += 1
    # first paint per revision (the bench's paint point), split by origin
    first_paint = {}
    for rev, ns, covers in paints:
        first_paint.setdefault(rev, ns)
    hist_lag, cur_lat, first_by_hist, first_by_cur = [], [], 0, 0
    to_current = []  # keystroke -> first CURRENT paint covering it
    cur_paint_list = sorted((first_paint[r], r) for r in cur if r in first_paint)
    for k in d["per_keystroke"]:
        rev, kns, pns, by = k["revision"], k["keystroke_ns"], k.get("paint_ns"), k.get("painted_by_revision")
        if pns is None:
            continue
        lat = (pns - kns) / 1e6
        if by in hist:
            first_by_hist += 1
            hist_lag.append(lat)
        else:
            first_by_cur += 1
            cur_lat.append(lat)
        for pns2, r in cur_paint_list:
            if r >= rev and pns2 > kns:
                to_current.append((pns2 - kns) / 1e6)
                break
    out = {
        "file": os.path.basename(json_path), "producer": d["producer"], "interval_ms": d["interval_ms"],
        "bytes": d["document_bytes_after"], "keystrokes": d["keystrokes"], "painted": d["painted"],
        "paints_total": d["paints"], "paints_historical": len(hist), "paints_current": len(cur),
        "historical_frames_painted_log": hist_frames, "historical_frames_refused_log": hist_drops,
        "keystrokes_first_shown_by_historical": first_by_hist, "keystrokes_first_shown_by_current": first_by_cur,
        "first_paint_all_ms": {"p50": pct([(k["paint_ns"] - k["keystroke_ns"]) / 1e6 for k in d["per_keystroke"] if k.get("paint_ns")], 50),
                                "p95": d["keystroke_to_paint_ms"].get("p95_ms"), "p99": d["keystroke_to_paint_ms"].get("p99_ms")},
        "historical_lag_ms": {"count": len(hist_lag), "p50": pct(hist_lag, 50), "p95": pct(hist_lag, 95), "p99": pct(hist_lag, 99), "max": max(hist_lag) if hist_lag else None},
        "first_shown_by_current_ms": {"count": len(cur_lat), "p50": pct(cur_lat, 50), "p95": pct(cur_lat, 95), "p99": pct(cur_lat, 99)},
        "keystroke_to_current_paint_ms": {"count": len(to_current), "p50": pct(to_current, 50), "p95": pct(to_current, 95), "p99": pct(to_current, 99), "max": max(to_current) if to_current else None},
        "compile_ms": d["compile_ms"], "coalesced": d["coalesced"], "unpainted": d["unpainted"],
    }
    return out


rows = []
for bench_dir in sys.argv[1:]:
    mode = os.path.basename(bench_dir.rstrip("/")).split("-", 1)[0]
    for jp in sorted(glob.glob(os.path.join(bench_dir, "typing-bench-*", "*.json"))):
        lp = os.path.join(bench_dir, os.path.basename(jp)[:-5] + ".log")
        if not os.path.exists(lp):
            print("no log for", jp, file=sys.stderr)
            continue
        o = analyze(jp, lp)
        o["mode"] = mode
        rows.append(o)
        print(json.dumps(o, indent=1))
print()
print("| mode | seed | interval | keys | paints (hist/cur) | keys first shown by hist/cur | first paint p50/p95/p99 | historical lag p50/p95/p99 | k→current paint p50/p95/p99/max | compile p50 | coalesced | unpainted |")
print("|---|---|---:|---:|---|---|---|---|---|---:|---:|---:|")
for o in rows:
    seed = o["file"][:-5].split("-")[1]
    a, h, c, t = o["first_paint_all_ms"], o["historical_lag_ms"], o["first_shown_by_current_ms"], o["keystroke_to_current_paint_ms"]
    print("| %s | %s | %dms | %d | %d (%d/%d) | %d/%d | %s/%s/%s | %s/%s/%s | %s/%s/%s/%s | %s | %d | %d |" % (
        o["mode"], seed, o["interval_ms"], o["keystrokes"], o["paints_total"], o["paints_historical"], o["paints_current"],
        o["keystrokes_first_shown_by_historical"], o["keystrokes_first_shown_by_current"],
        fmt(a["p50"]), fmt(a["p95"]), fmt(a["p99"]), fmt(h["p50"]), fmt(h["p95"]), fmt(h["p99"]),
        fmt(t["p50"]), fmt(t["p95"]), fmt(t["p99"]), fmt(t["max"]), fmt(o["compile_ms"].get("p50_ms")), o["coalesced"], o["unpainted"]))
