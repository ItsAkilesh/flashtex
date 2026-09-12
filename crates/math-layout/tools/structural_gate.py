#!/usr/bin/env python3
r"""Structural gate for the math visual corpus (crates/math-layout/fixtures/visual).

ORACLE TOOLING ONLY. For every corpus fixture this script:
  1. compiles the fixture as-is with the reference pdflatex (BasicTeX,
     /Library/TeX/texbin) in a scratch directory — the fixture preamble is the
     harness's 12pt Latin Modern "lm" preamble plus uncompressed streams;
  2. reads every glyph origin (font, code, x, baseline) and every rule (centre
     line, length, thickness) from the PDF content stream, using the PDF's own
     /Widths for advances (see oracle_compare.py);
  3. lays out the same case with `flashtex-math-corpus --runs` (CmMathMetrics
     latex_12pt) and compares: absolute page placement of the first glyph, and
     every glyph/rule relative to that anchor;
  4. checks the per-case maxima against fixtures/visual/thresholds.json and,
     with --regress <baseline.json>, against a previously recorded baseline
     (a case fails if it exceeds its threshold or worsens by more than the
     baseline tolerance). Exit 0 = pass, 1 = fail, 2 = usage/oracle error.

Pinned oracle. Step 1 needs pdflatex; step 2's output can be pinned instead:
fixtures/visual/oracle-geometry.json holds every reference glyph origin and
rule per case together with the fixture SHA-256 and the pdfTeX banner of the
run that produced it. By default the gate compiles with pdflatex when it is
installed and otherwise falls back to the pinned geometry (the report says
which). `--oracle pinned` forces the pinned file, `--oracle pdflatex` forces a
fresh compile, and `--pin-oracle DIR` re-extracts the pinned file from oracle
PDFs in DIR/<case>/<case>.pdf (as left by a pdflatex run in --scratch). A
pinned case is refused when its fixture SHA-256 no longer matches.

Usage:
  python3 tools/structural_gate.py [--scratch DIR] [--report OUT.md]
      [--regress BASELINE.json] [--write-baseline OUT.json] [--case ID ...]
      [--oracle auto|pinned|pdflatex] [--pin-oracle DIR]
"""
import argparse
import hashlib
import json
import os
import re
import subprocess
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
CRATE = os.path.dirname(HERE)
sys.path.insert(0, HERE)
import oracle_compare as oc  # noqa: E402

TEXBIN = "/Library/TeX/texbin"
BP_PER_TEX_PT = 72.0 / 72.27
PAGE_H = 792.0
PINNED = os.path.join(CRATE, "fixtures", "visual", "oracle-geometry.json")


def sha256_file(path):
    return hashlib.sha256(open(path, "rb").read()).hexdigest()


def pdftex_banner(log_path):
    try:
        return open(log_path, errors="replace").readline().strip()
    except OSError:
        return None


def pin_oracle(pdf_dir, fixtures, out):
    """Extract the reference geometry of every oracle PDF under pdf_dir into `out`."""
    pinned = {"_comment": ("Pinned structural oracle for tools/structural_gate.py: glyph origins (font, code, x, "
                           "y_top) and rules (x, y_top centre line, w, h) in PDF points, top-left page origin, read "
                           "from the pdflatex PDF of each fixture as committed (sha256). Oracle output, never "
                           "product input; regenerate with --pin-oracle after any fixture change."),
              "cases": {}}
    for f in sorted(os.listdir(fixtures)):
        if not f.endswith(".tex"):
            continue
        name = f[:-4]
        pdf = os.path.join(pdf_dir, name, name + ".pdf")
        if not os.path.exists(pdf):
            print(f"pin: no oracle PDF for {name} at {pdf}", file=sys.stderr)
            return 2
        glyphs, rules = reference_geometry(pdf)
        pinned["cases"][name] = {
            "fixture_sha256": sha256_file(os.path.join(fixtures, f)),
            "pdftex": pdftex_banner(os.path.join(pdf_dir, name, name + ".log")),
            "pdf_sha256": sha256_file(pdf),
            "glyphs": [{"font": g["font"], "code": g["code"], "x": round(g["x"], 6), "y_top": round(g["y_top"], 6)}
                       for g in glyphs],
            "rules": [{"x": round(r["x"], 6), "y_top": round(r["y_top"], 6), "w": round(r["w"], 6), "h": round(r["h"], 6)}
                      for r in rules],
        }
    json.dump(pinned, open(out, "w"), indent=1, sort_keys=True)
    print(f"pinned {len(pinned['cases'])} cases to {out}")
    return 0


