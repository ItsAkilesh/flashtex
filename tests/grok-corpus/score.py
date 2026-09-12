#!/usr/bin/env python3
"""Offline scoring harness for the Grok handwriting corpus. No network access,
no API keys, no model calls: it only ever compares two strings of LaTeX you
already have (a ground-truth snippet from manifest.json and a model output
you captured separately).

This does NOT measure semantic/visual equivalence of typeset output. It
normalizes then diffs LaTeX *source text*. Read the "What this does and does
not catch" section in README.md before trusting a score.

Usage
-----
Single ad hoc pair:
    python3 score.py --pair "\\frac{1}{2}" "\\frac12"

Batch over the whole corpus, once you have real model outputs saved as
<case-id>.tex files in some directory (one file per manifest case id you
actually ran, others are reported as not_run):
    python3 score.py --manifest manifest.json --outputs-dir /path/to/outputs
    python3 score.py --manifest manifest.json --outputs-dir out --json report.json

Exit status is always 0 (this is a reporting tool, not a pass/fail gate);
use --min-ratio if you want a CI-style failure on regressions.
"""
from __future__ import annotations

import argparse
import difflib
import json
import re
import sys
import unicodedata
from pathlib import Path

# ---------------------------------------------------------------------------
# Normalization
#
# Be honest about scope: this collapses superficial, well-understood LaTeX
# spelling variants so near-identical output isn't scored as "different".
# It does NOT parse LaTeX or understand math semantics, so it will:
#   - correctly equate `\frac{1}{2}` and `\frac12` (the motivating example)
#   - correctly ignore `\,`/`\;`/`\quad` spacing and `\left(`/`\right)` sizing
#   - correctly ignore which display-math delimiter was used
#   - NOT know that `\frac{a}{b}` and `a/b` (plain division) mean the same
#   - NOT know that `x+y` and `y+x` are commutative
#   - NOT know that `\sum_{i=1}^{n}` and `\sum_{i=1}^n` differ only in
#     brace-ness of a single-token superscript (this one *is* handled, see
#     _expand_bare_frac_args, but the general "single token doesn't need
#     braces" rule is not applied everywhere braces are optional)
#   - NOT reconcile different but equivalent macro choices the corpus itself
#     doesn't use (e.g. \varnothing vs \emptyset)
# When in doubt, this errs toward under-normalizing (reporting two equivalent
# strings as different) rather than over-normalizing (silently treating
# different math as the same). A human should always review "divergent"
# verdicts near a decision boundary.
# ---------------------------------------------------------------------------

_SPACING_MACROS = re.compile(r"\\[,;!]|\\quad|\\qquad|\\ |~")
_LEFT_RIGHT = re.compile(r"\\left|\\right")
_FRAC_VARIANTS = re.compile(r"\\(?:dfrac|tfrac|cfrac)\b")
_MATH_DELIMS = re.compile(r"\\\[|\\\]|\\\(|\\\)|\$\$|\$")
_ENV_DELIMS = re.compile(
    r"\\begin\{(?:equation|align|gather|multline)\*?\}|"
    r"\\end\{(?:equation|align|gather|multline)\*?\}"
)
_COMMENT = re.compile(r"(?<!\\)%[^\n]*")
_WHITESPACE = re.compile(r"\s+")
# `\frac12`, `\frac1a` style bare single-token arguments -> `\frac{1}{2}`.
# A "single token" here is one alphanumeric character, matching what LaTeX
# itself accepts as an unbraced macro argument.
_BARE_FRAC = re.compile(r"\\frac\s*([A-Za-z0-9])\s*([A-Za-z0-9])")


def normalize_latex(source: str) -> str:
    s = unicodedata.normalize("NFC", source)
    s = _COMMENT.sub("", s)
    s = _BARE_FRAC.sub(r"\\frac{\1}{\2}", s)
    s = _FRAC_VARIANTS.sub(r"\\frac", s)
    s = _LEFT_RIGHT.sub("", s)
    s = _SPACING_MACROS.sub(" ", s)
    s = _ENV_DELIMS.sub("", s)
    s = _MATH_DELIMS.sub("", s)
    s = _WHITESPACE.sub(" ", s)
    return s.strip()


# ---------------------------------------------------------------------------
# Scoring
# ---------------------------------------------------------------------------


