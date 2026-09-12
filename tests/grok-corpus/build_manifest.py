#!/usr/bin/env python3
"""Assembles manifest.json from the images generate.sh just produced plus the
ground-truth LaTeX embedded in each case's .tex source (between the
`% GT-BEGIN` / `% GT-END` markers). This is the single source of truth for
corpus case metadata; generate.sh calls it as its last step. Stdlib only.
"""
import hashlib
import json
import re
import struct
from pathlib import Path

CORPUS_DIR = Path(__file__).resolve().parent
CASES_DIR = CORPUS_DIR / "cases"
IMAGES_DIR = CORPUS_DIR / "images"
MANIFEST_PATH = CORPUS_DIR / "manifest.json"

GT_RE = re.compile(r"%\s*GT-BEGIN\s*\n(.*?)\n%\s*GT-END", re.DOTALL)


def ground_truth_from_tex(case_id: str) -> str:
    """Concatenates every GT-BEGIN/GT-END block in the case's .tex file, in
    document order, joined by a blank line. Some cases (e.g. side-by-side
    minipages) need more than one block so that layout-only markup between
    the ground-truth content is excluded.
    """
    text = (CASES_DIR / f"{case_id}.tex").read_text(encoding="utf-8")
    blocks = GT_RE.findall(text)
    if not blocks:
        raise SystemExit(f"no GT-BEGIN/GT-END markers found in {case_id}.tex")
    return "\n\n".join(block.strip("\n") for block in blocks)


def image_dimensions(path: Path) -> tuple[int, int]:
    data = path.read_bytes()
    if data[:8] == b"\x89PNG\r\n\x1a\n":
        width, height = struct.unpack(">II", data[16:24])
        return width, height
    if data[:2] == b"\xff\xd8":
        i = 2
        while i < len(data):
            if data[i] != 0xFF:
                i += 1
                continue
            marker = data[i + 1]
            if marker in (0xD8, 0xD9):
                i += 2
                continue
            seg_len = struct.unpack(">H", data[i + 2 : i + 4])[0]
            # SOF0..SOF3 / SOF5..SOF7 / SOF9..SOF11 / SOF13..SOF15 carry dimensions.
            if 0xC0 <= marker <= 0xCF and marker not in (0xC4, 0xC8, 0xCC):
                height, width = struct.unpack(">HH", data[i + 5 : i + 9])
                return width, height
            i += 2 + seg_len
        raise ValueError(f"could not find SOF marker in JPEG {path}")
    raise ValueError(f"unrecognized image format for {path}")


def file_record(image_name: str) -> dict:
    path = IMAGES_DIR / image_name
    width, height = image_dimensions(path)
    sha256 = hashlib.sha256(path.read_bytes()).hexdigest()
    size_bytes = path.stat().st_size
    mime = "image/jpeg" if image_name.lower().endswith((".jpg", ".jpeg")) else "image/png"
    return {
        "image": f"images/{image_name}",
        "mime_type": mime,
        "width": width,
        "height": height,
        "size_bytes": size_bytes,
        "sha256": sha256,
    }


# (case_id, image_filename, category, engine_or_None, description)
BASELINE_CASES = [
    (
        "clean-quadratic-formula",
        "clean-quadratic-formula.png",
        "baseline-clean",
        "pdflatex",
        "Single clean typeset equation, matching the scenario the Commander already "
        "validated with one live call against a synthetic rendered-math image.",
    ),
    (
        "clean-integral-unicode",
        "clean-integral-unicode.png",
        "baseline-clean",
        "xelatex+unicode-math",
        "Definite integral typed with literal Unicode math symbols (integral sign, "
        "pi, theta) via unicode-math and a system OpenType math font, to exercise "
        "the unicode-math path this machine's MacTeX confirmed working.",
    ),
    (
        "clean-multiline-derivation",
        "clean-multiline-derivation.png",
        "structural-hard",
        "pdflatex",
        "Three-step algebraic derivation in an align* environment: tests whether a "
        "conversion preserves line structure and alignment, not just a single formula.",
    ),
    (
        "clean-matrix-determinant",
        "clean-matrix-determinant.jpg",
        "structural-hard",
        "pdflatex",
        "3x3 matrix determinant expansion; stresses pmatrix/array layout and a longer "
        "single-line equation. Stored as JPEG to exercise the CaptureImage image/jpeg path.",
    ),
    (
        "clean-nested-fraction-scripts",
        "clean-nested-fraction-scripts.png",
        "structural-hard",
        "pdflatex",
        "Doubly-nested continued fraction plus stacked sub/superscripts "
        "(x^{y^{z}}, a_{i_{j_{k}}}^{2}); stresses deep nesting that is easy to "
        "flatten or truncate incorrectly.",
    ),
    (
        "clean-mixed-prose-math",
        "clean-mixed-prose-math.jpg",
        "structural-hard",
        "pdflatex",
        "A homework-style prose paragraph with inline math ($y=x^3-3x$, $y'(x)$, "
        "$y''(x)$); tests whether prose and math are both preserved faithfully "
        "instead of the model dropping the prose or over-mathifying it.",
    ),
    (
        "clean-multicolumn-two-equations",
        "clean-multicolumn-two-equations.png",
        "structural-hard",
        "pdflatex+multicol",
        "Two-column layout with one unrelated equation per column; tests whether a "
        "column-major or row-major (visually left-to-right across columns) reading "
        "order is used, and whether the two are kept distinct.",
    ),
    (
        "clean-two-problems-page",
        "clean-two-problems-page.jpg",
        "structural-hard",
        "pdflatex",
        "One page containing two separately labeled problems with independent "
        "equations; tests whether a conversion keeps them separate instead of "
        "merging into one derivation.",
    ),
    (
        "adversarial-prose-no-math",
        "adversarial-prose-no-math.png",
        "adversarial",
        "pdflatex",
        "A page of ordinary English prose containing no mathematics at all.",
    ),
]

