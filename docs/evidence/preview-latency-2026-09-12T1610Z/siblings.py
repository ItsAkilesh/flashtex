#!/usr/bin/env python3
"""Produce display-list-v2 siblings for benchmarks: the p3 seed (demo body x2)
and the same seed with one character typed before \\end{document}; also the
27-page render fixture (protocol/fixtures/compile-request.json entry text).
Writes <out>/<name>.tex, <out>/<name>.v2.json (the sibling line, exact bytes) and
prints page counts + per-page byte ranges equality between p3 and p3-typed."""
import json, os, subprocess, sys
root, out, render, fonts = sys.argv[1:5]
os.makedirs(out, exist_ok=True)
demo = open(os.path.join(root, "apps/mac/Samples/demo.tex"), encoding="utf-8").read()
head, body = demo.split("\\begin{document}\n", 1)
body = body.rsplit("\\end{document}", 1)[0]

def compile_(text, name):
    req = {"protocol_version": 1, "type": "compile", "id": name, "payload": {
        "project_id": "bench", "revision": 1, "entry_path": "main.tex",
        "documents": [{"path": "main.tex", "text": text}],
        "layout_capabilities": ["rules-v1", "font-hints-v1", "display-list-v2"]}}
    env = dict(os.environ, FLASHTEX_LM_DIR=fonts, FLASHTEX_FONT_DIRS=fonts,
               FLASHTEX_TFM_DIRS=os.path.join(fonts, "texmf/fonts/tfm/public/lm"))
    p = subprocess.run([render], input=(json.dumps(req) + "\n").encode(), capture_output=True, env=env, timeout=180)
    v1 = v2 = None
    for raw in p.stdout.split(b"\n"):
        if not raw.strip(): continue
        obj = json.loads(raw)
        if obj.get("type") == "compile_result": v1 = obj
        if obj.get("type") == "display_list": v2 = raw
    pages = len(v1["payload"].get("pages", [])) if v1 else -1
    open(os.path.join(out, name + ".tex"), "w", encoding="utf-8").write(text)
    if v2: open(os.path.join(out, name + ".v2.json"), "wb").write(v2)
    print("%-10s %7d bytes tex -> %2d page(s), sibling %s" % (name, len(text.encode()), pages, len(v2) if v2 else "none"))
    return v2

p3 = head + "\\begin{document}\n" + body * 2 + "\\end{document}\n"
a = compile_(p3, "p3")
i = p3.rfind("\\end{document}")
b = compile_(p3[:i] + "x" + p3[i:], "p3-typed")
c = compile_(p3[:i] + "xy" + p3[i:], "p3-typed2")
pmax = head + "\\begin{document}\n" + body * 4 + "\\end{document}\n"
compile_(pmax, "pmax")
compile_(pmax[:pmax.rfind("\\end{document}")] + "x" + pmax[pmax.rfind("\\end{document}"):], "pmax-typed")
req = json.load(open(os.path.join(root, "protocol/fixtures/compile-request.json")))
text = next(d["text"] for d in req["payload"]["documents"] if d["path"] == req["payload"]["entry_path"])
compile_(text, "fixture")

def page_blobs(line):
    obj = json.loads(line)
    return [json.dumps(p, separators=(",", ":"), sort_keys=True) for p in obj["payload"]["pages"]]
if a and b:
    pa, pb = page_blobs(a), page_blobs(b)
    same = [i + 1 for i in range(min(len(pa), len(pb))) if pa[i] == pb[i]]
    print("p3 vs p3-typed: pages with identical canonical JSON:", same, "of", len(pa), "/", len(pb))
    print("page sizes p3:", [len(x) for x in pa])