def pinned_geometry(pinned, name, tex):
    case = pinned["cases"].get(name)
    if case is None:
        raise RuntimeError(f"{name}: not in the pinned oracle; run --pin-oracle after a pdflatex run")
    actual = sha256_file(tex)
    if case["fixture_sha256"] != actual:
        raise RuntimeError(f"{name}: fixture changed since the oracle was pinned "
                           f"({actual[:12]} vs {case['fixture_sha256'][:12]}); regenerate with pdflatex")
    glyphs = [dict(g) for g in case["glyphs"]]
    rules = [dict(r) for r in case["rules"]]
    return glyphs, rules


def cm_name(base):
    """Map a Latin Modern PostScript name (or CM TFM name) to the CM TFM name."""
    b = base.lower()
    m = re.match(r"lmroman(\d+)", b)
    if m:
        return "cmr" + m.group(1)
    m = re.match(r"lmmathitalic(\d+)", b)
    if m:
        return "cmmi" + m.group(1)
    m = re.match(r"lmmathsymbols(\d+)", b)
    if m:
        return "cmsy" + m.group(1)
    if b.startswith("lmmathextension"):
        return "cmex10"
    return re.sub(r"-.*$", "", b)


def oracle_pdf(tex, scratch):
    name = os.path.basename(tex)[:-4]
    out = os.path.join(scratch, name)
    os.makedirs(out, exist_ok=True)
    r = subprocess.run([os.path.join(TEXBIN, "pdflatex"), "-interaction=batchmode", "-halt-on-error",
                        "-output-directory", out, tex], capture_output=True, text=True)
    pdf = os.path.join(out, name + ".pdf")
    if r.returncode != 0 or not os.path.exists(pdf):
        raise RuntimeError(f"pdflatex failed for {name}: see {out}/{name}.log")
    return pdf


def reference_geometry(pdf):
    data, objs = oc.load_pdf(pdf)
    table = oc.fonts(objs)
    table = {k: (cm_name(v[0]), v[1], v[2]) for k, v in table.items()}
    stream = re.search(rb"stream\r?\n(.*?)\r?\nendstream", data, re.S).group(1)
    glyphs, rules = oc.parse_content(stream, table)
    # top-down page coordinates in bp
    for g in glyphs:
        g["y_top"] = PAGE_H - g["y"]
    for r in rules:
        r["y_top"] = PAGE_H - r["y_center"]
    return glyphs, rules


