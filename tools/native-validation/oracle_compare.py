#!/usr/bin/env python3
"""Compare FlashTeX output against pdflatex used purely as a reference oracle.

pdflatex is never invoked by the product; this tool runs it only to produce a
reference PDF to measure against (GitHub issue #10). Python 3 stdlib only; word
boxes come from `oracle_extract` (PDFKit, built from oracle_extract.swift).

For every sample in --samples and every oracle variant:
  (a) pdflatex -interaction=batchmode in a temp dir            -> oracle PDF
  (b) FlashTeX compiler (preamble stripped) -> compile_result  -> flashtex-pdf --verify -> our PDF
  (c) page count / MediaBox equality, word-sequence equality after normalisation,
      per-word x/y deltas for aligned words, and line-start agreement.
Writes reports/oracle-<UTC>.md and reports/oracle-<UTC>.json.

Usage:
  oracle_compare.py --pdflatex /Library/TeX/texbin/pdflatex --extract <bin>
      --pdf-bin <flashtex-pdf> --compiler main=<bin> [--compiler de1020c=<bin>]
      --samples <dir> --reports-dir <dir> [--scratch <dir>] [--label key=value ...]
"""

import argparse
import difflib
import json
import os
import re
import statistics
import subprocess
import sys
import tempfile
import time
import unicodedata
from datetime import datetime, timezone

# Oracle variants. A is pdflatex's untouched default; B matches our compiler's
# font family, size and margins as far as LaTeX packages allow; C additionally
# removes justification and hyphenation, which our greedy breaker does not do.
VARIANTS = {
    "A-default": {
        "title": "default article (Computer Modern 10pt, LaTeX default margins)",
        "preamble": "\\documentclass{article}\n\\pagestyle{empty}\n",
        "apples": False,
    },
    "B-times12-1in": {
        "title": "article 12pt + times + geometry margin=1in + T1 fontenc + parindent 0pt + section numbering off (apples-to-apples for font/size/margins)",
        "preamble": ("\\documentclass[12pt]{article}\n\\usepackage[T1]{fontenc}\n\\usepackage{times}\n"
                     "\\usepackage[margin=1in]{geometry}\n\\setlength{\\parindent}{0pt}\n\\setcounter{secnumdepth}{0}\n\\pagestyle{empty}\n"),
        "apples": True,
    },
    "C-times12-1in-ragged": {
        "title": "variant B + \\raggedright + hyphenation off (closest to our greedy, unjustified line breaker)",
        "preamble": ("\\documentclass[12pt]{article}\n\\usepackage[T1]{fontenc}\n\\usepackage{times}\n"
                     "\\usepackage[margin=1in]{geometry}\n\\setlength{\\parindent}{0pt}\n\\setcounter{secnumdepth}{0}\n\\pagestyle{empty}\n"
                     "\\raggedright\n\\hyphenpenalty=10000\n\\exhyphenpenalty=10000\n"),
        "apples": True,
    },
}

BEGIN_DOC = "\\begin{document}"


def run(cmd, cwd=None, stdin_data=None, timeout=120):
    t0 = time.perf_counter()
    p = subprocess.run(cmd, cwd=cwd, input=stdin_data, capture_output=True, timeout=timeout)
    return p.returncode, p.stdout, p.stderr, time.perf_counter() - t0


def split_preamble(tex):
    """Return (stripped_preamble_text, body_from_begin_document)."""
    m = re.search(r"^[ \t]*" + re.escape(BEGIN_DOC), tex, re.M)  # not inside a comment line
    if not m:
        return "", tex
    return tex[:m.start()], tex[m.start():]


def strip_comment_lines(preamble):
    return "".join(l for l in preamble.splitlines(True) if not l.lstrip().startswith("%"))


def normalise_word(w):
    """Whitespace-split token -> comparable form.

    - NFKC folds the fi/ff/fl/ffi/ffl ligature code points pdflatex text
      extraction can yield (U+FB00..FB04) back to ASCII letters.
    - OT1-encoded accents come out of PDFKit as a spacing accent plus a base
      letter (`na¨ıve`, `caf´e`); NFD + dropping combining marks, spacing
      accents and mapping dotless i to i makes both sides compare as `naive`.
    """
    w = unicodedata.normalize("NFKC", w)
    w = unicodedata.normalize("NFD", w)
    out = []
    for ch in w:
        if unicodedata.combining(ch):
            continue
        if ch in "¨´`ˆ˜¸˚˝ˇ˘˙":
            continue
        if ch == "ı":
            ch = "i"
        out.append(ch)
    w = "".join(out)
    # pdflatex renders the em dash as one glyph; keep punctuation as-is otherwise.
    return w


