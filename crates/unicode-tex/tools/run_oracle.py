#!/usr/bin/env python3
"""Compile fixtures/tex/*.tex with xelatex and lualatex (oracle only) and
record word positions (pdftotext -bbox-layout) plus the probes each document
writes to its log (fontdimens, LuaTeX \\Umath parameters).

Output: fixtures/expected/<id>.<engine>.json. Usage:
    python3 tools/run_oracle.py [--build DIR] [--only id,id] [--engines xelatex,lualatex]
Requires /Library/TeX/texbin (MacTeX / TeX Live 2026) and poppler's pdftotext.
Never run by cargo test.
"""
import argparse
import html
import json
import os
import re
import subprocess
import sys
import tempfile

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.dirname(HERE)
TEXBIN = "/Library/TeX/texbin"

WORD_RE = re.compile(r'<word xMin="([\d.]+)" yMin="([\d.]+)" xMax="([\d.]+)" yMax="([\d.]+)">(.*?)</word>')
BP_TO_PT = 72.27 / 72.0


def dim(s):
    return float(s[:-2]) if s.endswith("pt") else float(s)


def parse_log(log):
    out = {}
    # TeX wraps log lines at 79 columns; join continuation lines of our probes.
    text = log.replace("\n", "")
    m = re.search(r"KCFONT (.*?)KC", text)
    if m:
        out["font"] = m.group(1).strip()
    m = re.search(r"KCDIMS size=([\d.]+pt) space=([\d.]+pt) stretch=([\d.]+pt) shrink=([\d.]+pt) extra=([\d.]+pt) xheight=([\d.]+pt)", text)
    if m:
        keys = ["size", "space", "stretch", "shrink", "extra", "xheight"]
        out["fontdimens_pt"] = {k: dim(v) for k, v in zip(keys, m.groups())}
    m = re.search(r"KCUMATH (.*?)(?:\(|\)|$|\[)", text)
    if m:
        vals = {}
        for kv in re.finditer(r"(\w+)=(-?[\d.]+)pt", m.group(1)):
            vals[kv.group(1)] = float(kv.group(2))
        out["umath_pt"] = vals
    return out


def run(engine, texfile, build):
    base = os.path.splitext(os.path.basename(texfile))[0]
    jobname = "%s.%s" % (base, engine)
    cmd = [os.path.join(TEXBIN, engine), "-interaction=nonstopmode", "-halt-on-error",
           "-jobname=" + jobname, "-output-directory=" + build, texfile]
    p = subprocess.run(cmd, capture_output=True, text=True, errors="replace", timeout=600)
    pdf = os.path.join(build, jobname + ".pdf")
    logf = os.path.join(build, jobname + ".log")
    log = open(logf, encoding="utf-8", errors="replace").read() if os.path.exists(logf) else ""
    result = {"engine": engine, "ok": p.returncode == 0 and os.path.exists(pdf)}
    if not result["ok"]:
        errs = [l for l in log.splitlines() if l.startswith("!")]
        result["errors"] = errs[:5]
        return result
    result.update(parse_log(log))
    xhtml = os.path.join(build, jobname + ".bbox.html")
    subprocess.run(["pdftotext", "-bbox-layout", pdf, xhtml], check=True)
    lines = []
    data = open(xhtml, encoding="utf-8").read()
    for block in re.findall(r"<line[^>]*>(.*?)</line>", data, re.S):
        words = []
        for m in WORD_RE.finditer(block):
            x0, y0, x1, y1, t = m.groups()
            # pdftotext reports PDF points (bp); the oracle JSON keeps TeX pt.
            words.append({"t": html.unescape(t), "x0": round(float(x0) * BP_TO_PT, 4),
                          "x1": round(float(x1) * BP_TO_PT, 4), "y0": round(float(y0) * BP_TO_PT, 4)})
        if words:
            lines.append(words)
    lines.sort(key=lambda ws: ws[0]["y0"])
    result["lines"] = lines
    return result


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--build", default=None)
    ap.add_argument("--only", default="")
    ap.add_argument("--engines", default="xelatex,lualatex")
    a = ap.parse_args()
    build = a.build or tempfile.mkdtemp(prefix="kc104-oracle-")
    os.makedirs(build, exist_ok=True)
    specs = json.load(open(os.path.join(ROOT, "fixtures", "specs.json"), encoding="utf-8"))
    only = set(filter(None, a.only.split(",")))
    versions = {}
    for engine in a.engines.split(","):
        v = subprocess.run([os.path.join(TEXBIN, engine), "--version"], capture_output=True, text=True).stdout
        versions[engine] = v.splitlines()[0]
    for fx in specs["fixtures"]:
        if only and fx["id"] not in only:
            continue
        tex = os.path.join(ROOT, "fixtures", "tex", fx["id"] + ".tex")
        for engine in a.engines.split(","):
            r = run(engine, tex, build)
            r["engine_version"] = versions[engine]
            r["pdftotext"] = "poppler pdftotext -bbox-layout, x in TeX pt (bp*72.27/72)"
            out = os.path.join(ROOT, "fixtures", "expected", "%s.%s.json" % (fx["id"], engine))
            with open(out, "w", encoding="utf-8") as f:
                json.dump(r, f, ensure_ascii=False, indent=1)
                f.write("\n")
            status = "ok" if r["ok"] else "FAIL %s" % r.get("errors")
            print("%-28s %-8s %s" % (fx["id"], engine, status), flush=True)


if __name__ == "__main__":
    sys.exit(main())