def compare_case(ours, ref_glyphs, ref_rules):
    """Returns (rows, rule_rows, summary)."""
    first = ours["glyphs"][0]
    ox0 = ours["x_pt"] * BP_PER_TEX_PT
    oy0 = ours["baseline_pt"] * BP_PER_TEX_PT
    # absolute placement of our first glyph vs the same glyph in the reference
    cand = [g for g in ref_glyphs if g["font"] == first["font"] and g["code"] == first["gid"]]
    if not cand:
        return None, None, {"error": f"anchor glyph {first['font']}/{first['gid']} not in reference"}
    ref_first = min(cand, key=lambda g: abs(g["x"] - (ox0 + first["x"] * BP_PER_TEX_PT)))
    place_dx = (ox0 + first["x"] * BP_PER_TEX_PT) - ref_first["x"]
    place_dy = (oy0 + first["baseline"] * BP_PER_TEX_PT) - ref_first["y_top"]
    ax, ay = ref_first["x"], ref_first["y_top"]
    used = set()
    rows, worst, missing = [], 0.0, 0
    for g in ours["glyphs"]:
        gx = (g["x"] - first["x"]) * BP_PER_TEX_PT
        gy = (g["baseline"] - first["baseline"]) * BP_PER_TEX_PT
        cands = [(i, r) for i, r in enumerate(ref_glyphs)
                 if i not in used and r["font"] == g["font"] and r["code"] == g["gid"]]
        if not cands:
            rows.append((g["ch"], g["font"], g["gid"], gx, gy, None, None))
            missing += 1
            continue
        i, r = min(cands, key=lambda c: abs(c[1]["x"] - ax - gx) + abs(c[1]["y_top"] - ay - gy))
        used.add(i)
        dx, dy = gx - (r["x"] - ax), gy - (r["y_top"] - ay)
        worst = max(worst, abs(dx), abs(dy))
        rows.append((g["ch"], g["font"], g["gid"], gx, gy, dx, dy))
    extra = len(ref_glyphs) - len(used)
    rule_rows = []
    ref_sorted = sorted(ref_rules, key=lambda r: (r["y_top"], r["x"]))
    our_sorted = sorted(ours["rules"], key=lambda r: (r["y"], r["x"]))
    for o, r in zip(our_sorted, ref_sorted):
        ox = (o["x"] - first["x"]) * BP_PER_TEX_PT
        oyc = (o["y"] + o["h"] / 2.0 - first["baseline"]) * BP_PER_TEX_PT
        d = (ox - (r["x"] - ax), oyc - (r["y_top"] - ay), o["w"] * BP_PER_TEX_PT - r["w"], o["h"] * BP_PER_TEX_PT - r["h"])
        worst = max(worst, *[abs(v) for v in d])
        rule_rows.append((ox, oyc, o["w"] * BP_PER_TEX_PT, o["h"] * BP_PER_TEX_PT) + d)
    summary = {
        "glyphs": len(ours["glyphs"]), "rules": len(ours["rules"]),
        "reference_glyphs": len(ref_glyphs), "reference_rules": len(ref_rules),
        "missing_glyphs": missing, "unmatched_reference_glyphs": extra,
        "rule_count_mismatch": len(ours["rules"]) != len(ref_rules),
        "max_abs_delta_bp": round(worst, 4),
        "placement_dx_bp": round(place_dx, 4), "placement_dy_bp": round(place_dy, 4),
        "limitations": ours.get("limitations", []),
    }
    return rows, rule_rows, summary


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--scratch", default=os.path.join(os.environ.get("TMPDIR", "/tmp"), "flashtex-math-structural"))
    ap.add_argument("--report")
    ap.add_argument("--regress")
    ap.add_argument("--write-baseline")
    ap.add_argument("--case", action="append")
    ap.add_argument("--runs-json", help="precomputed `flashtex-math-corpus --runs` output")
    ap.add_argument("--oracle", choices=["auto", "pinned", "pdflatex"], default="auto")
    ap.add_argument("--pin-oracle", metavar="DIR", help="extract fixtures/visual/oracle-geometry.json from DIR/<case>/<case>.pdf and exit")
    a = ap.parse_args()
    fixtures = os.path.join(CRATE, "fixtures", "visual")
    if a.pin_oracle:
        return pin_oracle(a.pin_oracle, fixtures, PINNED)
    have_tex = os.path.exists(os.path.join(TEXBIN, "pdflatex"))
    if a.oracle == "pdflatex" and not have_tex:
        print(f"oracle error: {TEXBIN}/pdflatex is not installed", file=sys.stderr)
        return 2
    use_pinned = a.oracle == "pinned" or (a.oracle == "auto" and not have_tex)
    pinned = json.load(open(PINNED)) if use_pinned else None
    thresholds = json.load(open(os.path.join(fixtures, "thresholds.json")))
    if a.runs_json:
        runs = json.load(open(a.runs_json))
    else:
        r = subprocess.run(["cargo", "run", "-q", "--manifest-path", os.path.join(CRATE, "Cargo.toml"),
                            "--bin", "flashtex-math-corpus", "--", "--runs"], capture_output=True, text=True)
        if r.returncode != 0:
            print(r.stderr, file=sys.stderr)
            return 2
        runs = json.loads(r.stdout)
    baseline = json.load(open(a.regress)) if a.regress else None
    os.makedirs(a.scratch, exist_ok=True)
    lines, results, failed = [], {}, []
    for ours in runs:
        name = ours["name"]
        if a.case and name not in a.case:
            continue
        tex = os.path.join(fixtures, name + ".tex")
        try:
            if pinned is not None:
                ref_glyphs, ref_rules = pinned_geometry(pinned, name, tex)
            else:
                pdf = oracle_pdf(tex, a.scratch)
                ref_glyphs, ref_rules = reference_geometry(pdf)
        except Exception as e:  # noqa: BLE001
            print(f"oracle error: {e}", file=sys.stderr)
            return 2
        rows, rule_rows, summary = compare_case(ours, ref_glyphs, ref_rules)
        limit = dict(thresholds.get("default", {}))
        limit.update(thresholds.get(name, {}))
        verdict = []
        if "error" in summary:
            verdict.append(summary["error"])
        else:
            if summary["max_abs_delta_bp"] > limit["max_abs_delta_bp"]:
                verdict.append(f"max |Δ| {summary['max_abs_delta_bp']} > {limit['max_abs_delta_bp']}")
            if abs(summary["placement_dx_bp"]) > limit["max_placement_delta_bp"] or abs(summary["placement_dy_bp"]) > limit["max_placement_delta_bp"]:
                verdict.append(f"placement ({summary['placement_dx_bp']}, {summary['placement_dy_bp']}) > {limit['max_placement_delta_bp']}")
            if summary["missing_glyphs"] or summary["unmatched_reference_glyphs"] or summary["rule_count_mismatch"]:
                verdict.append("glyph/rule inventory differs")
            if baseline and name in baseline:
                prev = baseline[name]["max_abs_delta_bp"]
                if summary["max_abs_delta_bp"] > prev + thresholds.get("regress_tolerance_bp", 0.001):
                    verdict.append(f"regressed: {summary['max_abs_delta_bp']} vs baseline {prev}")
        summary["pass"] = not verdict
        summary["verdict"] = verdict
        results[name] = summary
        if verdict:
            failed.append(name)
        lines.append(f"\n### {name}\n")
        lines.append(f"- glyphs {summary.get('glyphs')} (reference {summary.get('reference_glyphs')}), rules {summary.get('rules')} (reference {summary.get('reference_rules')})")
        if "error" not in summary:
            lines.append(f"- placement of the first glyph: Δx {summary['placement_dx_bp']:+.4f} bp, Δy {summary['placement_dy_bp']:+.4f} bp")
            lines.append(f"- max |Δ| over glyph origins and rule geometry: {summary['max_abs_delta_bp']:.4f} bp (threshold {limit['max_abs_delta_bp']})")
        lines.append(f"- limitations: {summary.get('limitations') or 'none'}")
        lines.append(f"- verdict: {'PASS' if summary['pass'] else 'FAIL: ' + '; '.join(verdict)}")
        if rows:
            lines.append("\n| glyph | font | gid | x | baseline | Δx | Δy |")
            lines.append("|---|---|---|---|---|---|---|")
            for ch, font, gid, gx, gy, dx, dy in rows:
                dxs = "missing" if dx is None else f"{dx:+.4f}"
                dys = "" if dy is None else f"{dy:+.4f}"
                lines.append(f"| {ch} | {font} | {gid} | {gx:.4f} | {gy:.4f} | {dxs} | {dys} |")
        if rule_rows:
            lines.append("\n| rule x | centre y | w | h | Δx | Δy | Δw | Δh |")
            lines.append("|---|---|---|---|---|---|---|---|")
            for row in rule_rows:
                lines.append("| " + " | ".join(f"{v:+.4f}" if i >= 4 else f"{v:.4f}" for i, v in enumerate(row)) + " |")
    header = [
        "# Structural gate report",
        "",
        f"Cases: {len(results)}; failed: {len(failed)} {failed if failed else ''}",
        f"Largest max |Δ| over the corpus: {max((r.get('max_abs_delta_bp', 0) for r in results.values()), default=0):.4f} bp",
        ("Reference: " + (f"pinned oracle geometry ({os.path.relpath(PINNED, CRATE)}; pdflatex not run" +
                          (" — not installed" if not have_tex else "") + ")" if pinned is not None else
                          "pdflatex run now") +
         " on the fixture as committed (12pt article, OT1 Computer Modern); Δ = ours − reference in PDF points; positions relative to the first glyph of each case except the placement row."),
    ] + ([f"Pinned pdfTeX: {next(iter(pinned['cases'].values()))['pdftex']}"] if pinned is not None else [])
    text = "\n".join(header + lines) + "\n"
    if a.report:
        open(a.report, "w").write(text)
    else:
        sys.stdout.write(text)
    if a.write_baseline:
        json.dump(results, open(a.write_baseline, "w"), indent=1, sort_keys=True)
    return 1 if failed else 0


if __name__ == "__main__":
    sys.exit(main())
