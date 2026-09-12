#!/usr/bin/env python3
"""Preview/PDF box parity for the math visual corpus (FT-020 rev 3 acceptance
"preview/PDF consume identical boxes").

For every corpus case rendered by tools/run_visual.sh (work dir
<run>/flashtex/<case>/math/{compile_result.json,flashtex.pdf}) this script:
  1. reads the boxes the PDF writer (crates/pdf, pinned in run_visual.sh)
     actually wrote: every text-object origin (`x y Td` before `Tj`, PDF
     bottom-left origin, converted to top-left) and every `x y w h re f`
     rectangle in page 1's content stream;
  2. runs tools/coretext_boxes.swift on the same compile_result: the preview
     draw (CoreText per item at x_pt / baseline_y_pt; rule rectangles) into a
     CoreGraphics PDF, recording the box of each draw call;
  3. matches the three inventories (compile_result, PDF writer, CoreText) item
     by item, in order, and reports the largest |delta| of glyph origins and
     rule rectangles. Gate: every delta < --tolerance (default 0.05 pt); the
     writer rounds to 3 decimals so 0.0005 pt is its own floor.
Fonts are compared separately and reported (Latin Modern present or Times
substitution on each side); glyph *shapes* are outside this check — box
parity means the same origins, sizes and rule rectangles, not pixels.

Usage: python3 tools/box_parity.py --run <work dir> [--report OUT.md]
           [--tolerance 0.05] [--swift-bin PATH]
Exit 0 = pass, 1 = a delta exceeds the tolerance or an inventory differs,
2 = usage/tooling error.
"""
import argparse
import json
import os
import re
import subprocess
import sys
import tempfile
import zlib

HERE = os.path.dirname(os.path.abspath(__file__))
CRATE = os.path.dirname(HERE)


def page_streams(pdf_bytes):
    for m in re.finditer(rb"<<([^>]*?)>>\s*stream\r?\n(.*?)\r?\nendstream", pdf_bytes, re.S):
        hdr, body = m.group(1), m.group(2)
        if b"FlateDecode" in hdr:
            try:
                body = zlib.decompress(body)
            except zlib.error:
                continue
        if b"BT" in body or b" re" in body:
            yield body


def pdf_boxes(pdf_path, page_h):
    """Text origins and rule rectangles from the writer's content stream."""
    data = open(pdf_path, "rb").read()
    stream = next(page_streams(data), b"")
    text, rules = [], []
    pos = None
    font = None
    for tok in re.finditer(rb"([-\d.]+)\s+([-\d.]+)\s+Td|/(\w+)\s+([-\d.]+)\s+Tf|\(((?:\\.|[^\\)])*)\)\s*Tj|<([0-9A-Fa-f]+)>\s*Tj|([-\d.]+)\s+([-\d.]+)\s+([-\d.]+)\s+([-\d.]+)\s+re\s+f", stream):
        if tok.group(1) is not None:
            pos = (float(tok.group(1)), float(tok.group(2)))
        elif tok.group(3) is not None:
            font = (tok.group(3).decode(), float(tok.group(4)))
        elif tok.group(5) is not None or tok.group(6) is not None:
            text.append({"x_pt": pos[0], "baseline_y_pt": page_h - pos[1], "font": font[0], "size": font[1]})
        else:
            x, y, w, h = (float(tok.group(i)) for i in range(7, 11))
            rules.append({"x_pt": x, "y_pt": page_h - (y + h), "width_pt": w, "height_pt": h})
    return text, rules


def pdf_font_names(pdf_path):
    data = open(pdf_path, "rb").read()
    names = {}
    for m in re.finditer(rb"/(F\d+)\s+(\d+)\s+0\s+R", data):
        obj = re.search(rb"\n" + m.group(2) + rb" 0 obj\s*(.*?)endobj", data, re.S)
        if obj:
            b = re.search(rb"/BaseFont\s*/(?:[A-Z]{6}\+)?([\w-]+)", obj.group(1))
            if b:
                names[m.group(1).decode()] = b.group(1).decode()
    return names


