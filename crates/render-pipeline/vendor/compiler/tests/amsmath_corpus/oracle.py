#!/usr/bin/env python3
"""FT-061 amsmath oracle corpus: pdflatex word positions vs flashtex-render.

Oracle tooling only; pdflatex never runs in the product path.

    # regenerate the pinned reference positions (needs MacTeX; oracle only)
    python3 oracle.py refs --texbin /Library/TeX/texbin
    # measure a flashtex-render build against the pinned references
    python3 oracle.py check --render path/to/flashtex-render --fonts apps/mac/Fonts

Words are formed identically on both sides from glyph origins (a gap wider
than 0.16 em or a new baseline starts a word; PDF text objects are ignored),
using tools/visual-oracle/pdftext.py for the reference PDF and the
rendering-v2 display list's glyph_run items for the candidate. Words are
aligned on normalised text (tools/visual-oracle/rank.py:align_words).
A fixture passes when both sides have one page, every word aligns, and
every aligned word's origin is within 0.5 bp in x and y.
"""
import argparse, json, os, subprocess, sys, tempfile

HERE = os.path.dirname(os.path.abspath(__file__))
REPO = os.path.abspath(os.path.join(HERE, "..", "..", "..", ".."))
sys.path.insert(0, os.path.join(REPO, "tools", "visual-oracle"))
sys.path.insert(0, os.path.join(REPO, "tools", "real-world-corpus"))
import pdftext  # noqa: E402
import rank  # noqa: E402

FIXTURES = os.path.join(HERE, "fixtures")
REFS = os.path.join(HERE, "refs")
TOL = 0.5
Q = float(2 ** 20)


def regroup(glyphs):
    glyphs = sorted((dict(g, bt=0) for g in glyphs), key=lambda g: (round(g["y_top"], 1), g["x"]))
    return [{"text": w["text"], "x": round(w["x"], 4), "y_top": round(w["y_top"], 4)}
            for w in pdftext.words_from_glyphs(glyphs)]


def ref_pages(pdf):
    doc = pdftext.PdfDocument.load(pdf)
    return [regroup(pdftext.page_glyphs(doc, page)[0]) for page in doc.pages()]


def cand_pages(v2path):
    pl = json.load(open(v2path, encoding="utf-8"))["payload"]
    pages = []
    for page in pl["pages"]:
        glyphs = []
        for item in page.get("items", []):
            if item.get("kind") != "glyph_run":
                continue
            text = item.get("text") or ""
            clusters = item.get("clusters") or []
            for gi, g in enumerate(item.get("glyphs") or []):
                ci = g.get("cluster", gi)
                ct = text[clusters[ci]["text_start_byte"]:clusters[ci]["text_end_byte"]] if ci < len(clusters) else "?"
                glyphs.append({"text": ct, "x": g["origin_x"] / Q, "y_top": g["baseline_y"] / Q,
                               "advance": g["advance_x"] / Q, "size": item.get("font_size", 0) / Q, "font": ""})
        pages.append(regroup(glyphs))
    return pages


def fixtures(only):
    names = sorted(f[:-4] for f in os.listdir(FIXTURES) if f.endswith(".tex"))
    return [n for n in names if not only or any(o in n for o in only)]


