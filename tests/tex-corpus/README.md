# Compiler compatibility and recovery corpus

Owner: FT-011 / commander-corpus. Created September 12, 2026. Contract:
[Runtime v1](../../../docs/contracts/runtime-v1.md).

These 14 original small projects define requirements for the original Rust
compiler. **No case currently claims verified compiler support.** Validation here
checks fixture and manifest integrity, not rendering or full LaTeX compatibility.
The complete product requirement remains larger than this finite corpus.

Run from repository root:

```sh
python3 tests/tex-corpus/validate.py
python3 tests/tex-corpus/validate.py --emit-request included-file > /tmp/flashtex-include-request.jsonl
```

The second command emits exactly one runtime-v1 JSON line, with all project source
files included. It can be sent to a compatible compiler process when available;
it does not invoke any installed TeX engine. No network, package installation,
external assets, or paid inference is needed. Python 3.9+ is sufficient.

## Manifest and expected behavior

`manifest.json` defines each case's entry file, complete source inventory, features,
compatibility profile, and expectations. Every expectation cites a source witness:
project-relative file, zero-based UTF-8 start byte, exclusive end byte, and the
exact original source text. These ranges are fixture evidence, not assertions that
an entire expanded macro must map one-to-one to its invocation. Literal navigation
cases require original file/range identity. Expansion provenance beyond that is an
open contract design decision; do not fabricate exact per-glyph provenance.

| Cases | Requirement |
|---|---|
| plain-paragraphs | Text order and paragraph boundary |
| macro-arguments, macro-scope | Argument substitution and local binding restoration |
| unicode-literals, literal-source-map | Unicode preservation and byte-accurate navigation |
| math-inline-display | Inline scripts and a real two-dimensional fraction |
| included-file, include-scope | Included source identity and caller execution scope |
| comments-escapes | TeX comment line removal and escaped literal characters |
| unknown-command, unclosed-group, extra-closing-group, missing-include | Explicit diagnostic plus usable provisional output |
| tikz-required | Required package/diagram support; unsupported must remain explicit |

Profiles separate basic LaTeX, Unicode-capable LaTeX, and TikZ package behavior.
Unicode cases require an encoding/font profile with the necessary glyph coverage;
this corpus supplies no font and does not assert identical output across engines.
TikZ is intentionally an outstanding compatibility case, not a request to hide a
reference compiler behind the product or substitute a static image.

`visible_text` specifies semantic text order; output may split literal runs into
items and normalize ordinary source whitespace. `math_layout` and `diagram_layout`
need layout/visual assertions, not string comparison. `source_navigation` checks
clickable source identity against the exact input revision. `diagnostic` plus
`recovered_text` specifies FlashTeX recovery behavior, not a promise that all
reference TeX engines recover identically. Diagnostic wording is not prescribed;
location, severity, recovery explanation, and remaining content matter.

## Evidence required before calling a feature supported

A future compiler runner must record the compiler commit/build command, input
case/revision, selected compatibility profile, result status, diagnostics, and the
actual output artifact or display list. It should validate every returned source
range against that revision and retain failed cases. Record observed support in a
separate result artifact, with pass/fail/unsupported/unverified per expectation;
do not edit the fixture to match a compiler defect or mark unsupported as passing.
For unsupported required constructs, an explicit diagnostic passes only the
honesty/recovery gate, not the feature's rendering gate.

For performance and incrementality, use paired edits and compare incremental
output with a clean compile of the identical edited sources; this static corpus
alone provides neither a latency measurement nor proof of cache correctness.
Full project completion additionally requires native builds, PDF/export checks,
real capture conversion, package compatibility, and end-to-end validation.