def compare(case, res_items, pdf_text, pdf_rules, ct_items, tol):
    exp_text = [i for i in res_items if i["kind"] == "text"]
    exp_rules = [i for i in res_items if i["kind"] == "rule"]
    ct_text = [i for i in ct_items if i["kind"] == "text"]
    ct_rules = [i for i in ct_items if i["kind"] == "rule"]
    problems = []
    if not (len(exp_text) == len(pdf_text) == len(ct_text)):
        problems.append(f"text inventory differs: compile_result {len(exp_text)}, pdf {len(pdf_text)}, coretext {len(ct_text)}")
    if not (len(exp_rules) == len(pdf_rules) == len(ct_rules)):
        problems.append(f"rule inventory differs: compile_result {len(exp_rules)}, pdf {len(pdf_rules)}, coretext {len(ct_rules)}")
    worst_pdf = worst_ct = 0.0
    rows = []
    for e, p, c in zip(exp_text, pdf_text, ct_text):
        dp = max(abs(p["x_pt"] - e["x_pt"]), abs(p["baseline_y_pt"] - e["baseline_y_pt"]), abs(p["size"] - e["font_size_pt"]))
        dc = max(abs(c["x_pt"] - e["x_pt"]), abs(c["baseline_y_pt"] - e["baseline_y_pt"]), abs(c["font_size_pt"] - e["font_size_pt"]))
        worst_pdf, worst_ct = max(worst_pdf, dp), max(worst_ct, dc)
        rows.append((e["text"], e["x_pt"], e["baseline_y_pt"], dp, dc, p["font"], c["font"]))
    rule_rows = []
    for e, p, c in zip(exp_rules, pdf_rules, ct_rules):
        dp = max(abs(p[k] - e[k]) for k in ("x_pt", "y_pt", "width_pt", "height_pt"))
        dc = max(abs(c[k] - e[k]) for k in ("x_pt", "y_pt", "width_pt", "height_pt"))
        worst_pdf, worst_ct = max(worst_pdf, dp), max(worst_ct, dc)
        rule_rows.append((e["x_pt"], e["y_pt"], e["width_pt"], e["height_pt"], dp, dc))
    if worst_pdf >= tol:
        problems.append(f"pdf writer max |Δ| {worst_pdf:.4f} >= {tol}")
    if worst_ct >= tol:
        problems.append(f"coretext max |Δ| {worst_ct:.4f} >= {tol}")
    return {"case": case, "text_items": len(exp_text), "rule_items": len(exp_rules),
            "max_delta_pdf_pt": round(worst_pdf, 5), "max_delta_coretext_pt": round(worst_ct, 5),
            "pass": not problems, "problems": problems}, rows, rule_rows


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--run", required=True, help="tools/run_visual.sh work dir (contains flashtex/<case>/math)")
    ap.add_argument("--report")
    ap.add_argument("--json")
    ap.add_argument("--tolerance", type=float, default=0.05)
    ap.add_argument("--swift-bin", help="prebuilt coretext_boxes binary")
    a = ap.parse_args()
    root = os.path.join(a.run, "flashtex")
    if not os.path.isdir(root):
        print(f"no flashtex/ under {a.run}", file=sys.stderr)
        return 2
    tmp = tempfile.mkdtemp(prefix="box-parity-")
    swift_bin = a.swift_bin or os.path.join(tmp, "coretext_boxes")
    if not a.swift_bin:
        r = subprocess.run(["swiftc", "-O", "-o", swift_bin, os.path.join(HERE, "coretext_boxes.swift")], capture_output=True, text=True)
        if r.returncode != 0:
            print(r.stderr, file=sys.stderr)
            return 2
    results, lines, failed = [], [], []
    for case in sorted(os.listdir(root)):
        d = os.path.join(root, case, "math")
        cr, pdf = os.path.join(d, "compile_result.json"), os.path.join(d, "flashtex.pdf")
        if not (os.path.exists(cr) and os.path.exists(pdf)):
            continue
        res = json.load(open(cr))
        page = res["payload"]["pages"][0]
        page_h = page["height_pt"]
        pdf_text, pdf_rules = pdf_boxes(pdf, page_h)
        fonts = pdf_font_names(pdf)
        for t in pdf_text:
            t["font"] = fonts.get(t["font"], t["font"])
        ct_pdf = os.path.join(tmp, case + ".coretext.pdf")
        r = subprocess.run([swift_bin, cr, ct_pdf], capture_output=True, text=True)
        if r.returncode != 0:
            print(f"{case}: coretext_boxes failed: {r.stderr}", file=sys.stderr)
            return 2
        ct = json.loads(r.stdout)
        summary, rows, rule_rows = compare(case, page["items"], pdf_text, pdf_rules, ct["items"], a.tolerance)
        summary["pdf_stderr"] = open(os.path.join(d, "pdf.stderr"), errors="replace").read().strip() if os.path.exists(os.path.join(d, "pdf.stderr")) else ""
        results.append(summary)
        if not summary["pass"]:
            failed.append(case)
        lines.append(f"\n### {case}\n")
        lines.append(f"- text items {summary['text_items']}, rule items {summary['rule_items']}; max |Δ| vs compile_result: PDF writer {summary['max_delta_pdf_pt']:.5f} pt, CoreText {summary['max_delta_coretext_pt']:.5f} pt")
        lines.append(f"- verdict: {'PASS' if summary['pass'] else 'FAIL: ' + '; '.join(summary['problems'])}")
        if rows:
            lines.append("\n| glyph | x | baseline | Δ pdf | Δ coretext | pdf face | coretext face |")
            lines.append("|---|---|---|---|---|---|---|")
            for t, x, y, dp, dc, pf, cf in rows:
                lines.append(f"| {t} | {x:.5f} | {y:.5f} | {dp:.5f} | {dc:.5f} | {pf} | {cf} |")
        if rule_rows:
            lines.append("\n| rule x | y | w | h | Δ pdf | Δ coretext |")
            lines.append("|---|---|---|---|---|---|")
            for x, y, w, h, dp, dc in rule_rows:
                lines.append(f"| {x:.5f} | {y:.5f} | {w:.5f} | {h:.5f} | {dp:.5f} | {dc:.5f} |")
    worst_pdf = max((r["max_delta_pdf_pt"] for r in results), default=0.0)
    worst_ct = max((r["max_delta_coretext_pt"] for r in results), default=0.0)
    header = [
        "# Preview/PDF box parity report",
        "",
        f"Cases: {len(results)}; failed: {len(failed)} {failed if failed else ''}",
        f"Tolerance: {a.tolerance} pt. Largest |Δ| over the corpus: PDF writer {worst_pdf:.5f} pt, CoreText placement {worst_ct:.5f} pt.",
        "Both sides consume the same runtime-v1 compile_result (text items per glyph with font-hints-v1, rule items per rules-v1). "
        "The PDF column is what crates/pdf wrote (`Td` origins, `re` rectangles, 3-decimal rounding); the CoreText column is what "
        "tools/coretext_boxes.swift drew (CTLineDraw per item at x_pt/baseline_y_pt, fill per rule). Faces are listed because the "
        "two sides substitute independently when Latin Modern is absent; glyph shapes are not compared here.",
        f"Run: `{a.run}`",
    ]
    text = "\n".join(header + lines) + "\n"
    if a.report:
        open(a.report, "w").write(text)
    else:
        sys.stdout.write(text)
    if a.json:
        json.dump({"tolerance_pt": a.tolerance, "max_delta_pdf_pt": worst_pdf, "max_delta_coretext_pt": worst_ct,
                   "cases": results}, open(a.json, "w"), indent=1)
    return 1 if failed else 0


if __name__ == "__main__":
    sys.exit(main())