def cmd_refs(args):
    os.makedirs(REFS, exist_ok=True)
    pdflatex = os.path.join(args.texbin, "pdflatex")
    version = subprocess.run([pdflatex, "--version"], capture_output=True, text=True).stdout.splitlines()[0]
    env = dict(os.environ, SOURCE_DATE_EPOCH="0", FORCE_SOURCE_DATE="1")
    with tempfile.TemporaryDirectory() as work:
        for name in fixtures(args.only):
            src = open(os.path.join(FIXTURES, name + ".tex"), encoding="utf-8").read()
            with open(os.path.join(work, name + ".tex"), "w", encoding="utf-8") as f:
                f.write(src)
            for _ in range(2):
                subprocess.run([pdflatex, "-interaction=batchmode", name + ".tex"], cwd=work, env=env,
                               stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
            pages = ref_pages(os.path.join(work, name + ".pdf"))
            with open(os.path.join(REFS, name + ".json"), "w", encoding="utf-8") as f:
                json.dump({"fixture": name + ".tex", "reference_engine": version,
                           "invocation": "pdflatex -interaction=batchmode, two passes, SOURCE_DATE_EPOCH=0 FORCE_SOURCE_DATE=1",
                           "unit": "bp, y from page top", "pages": pages}, f, indent=1)
                f.write("\n")
            print(f"{name}: {len(pages)} page(s), {sum(len(p) for p in pages)} words")


def cmd_check(args):
    passed, rows = 0, []
    with tempfile.TemporaryDirectory() as work:
        for name in fixtures(args.only):
            ref = json.load(open(os.path.join(REFS, name + ".json"), encoding="utf-8"))["pages"]
            text = open(os.path.join(FIXTURES, name + ".tex"), encoding="utf-8").read()
            req = {"protocol_version": 1, "id": name, "type": "compile",
                   "payload": {"project_id": "amsmath-corpus", "revision": 1, "entry_path": "main.tex",
                               "documents": [{"path": "main.tex", "text": text}]}}
            v2 = os.path.join(work, name + ".v2.json")
            env = dict(os.environ, FLASHTEX_FONT_DIRS=args.fonts, FLASHTEX_TFM_DIRS=args.fonts)
            p = subprocess.run([args.render, "--v2", v2], input=(json.dumps(req) + "\n").encode(), env=env,
                               capture_output=True, timeout=120)
            diags = []
            for line in p.stdout.decode("utf-8", "replace").splitlines():
                try:
                    m = json.loads(line)
                except json.JSONDecodeError:
                    continue
                if m.get("type") == "compile_result":
                    diags = [d for d in m["payload"].get("diagnostics", [])
                             if d.get("severity") == "error" or d.get("code") == "math_limitation"]
            cand = cand_pages(v2) if os.path.isfile(v2) else []
            n = ok = unaligned = 0
            worst = 0.0
            for rw, cw in zip(ref, cand):
                idx, ur, uc = rank.align_words(rw, cw)
                unaligned += ur + uc
                for i, j in idx:
                    d = max(abs(cw[j]["x"] - rw[i]["x"]), abs(cw[j]["y_top"] - rw[i]["y_top"]))
                    worst, n, ok = max(worst, d), n + 1, ok + (d <= TOL)
            good = len(ref) == len(cand) == 1 and n > 0 and ok == n and unaligned == 0
            passed += good
            row = {"fixture": name, "pass": good, "pages": [len(ref), len(cand)], "aligned": n, "within_tol": ok,
                   "unaligned": unaligned, "worst_bp": round(worst, 3),
                   "diagnostics": [(d.get("code"), (d.get("message") or "")[:120]) for d in diags]}
            rows.append(row)
            print(f"{'PASS' if good else 'FAIL'} {name:26} words {n:3} ok {ok:3} unaligned {unaligned:3} worst {worst:8.3f}")
    print(f"TOTAL {passed}/{len(rows)} within {TOL} bp")
    if args.json:
        with open(args.json, "w", encoding="utf-8") as f:
            json.dump({"passed": passed, "total": len(rows), "tolerance_bp": TOL, "fixtures": rows}, f, indent=1)
            f.write("\n")
    return 0 if passed == len(rows) else 1


def main():
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    sub = ap.add_subparsers(dest="cmd", required=True)
    r = sub.add_parser("refs")
    r.add_argument("--texbin", default="/Library/TeX/texbin")
    r.add_argument("only", nargs="*")
    c = sub.add_parser("check")
    c.add_argument("--render", required=True)
    c.add_argument("--fonts", default=os.path.join(REPO, "apps", "mac", "Fonts"))
    c.add_argument("--json")
    c.add_argument("only", nargs="*")
    args = ap.parse_args()
    return cmd_refs(args) if args.cmd == "refs" else cmd_check(args)


if __name__ == "__main__":
    sys.exit(main() or 0)