# (case_id, image_filename, description, expected_behavior)
SYNTHETIC_ADVERSARIAL_CASES = [
    (
        "adversarial-blank-page",
        "adversarial-blank-page.png",
        "A featureless solid-white image: no ink, no structure, nothing to transcribe.",
        "No faithful LaTeX transcription exists. The bridge's own Proposal schema "
        "requires 1-65536 non-NUL bytes of latex (see Proposal::validate in "
        "crates/bridge/src/lib.rs), so a model that (correctly) has nothing to say "
        "cannot return an empty proposal; the only schema-honest outcomes are a "
        "content:\"refusal\" response (handled in grok::parse_response as "
        "provider_refusal) or a proposal with placeholder/ambiguous content. A "
        "model that instead emits confident, unrelated LaTeX is hallucinating and "
        "that failure mode is exactly what this case is for catching.",
    ),
    (
        "adversarial-pure-noise",
        "adversarial-pure-noise.png",
        "Uniform random RGB noise, no rendered content of any kind.",
        "Same reasoning as adversarial-blank-page: nothing to transcribe. A "
        "trustworthy pipeline reports a refusal or high-ambiguity proposal, never "
        "invented mathematics.",
    ),
]

# (id_suffix, corpus_augment op, human-readable params)
DEGRADATIONS = [
    ("rotate6", {"type": "rotate", "params": {"degrees": 6}}, "png"),
    ("rotate15", {"type": "rotate", "params": {"degrees": 15}}, "png"),
    ("perspective", {"type": "perspective", "params": {"strength": 0.6}}, "png"),
    ("low-contrast", {"type": "contrast", "params": {"delta": -55}}, "png"),
    ("lighting-gradient", {"type": "lighting-gradient", "params": {"strength": 0.6}}, "png"),
    ("jpeg-artifacts", {"type": "jpeg-recompress", "params": {"quality": 8}}, "jpg"),
    ("downscale-blur", {"type": "downscale-blur", "params": {"factor": 0.28}}, "png"),
    ("noise", {"type": "additive-noise", "params": {"amount": 30, "seed": 42}}, "png"),
]

DEGRADATION_SOURCES = ["clean-quadratic-formula", "clean-multiline-derivation"]


def build() -> dict:
    cases = []
    ground_truth_by_id: dict[str, str] = {}

    for case_id, image_name, category, engine, description in BASELINE_CASES:
        gt = ground_truth_from_tex(case_id)
        ground_truth_by_id[case_id] = gt
        record = {
            "id": case_id,
            "category": category,
            "engine": engine,
            "source_tex": f"cases/{case_id}.tex",
            "ground_truth_latex": gt,
            "description": description,
            "degradation": None,
            "source_case": None,
        }
        record.update(file_record(image_name))
        cases.append(record)

    for case_id, image_name, description, expected_behavior in SYNTHETIC_ADVERSARIAL_CASES:
        record = {
            "id": case_id,
            "category": "adversarial",
            "engine": "corpus_augment (synthetic, no LaTeX)",
            "source_tex": None,
            "ground_truth_latex": "",
            "description": description,
            "expected_behavior": expected_behavior,
            "degradation": None,
            "source_case": None,
        }
        record.update(file_record(image_name))
        cases.append(record)

    for source in DEGRADATION_SOURCES:
        base_desc = next(d for c, _, _, _, d in BASELINE_CASES if c == source)
        for suffix, degradation, ext in DEGRADATIONS:
            case_id = f"degraded-{source}-{suffix}"
            image_name = f"{case_id}.{ext}"
            record = {
                "id": case_id,
                "category": "degraded-photo",
                "engine": "corpus_augment",
                "source_tex": f"cases/{source}.tex",
                "ground_truth_latex": ground_truth_by_id[source],
                "description": (
                    f"{base_desc} Degraded via {degradation['type']} "
                    f"({degradation['params']}) to simulate a realistic phone-camera "
                    "capture rather than a clean scan."
                ),
                "degradation": degradation,
                "source_case": source,
            }
            record.update(file_record(image_name))
            cases.append(record)

    return {
        "schema_version": 1,
        "purpose": (
            "Offline, reproducible image corpus for measuring Grok handwriting-to-"
            "LaTeX conversion quality (crates/bridge CaptureImage -> Converter -> "
            "Proposal). Every image here is machine-typeset with MacTeX from known "
            "LaTeX, so ground_truth_latex is exact, not eyeballed. No image was "
            "captured from or matched against a real handwriting sample."
        ),
        "generated_by": "tests/grok-corpus/generate.sh",
        "scoring": "tests/grok-corpus/score.py",
        "caveat": (
            "This manifest makes conversion quality MEASURABLE. It does not itself "
            "prove Grok performs well on real handwriting: no case here is an actual "
            "photograph of handwriting, and no live Grok call has been made against "
            "this corpus."
        ),
        "cases": cases,
    }


def main() -> None:
    manifest = build()
    MANIFEST_PATH.write_text(json.dumps(manifest, indent=2, ensure_ascii=False) + "\n", encoding="utf-8")
    print(f"wrote {len(manifest['cases'])} cases to {MANIFEST_PATH}")


if __name__ == "__main__":
    main()
