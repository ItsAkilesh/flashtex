#!/usr/bin/env python3
"""Writes summary.md from run.sh's raw cell summaries.
Usage: summarize.py <out.md> <raw dir> <repo root> <UTC> <helper> <render> <seeds.txt>"""
import glob, hashlib, json, os, subprocess, sys

out, raw, root, utc, helper, render, seeds_txt = sys.argv[1:8]


def sh(*a):
    try:
        return subprocess.check_output(a, text=True).strip()
    except Exception as e:  # noqa: BLE001
        return "unavailable (%s)" % e


def ms(v):
    return "—" if v is None else ("%.0f" % v if v >= 10 else "%.1f" % v)


def sha(path):
    return hashlib.sha256(open(path, "rb").read()).hexdigest()


runs = []
for f in sorted(glob.glob(os.path.join(raw, "*.json"))):
    d = json.load(open(f, encoding="utf-8"))
    d["_file"] = os.path.basename(f)
    runs.append(d)

lines = []
lines.append("# Helper display-candidate route — keystroke → v2 paint (%s)\n" % utc)
lines.append("Machine: %s, macOS %s. App: `%s` @ `%s`. Helper: `%s` (sha256 %s…). Producer: `%s` (sha256 %s…, built from origin/agent/mac-render-pipeline/unified f762f82a)." % (
    sh("sysctl", "-n", "machdep.cpu.brand_string"), sh("sw_vers", "-productVersion"),
    sh("git", "-C", root, "rev-parse", "--abbrev-ref", "HEAD"), sh("git", "-C", root, "rev-parse", "--short", "HEAD"),
    helper, sha(helper)[:16], render, sha(render)[:16]))
lines.append("")
lines.append("Paint point (from the app's summary): the v2 pane's bitmap blit of the validated candidate frame for the typed revision; the v1 control uses PreviewView's Canvas pass. Both are `%s`." % (runs[0]["paint_point"] if runs else "?"))
lines.append("")
lines.append("Seeds:\n")
lines.append("```\n" + open(seeds_txt, encoding="utf-8").read().strip() + "\n```\n")
lines.append("| cell | keystrokes | painted | coalesced | p50 ms | p95 ms | p99 ms | max ms | v1 previews | cand. admitted | painted | refused | invalid | dropped@paint | v2 declined | validation p50/p95 ms | load before→after |")
lines.append("|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---|---|")
for d in runs:
    k = d["keystroke_to_paint_ms"]; c = d.get("candidate_counters", {}); v = d.get("candidate_validation_ms")
    lines.append("| %s | %d | %d | %d | %s | %s | %s | %s | %d | %d | %d | %d | %d | %d | %d | %s | %.1f→%.1f (%s) |" % (
        d["_file"][:-5], d["keystrokes"], d["painted"], d["coalesced"], ms(k.get("p50_ms")), ms(k.get("p95_ms")), ms(k.get("p99_ms")), ms(k.get("max_ms")),
        c.get("v1_previews_applied", 0), c.get("candidates_admitted", 0), c.get("candidates_painted", 0), c.get("candidates_refused", 0),
        c.get("candidates_invalid", 0), c.get("candidates_dropped_at_paint", 0), c.get("declined_display_list_v2", 0),
        ("%.1f/%.1f" % (v["p50"], v["p95"])) if v else "—", d["load_avg_before"], d["load_avg_after"], d["load_note"]))
lines.append("")
lines.append("Reading: `helper-v2-*` cells paint ONLY when a candidate passed the native gate (session/request/generation/source-version correlation, off-main RenderingV2 + font + page validation, durable-text sha256/byte_length binding, paint-time recheck); a keystroke whose candidate was dropped by the helper's optional slot or refused natively is covered by the next painted revision (`coalesced`). `helper-v1-*` is the same helper/producer painting the v1 result. A p27 cell with `v2 declined` > 0 and 0 candidates is the producer's documented fallback (its v2 sibling exceeds the transport caps): those keystrokes cannot paint in the v2 pane and `painted` is expected to be 0 there — that is the fallback evidence, not a latency number.")
lines.append("")
lines.append("Not claimed: pixel parity with any PDF raster; an isolated-machine latency (see the load column); behaviour of producers other than the pinned build.")
open(out, "w", encoding="utf-8").write("\n".join(lines) + "\n")
print(open(out, encoding="utf-8").read())