def words_of(doc):
    """Flatten extractor JSON into [(page_no, text, x, bottom, right, top)]."""
    out = []
    for p in doc["pages"]:
        for w in p["words"]:
            out.append({"page": p["number"], "text": w["text"], "norm": normalise_word(w["text"]),
                        "x": w["x_pt"], "bottom": w["bottom_pt"], "right": w["right_pt"], "top": w["top_pt"]})
    return out


def mark_line_starts(words, tol=0.6):
    """A word starts a line if no other word on the same page shares its
    baseline band (bottom within tol pt) with a smaller x."""
    by_page = {}
    for i, w in enumerate(words):
        by_page.setdefault(w["page"], []).append(i)
    for idxs in by_page.values():
        for i in idxs:
            wi = words[i]
            wi["line_start"] = not any(
                abs(words[j]["bottom"] - wi["bottom"]) <= tol and words[j]["x"] < wi["x"] - 0.01
                for j in idxs if j != i)
    return words


def compare(oracle, ours):
    """Alignment and metrics between two word lists."""
    a = [w["norm"] for w in oracle]
    b = [w["norm"] for w in ours]
    sm = difflib.SequenceMatcher(a=a, b=b, autojunk=False)
    pairs = []
    for i, j, n in sm.get_matching_blocks():
        for k in range(n):
            pairs.append((i + k, j + k))
    deltas = []
    for i, j in pairs:
        o, u = oracle[i], ours[j]
        deltas.append({
            "word": o["text"], "oracle_page": o["page"], "ours_page": u["page"],
            "oracle_x": round(o["x"], 3), "ours_x": round(u["x"], 3), "dx": round(u["x"] - o["x"], 3),
            "oracle_bottom": round(o["bottom"], 3), "ours_bottom": round(u["bottom"], 3),
            "dy": round(u["bottom"] - o["bottom"], 3),
            "oracle_line_start": o["line_start"], "ours_line_start": u["line_start"],
        })
    same_page = [d for d in deltas if d["oracle_page"] == d["ours_page"]]
    res = {
        "oracle_words": len(a), "ours_words": len(b), "aligned_words": len(pairs),
        "sequence_equal": a == b, "similarity_ratio": round(sm.ratio(), 4),
        "aligned_on_same_page": len(same_page),
    }
    # Unmatched fragments, for the report.
    diffs = []
    for tag, i1, i2, j1, j2 in sm.get_opcodes():
        if tag != "equal":
            diffs.append({"op": tag, "oracle": a[i1:i2][:12], "ours": b[j1:j2][:12]})
    res["differences"] = diffs[:20]
    res["difference_count"] = len(diffs)
    if same_page:
        ax = [abs(d["dx"]) for d in same_page]
        ay = [abs(d["dy"]) for d in same_page]
        res["dx_abs_mean"] = round(statistics.fmean(ax), 3)
        res["dx_abs_max"] = round(max(ax), 3)
        res["dy_abs_mean"] = round(statistics.fmean(ay), 3)
        res["dy_abs_max"] = round(max(ay), 3)
        res["largest"] = sorted(same_page, key=lambda d: (d["dx"] ** 2 + d["dy"] ** 2), reverse=True)[:10]
        agree = sum(1 for d in same_page if d["oracle_line_start"] == d["ours_line_start"])
        res["line_start_agreement"] = round(agree / len(same_page), 4)
        res["oracle_line_starts"] = sum(1 for w in oracle if w["line_start"])
        res["ours_line_starts"] = sum(1 for w in ours if w["line_start"])
        res["line_start_disagreements"] = [
            {"word": d["word"], "oracle_starts_line": d["oracle_line_start"], "ours_starts_line": d["ours_line_start"]}
            for d in same_page if d["oracle_line_start"] != d["ours_line_start"]][:15]
    return res, deltas