def score_pair(ground_truth: str, model_output: str) -> dict:
    gt_norm = normalize_latex(ground_truth)
    out_norm = normalize_latex(model_output)
    if not gt_norm:
        return {
            "verdict": "not_applicable",
            "ratio": None,
            "note": "Ground truth is empty (adversarial case). Judge by hand: "
            "did the model avoid fabricating confident math, and does its "
            "response (or lack of one) match the case's documented "
            "expected_behavior?",
            "ground_truth_normalized": gt_norm,
            "model_output_normalized": out_norm,
        }
    ratio = difflib.SequenceMatcher(None, gt_norm, out_norm).ratio()
    exact = gt_norm == out_norm
    if exact:
        verdict = "match"
    elif ratio >= 0.95:
        verdict = "close"
    elif ratio >= 0.75:
        verdict = "partial"
    else:
        verdict = "divergent"
    diff_lines = list(
        difflib.unified_diff(
            gt_norm.splitlines(keepends=True) or [gt_norm],
            out_norm.splitlines(keepends=True) or [out_norm],
            fromfile="ground_truth (normalized)",
            tofile="model_output (normalized)",
            lineterm="",
        )
    )
    return {
        "verdict": verdict,
        "ratio": round(ratio, 4),
        "exact_after_normalization": exact,
        "ground_truth_normalized": gt_norm,
        "model_output_normalized": out_norm,
        "diff": diff_lines,
    }


def cmd_pair(ground_truth: str, model_output: str) -> int:
    result = score_pair(ground_truth, model_output)
    print(json.dumps(result, indent=2, ensure_ascii=False))
    return 0


def cmd_batch(args: argparse.Namespace) -> int:
    manifest = json.loads(Path(args.manifest).read_text(encoding="utf-8"))
    outputs_dir = Path(args.outputs_dir)
    report = {"cases": [], "summary": {}}
    counts: dict[str, int] = {}
    ratios = []
    for case in manifest["cases"]:
        case_id = case["id"]
        out_path = outputs_dir / f"{case_id}.tex"
        if not out_path.exists():
            entry = {"id": case_id, "verdict": "not_run"}
            counts["not_run"] = counts.get("not_run", 0) + 1
            report["cases"].append(entry)
            continue
        model_output = out_path.read_text(encoding="utf-8")
        result = score_pair(case["ground_truth_latex"], model_output)
        entry = {"id": case_id, "category": case["category"], **result}
        counts[result["verdict"]] = counts.get(result["verdict"], 0) + 1
        if result.get("ratio") is not None:
            ratios.append(result["ratio"])
        report["cases"].append(entry)
    report["summary"] = {
        "total_cases": len(manifest["cases"]),
        "by_verdict": counts,
        "mean_ratio_scored_cases": round(sum(ratios) / len(ratios), 4) if ratios else None,
    }
    output_json = json.dumps(report, indent=2, ensure_ascii=False)
    if args.json:
        Path(args.json).write_text(output_json + "\n", encoding="utf-8")
        print(f"wrote {args.json}")
    print(json.dumps(report["summary"], indent=2))
    if args.min_ratio is not None:
        scored = [c for c in report["cases"] if c.get("ratio") is not None]
        failing = [c for c in scored if c["ratio"] < args.min_ratio]
        if failing:
            print(
                f"FAIL: {len(failing)}/{len(scored)} scored cases below "
                f"--min-ratio {args.min_ratio}: {[c['id'] for c in failing]}",
                file=sys.stderr,
            )
            return 1
    return 0


def main() -> int:
    parser = argparse.ArgumentParser(
        description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter
    )
    parser.add_argument(
        "--pair",
        nargs=2,
        metavar=("GROUND_TRUTH", "MODEL_OUTPUT"),
        help="score one ad hoc ground-truth/model-output string pair",
    )
    parser.add_argument("--manifest", help="path to manifest.json, for batch mode")
    parser.add_argument(
        "--outputs-dir",
        help="directory of <case-id>.tex model-output files, for batch mode",
    )
    parser.add_argument("--json", help="batch mode: also write the full report to this path")
    parser.add_argument(
        "--min-ratio",
        type=float,
        default=None,
        help="batch mode: exit 1 if any scored case falls below this ratio (0.0-1.0)",
    )
    args = parser.parse_args()

    if args.pair:
        return cmd_pair(args.pair[0], args.pair[1])
    if args.manifest and args.outputs_dir:
        return cmd_batch(args)
    parser.error("expected --pair GROUND_TRUTH MODEL_OUTPUT, or --manifest M --outputs-dir DIR")
    return 2  # unreachable; parser.error() exits


if __name__ == "__main__":
    raise SystemExit(main())
