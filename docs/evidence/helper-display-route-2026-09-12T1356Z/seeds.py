#!/usr/bin/env python3
"""Seeds for run.sh: apps/mac/Samples/demo.tex body repeated until the producer
lays out >= 3 pages (p3) and >= 27 pages (p27). Page counts are PROBED through
the actual producer (one direct compile with display-list-v2 requested), so the
recorded counts and whether the producer declined the v2 sibling are facts of
this build, not assumptions. Usage: seeds.py <repo root> <work dir> <flashtex-render> <fonts dir>"""
import json, os, subprocess, sys

root, work, render, fonts = sys.argv[1:5]
demo = open(os.path.join(root, "apps/mac/Samples/demo.tex"), encoding="utf-8").read()
head, body = demo.split("\\begin{document}\n", 1)
body = body.rsplit("\\end{document}", 1)[0]


def probe(text):
    req = {"protocol_version": 1, "type": "compile", "id": "seed", "payload": {
        "project_id": "seed", "revision": 1, "entry_path": "main.tex",
        "documents": [{"path": "main.tex", "text": text}],
        "layout_capabilities": ["rules-v1", "font-hints-v1", "display-list-v2"]}}
    env = dict(os.environ, FLASHTEX_LM_DIR=fonts, FLASHTEX_FONT_DIRS=fonts)
    p = subprocess.run([render], input=json.dumps(req) + "\n", text=True, capture_output=True, env=env, timeout=120)
    lines = [json.loads(l) for l in p.stdout.splitlines() if l.strip()]
    v1 = next((l for l in lines if l.get("type") == "compile_result"), None)
    v2 = next((l for l in lines if l.get("type") == "display_list"), None)
    pages = len(v1["payload"].get("pages", [])) if v1 else -1
    declined = any(d.get("code") == "display_list_declined" for d in (v1 or {}).get("payload", {}).get("diagnostics", []))
    v2_bytes = len(json.dumps(v2, separators=(",", ":"))) if v2 else 0
    return pages, v2 is not None, declined, v2_bytes


def build(target_pages):
    n = 1
    while True:
        text = head + "\\begin{document}\n" + body * n + "\\end{document}\n"
        pages, has_v2, declined, v2_bytes = probe(text)
        if pages >= target_pages or n > 64:
            return text, n, pages, has_v2, declined, v2_bytes
        n += 1


for name, target in (("p3", 3), ("p27", 27)):
    text, n, pages, has_v2, declined, v2_bytes = build(target)
    open(os.path.join(work, name + ".tex"), "w", encoding="utf-8").write(text)
    print("seed %-4s body x%-2d %6d bytes -> %2d page(s); v2 sibling: %s%s" % (
        name, n, len(text.encode("utf-8")), pages,
        ("%d bytes" % v2_bytes) if has_v2 else "none", " (display_list_declined by the producer)" if declined else ""))
