#!/usr/bin/env python3
"""Drives flashtex-render over its stdin/stdout JSON Lines like the Mac app:
one compile request per keystroke (typed before \\end{document}), waits for
the compile_result and the display_list sibling, records wall time per
request (send -> last reply byte) and the reply sizes.
Usage: ipc_bench.py <binary> <file.tex> [steps] [--no-v2] [--md5 out]
"""
import hashlib, json, os, subprocess, sys, time

binary, path = sys.argv[1], sys.argv[2]
steps = int(sys.argv[3]) if len(sys.argv) > 3 and sys.argv[3].isdigit() else 40
v2 = "--no-v2" not in sys.argv
md5_out = None
if "--md5" in sys.argv:
    md5_out = sys.argv[sys.argv.index("--md5") + 1]
text = open(path, encoding="utf-8").read()
name = os.path.basename(path)
at = text.rfind("\\end{document}")
if at < 0:
    at = len(text)
typed = "abcde fghij "
env = dict(os.environ)
env.setdefault("FLASHTEX_FONT_DIRS", "/Users/jay3332/Projects/flashtex/.claude/worktrees/agent-a30b84d731d111cea/apps/mac/Fonts")
p = subprocess.Popen([binary], stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=subprocess.DEVNULL, env=env)
caps = ["rules-v1", "font-hints-v1"] + (["display-list-v2"] if v2 else [])
lat, sizes_v1, sizes_v2 = [], [], []
digest = hashlib.md5()
first = None
for step in range(steps + 1):
    body = text[:at] + "".join(typed[i % len(typed)] for i in range(step)) + text[at:]
    req = {"protocol_version": 1, "id": f"mac-{step}", "type": "compile",
           "payload": {"project_id": "demo", "revision": step + 1, "entry_path": name,
                       "documents": [{"path": name, "text": body}], "layout_capabilities": caps}}
    line = (json.dumps(req, separators=(",", ":")) + "\n").encode()
    t0 = time.perf_counter()
    p.stdin.write(line); p.stdin.flush()
    r1 = p.stdout.readline()
    t1 = time.perf_counter()
    r2 = b""
    if v2 and b'"display-list-v2"' in r1:
        r2 = p.stdout.readline()
    t2 = time.perf_counter()
    digest.update(r1); digest.update(r2)
    if step == 0:
        first = ((t2 - t0) * 1000, len(r1), len(r2))
        continue
    lat.append((t2 - t0) * 1000); sizes_v1.append(len(r1)); sizes_v2.append(len(r2))
p.stdin.close(); p.wait()
lat.sort()
def pct(q): return lat[min(len(lat) - 1, int(round(q * (len(lat) - 1))))]
print(f"{name:12s} v2={v2!s:5s} steps={steps} first(cold) {first[0]:.1f} ms (v1 {first[1]} B, v2 {first[2]} B) | warm p50 {pct(0.5):.2f} p95 {pct(0.95):.2f} max {lat[-1]:.2f} ms | v1 {sum(sizes_v1)//len(sizes_v1)} B v2 {sum(sizes_v2)//len(sizes_v2)} B | md5 {digest.hexdigest()[:16]}")
if md5_out:
    open(md5_out, "a").write(f"{name} v2={v2} steps={steps} {digest.hexdigest()}\n")
