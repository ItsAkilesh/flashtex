#!/usr/bin/env python3
"""Box-command oracle: pdflatex word and rule positions vs flashtex-render.

Oracle tooling only; pdflatex never runs in the product path or in cargo.

    # regenerate the pinned references (needs TeX Live)
    python3 oracle.py refs --texbin /Library/TeX/texbin
    # measure a flashtex-render build against them
    python3 oracle.py check --render path/to/flashtex-render --fonts apps/mac/Fonts

Fixtures are `crates/render-pipeline/fixtures/boxes/*.tex`; references are
written to `fixtures/boxes/refs/*.json`. Word formation, rule extraction
(filled `re` paths and single stroked `m`/`l` segments, continuing rules
merged) and the pass criteria are those of
`crates/compiler/tests/tabular_corpus/oracle.py`, which this script runs
with the fixture and reference directories pointed here: one page, every
word aligned and within 0.5 bp of its reference origin, equal merged rule
counts and every reference rule matched within 0.1 bp.
"""
import importlib.util
import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
REPO = os.path.abspath(os.path.join(HERE, "..", "..", "..", ".."))
BASE = os.path.join(REPO, "crates", "compiler", "tests", "tabular_corpus", "oracle.py")

spec = importlib.util.spec_from_file_location("tabular_oracle", BASE)
oracle = importlib.util.module_from_spec(spec)
spec.loader.exec_module(oracle)
oracle.FIXTURES = os.path.join(REPO, "crates", "render-pipeline", "fixtures", "boxes")
oracle.REFS = os.path.join(oracle.FIXTURES, "refs")

if __name__ == "__main__":
    sys.exit(oracle.main())