def main():
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--pdflatex", required=True)
    ap.add_argument("--extract", required=True, help="built oracle_extract binary")
    ap.add_argument("--pdf-bin", required=True, help="built flashtex-pdf binary")
    ap.add_argument("--compiler", action="append", required=True, help="label=path to a built flashtex-compiler")
    ap.add_argument("--samples", required=True)
    ap.add_argument("--reports-dir", required=True)
    ap.add_argument("--scratch", default=None, help="keep intermediate PDFs/JSON here (default: temp dir)")
    ap.add_argument("--label", action="append", default=[], help="key=value recorded in the report header")
    ap.add_argument("--only", action="append", default=[], help="sample name(s) without .tex to run; default all")
    args = ap.parse_args()

    stamp = datetime.now(timezone.utc).strftime("%Y%m%dT%H%M%SZ")
    scratch = args.scratch or tempfile.mkdtemp(prefix="flashtex-oracle-")
    os.makedirs(scratch, exist_ok=True)
    os.makedirs(args.reports_dir, exist_ok=True)
    compilers = [c.split("=", 1) for c in args.compiler]
    samples = sorted(f for f in os.listdir(args.samples) if f.endswith(".tex") and (not args.only or f[:-4] in args.only))

    rc, out, _, _ = run([args.pdflatex, "--version"])
    pdflatex_version = out.decode().splitlines()[0] if rc == 0 else f"exit {rc}"
    commands = []
    results = []
    stripped = {}

    for sample in samples:
        name = sample[:-4]
        tex = open(os.path.join(args.samples, sample), encoding="utf-8").read()
        preamble, body = split_preamble(tex)
        stripped[name] = strip_comment_lines(preamble).strip()

        # (b) our pipeline, once per compiler (independent of oracle variant)
        ours = {}
        for label, path in compilers:
            req = {"protocol_version": 1, "id": f"oracle-{name}", "type": "compile",
                   "payload": {"project_id": "oracle", "revision": 1, "entry_path": "main.tex",
                               "documents": [{"path": "main.tex", "text": body}]}}
            rc, out, err, secs = run([path], stdin_data=(json.dumps(req, ensure_ascii=False) + "\n").encode("utf-8"))
            commands.append((f"{name}/{label}", f"{path} < compile-envelope", rc, secs))
            result_path = os.path.join(scratch, f"{name}.{label}.compile_result.json")
            open(result_path, "wb").write(out)
            entry = {"compiler": label, "compile_exit": rc, "compile_seconds": round(secs, 3)}
            try:
                env = json.loads(out.decode("utf-8"))
                entry["status"] = env["payload"].get("status")
                entry["diagnostics"] = [d["message"] for d in env["payload"].get("diagnostics", [])]
                entry["pages"] = [(p["width_pt"], p["height_pt"]) for p in env["payload"]["pages"]]
            except Exception as exc:
                entry["status"] = f"unparseable: {exc}"
            our_pdf = os.path.join(scratch, f"{name}.{label}.flashtex.pdf")
            rc, out2, err2, secs2 = run([args.pdf_bin, result_path, "--out", our_pdf, "--verify"])
            commands.append((f"{name}/{label}", f"{args.pdf_bin} {os.path.basename(result_path)} --out {os.path.basename(our_pdf)} --verify", rc, secs2))
            entry["pdf_exit"] = rc
            entry["pdf_warnings"] = err2.decode("utf-8", "replace").strip()[:500]
            if rc == 0:
                rc3, out3, err3, _ = run([args.extract, our_pdf])
                entry["extract_exit"] = rc3
                if rc3 == 0:
                    entry["doc"] = json.loads(out3.decode("utf-8"))
                    entry["words"] = mark_line_starts(words_of(entry["doc"]))
            ours[label] = entry

        # (a) oracle per variant
        for vkey, v in VARIANTS.items():
            vdir = os.path.join(scratch, f"{name}.{vkey}")
            os.makedirs(vdir, exist_ok=True)
            open(os.path.join(vdir, "main.tex"), "w", encoding="utf-8").write(v["preamble"] + body)
            rc, out, err, secs = run([args.pdflatex, "-interaction=batchmode", "-halt-on-error", "main.tex"], cwd=vdir)
            commands.append((f"{name}/{vkey}", f"cd {os.path.basename(vdir)} && {args.pdflatex} -interaction=batchmode -halt-on-error main.tex", rc, secs))
            oracle_pdf = os.path.join(vdir, "main.pdf")
            row = {"sample": name, "variant": vkey, "variant_title": v["title"], "apples_to_apples": v["apples"],
                   "pdflatex_exit": rc, "pdflatex_seconds": round(secs, 3), "comparisons": []}
            log = os.path.join(vdir, "main.log")
            if rc != 0 or not os.path.exists(oracle_pdf):
                row["pdflatex_log_tail"] = open(log, encoding="latin-1").read()[-1500:] if os.path.exists(log) else ""
                results.append(row)
                continue
            if os.path.exists(log):
                logtext = open(log, encoding="latin-1").read()
                row["overfull_boxes"] = len(re.findall(r"^Overfull \\hbox", logtext, re.M))
                row["latex_warnings"] = len(re.findall(r"LaTeX Warning", logtext))
            rc, out, err, _ = run([args.extract, oracle_pdf])
            if rc != 0:
                row["extract_error"] = err.decode("utf-8", "replace")[:300]
                results.append(row)
                continue
            odoc = json.loads(out.decode("utf-8"))
            owords = mark_line_starts(words_of(odoc))
            row["oracle_pages"] = [(p["width_pt"], p["height_pt"]) for p in odoc["pages"]]
            row["oracle_word_count"] = len(owords)
            for label, entry in ours.items():
                cmp_ = {"compiler": label, "status": entry.get("status"), "diagnostics": entry.get("diagnostics", []),
                        "pdf_exit": entry.get("pdf_exit"), "pdf_warnings": entry.get("pdf_warnings", "")}
                if "words" in entry:
                    opages = row["oracle_pages"]
                    upages = [(p["width_pt"], p["height_pt"]) for p in entry["doc"]["pages"]]
                    cmp_["ours_pages"] = upages
                    cmp_["page_count_equal"] = len(opages) == len(upages)
                    cmp_["mediabox_equal"] = opages == upages
                    metrics, deltas = compare(owords, entry["words"])
                    cmp_.update(metrics)
                    cmp_["deltas"] = deltas
                else:
                    cmp_["error"] = "no word boxes for our PDF"
                row["comparisons"].append(cmp_)
            results.append(row)

    # ------------------------------------------------------------ outputs
    json_path = os.path.join(args.reports_dir, f"oracle-{stamp}.json")
    md_path = os.path.join(args.reports_dir, f"oracle-{stamp}.md")
    with open(json_path, "w", encoding="utf-8") as f:
        json.dump({"generated_utc": stamp, "pdflatex": pdflatex_version, "labels": dict(l.split("=", 1) for l in args.label),
                   "compilers": dict(compilers), "variants": {k: v["preamble"] for k, v in VARIANTS.items()},
                   "stripped_preambles": stripped, "results": results,
                   "commands": [{"scope": s, "command": c, "exit": rc, "seconds": round(t, 3)} for s, c, rc, t in commands]},
                  f, separators=(",", ":"), ensure_ascii=False)  # compact: per-word deltas for every comparison

    L = []
    L.append("# FlashTeX vs pdflatex reference-oracle comparison\n")
    L.append(f"- Generated: {stamp}. pdflatex is a reference oracle only (issue #10); the product never runs it.")
    L.append(f"- Oracle: `{args.pdflatex}` = {pdflatex_version}")
    for l in args.label:
        k, v = l.split("=", 1)
        L.append(f"- {k}: {v}")
    L.append(f"- Compilers under test: " + ", ".join(f"`{k}` = `{v}`" for k, v in compilers))
    L.append(f"- PDF writer: `{args.pdf_bin}` (`--verify`)")
    L.append(f"- Word boxes: PDFKit via `oracle_extract` for both sides; x = glyph-box left, y = glyph-box bottom from page top (baseline + descent), points.")
    L.append(f"- Intermediate files: `{scratch}`; machine-readable deltas: `{os.path.basename(json_path)}`\n")
    L.append("## Oracle variants\n")
    for k, v in VARIANTS.items():
        L.append(f"- `{k}` ({'apples-to-apples' if v['apples'] else 'as-is default'}): {v['title']}\n\n  ```latex\n" +
                 "".join("  " + x + "\n" for x in v["preamble"].rstrip("\n").splitlines()) + "  ```")
    L.append("\n## What was stripped for the FlashTeX compiler\n")
    L.append("Everything before `\\begin{document}` (comment lines omitted here). The compiler renders "
             "`\\documentclass{article}` as the literal word `article` plus a diagnostic, so the preamble is removed rather than passed through.\n")
    for name, pre in stripped.items():
        L.append(f"- `{name}.tex`: `{pre or '(nothing)'}`")
    L.append("\n## Headline table\n")
    L.append("| Sample | Variant | Compiler | Pages (oracle/ours) | MediaBox equal | Words (oracle/ours/aligned) | Sequence equal | Similarity | mean\\|dx\\| | max\\|dx\\| | mean\\|dy\\| | max\\|dy\\| | Line-start agreement |")
    L.append("|---|---|---|---|---|---|---|---|---|---|---|---|---|")
    for r in results:
        if r["pdflatex_exit"] != 0:
            L.append(f"| {r['sample']} | {r['variant']} | - | pdflatex exit {r['pdflatex_exit']} | | | | | | | | | |")
            continue
        for c in r["comparisons"]:
            if "error" in c:
                L.append(f"| {r['sample']} | {r['variant']} | {c['compiler']} | {c['error']} | | | | | | | | | |")
                continue
            L.append("| {s} | {v} | {c} | {op}/{up} | {mb} | {ow}/{uw}/{aw} | {eq} | {sim} | {dxm} | {dxx} | {dym} | {dyx} | {ls} |".format(
                s=r["sample"], v=r["variant"], c=c["compiler"], op=len(r["oracle_pages"]), up=len(c["ours_pages"]),
                mb="yes" if c["mediabox_equal"] else "no", ow=c["oracle_words"], uw=c["ours_words"], aw=c["aligned_words"],
                eq="yes" if c["sequence_equal"] else "no", sim=c["similarity_ratio"],
                dxm=c.get("dx_abs_mean", "-"), dxx=c.get("dx_abs_max", "-"), dym=c.get("dy_abs_mean", "-"), dyx=c.get("dy_abs_max", "-"),
                ls=c.get("line_start_agreement", "-")))
    L.append("\n## Details\n")
    for r in results:
        L.append(f"### {r['sample']} / {r['variant']}\n")
        L.append(f"- pdflatex exit {r['pdflatex_exit']} in {r['pdflatex_seconds']} s; overfull hboxes: {r.get('overfull_boxes', '-')}; LaTeX warnings: {r.get('latex_warnings', '-')}")
        if r["pdflatex_exit"] != 0:
            L.append("\n```\n" + r.get("pdflatex_log_tail", "") + "\n```")
            continue
        L.append(f"- oracle pages: {r['oracle_pages']}; oracle words: {r['oracle_word_count']}")
        for c in r["comparisons"]:
            L.append(f"- **{c['compiler']}**: status `{c['status']}`, diagnostics {len(c['diagnostics'])}"
                     + (f" ({'; '.join(sorted(set(c['diagnostics']))[:4])})" if c["diagnostics"] else "")
                     + f"; flashtex-pdf exit {c['pdf_exit']}" + (f", warnings: `{c['pdf_warnings'][:200]}`" if c["pdf_warnings"] else ""))
            if "error" in c:
                continue
            L.append(f"  - ours pages: {c['ours_pages']}; aligned on same page: {c['aligned_on_same_page']}/{c['aligned_words']}; "
                     f"line starts oracle/ours: {c.get('oracle_line_starts', '-')}/{c.get('ours_line_starts', '-')}")
            if c["differences"]:
                L.append(f"  - word-sequence differences ({c['difference_count']}, first {len(c['differences'])}):")
                for d in c["differences"]:
                    L.append(f"    - {d['op']}: oracle {d['oracle']} vs ours {d['ours']}")
            if c.get("largest"):
                L.append("  - 10 largest per-word deltas (ours minus oracle, pt):")
                L.append("    | word | page o/u | oracle x | ours x | dx | oracle y | ours y | dy |")
                L.append("    |---|---|---|---|---|---|---|---|")
                for d in c["largest"]:
                    L.append(f"    | {d['word']} | {d['oracle_page']}/{d['ours_page']} | {d['oracle_x']} | {d['ours_x']} | {d['dx']} | {d['oracle_bottom']} | {d['ours_bottom']} | {d['dy']} |")
            if c.get("line_start_disagreements"):
                L.append(f"  - line-start disagreements (first {len(c['line_start_disagreements'])}): "
                         + ", ".join(f"`{d['word']}` (oracle {'starts' if d['oracle_starts_line'] else 'continues'}, ours {'starts' if d['ours_starts_line'] else 'continues'})"
                                     for d in c["line_start_disagreements"]))
        L.append("")
    L.append("## Commands run\n")
    L.append("| Scope | Command | Exit | Seconds |")
    L.append("|---|---|---|---|")
    for s, c, rc, t in commands:
        L.append(f"| {s} | `{c}` | {rc} | {t:.2f} |")
    open(md_path, "w", encoding="utf-8").write("\n".join(L) + "\n")
    print(f"report: {md_path}\njson:   {json_path}")
    # Non-zero only when a tool failed; layout disagreement is a finding, not a failure.
    failed = any(r["pdflatex_exit"] != 0 for r in results) or any(
        c.get("pdf_exit") not in (0, None) or "error" in c for r in results for c in r["comparisons"])
    return 1 if failed else 0


if __name__ == "__main__":
    sys.exit(main())
