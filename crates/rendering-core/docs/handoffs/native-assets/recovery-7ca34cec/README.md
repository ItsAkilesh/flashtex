# Existing recovery/export boundary, unchanged policy

Four original7ca discovery cases were consumed using existing pipeline_cff_probe
binary41d9cc1f, without rebuilding or rerunning the producer. Original request,
stdout and display hashes are retained in report.json; source document hashes,
lengths, project/request IDs and compile revisions agree. Diagnostic code/message/
severity and source spans agree across v1's `source` and v2's `sources` array;
the two diagnostic JSON shapes are not claimed byte-identical.

Actual standalone export observations:

| Original case | Diagnostic | Existing export result |
| --- | --- | --- |
| missing bundled10pt | tfm_missing, warning | PDF produced |
| corrupt explicit10pt | tfm_missing, warning | PDF produced |
| corrupt flat12pt, missing rooted12pt | required_metrics_unavailable, error | refused, no PDF |
| missing rooted12pt, no override | required_metrics_unavailable, error | refused, no PDF |

Both warning fallback PDFs have SHAde995170db6c28679df64a49b9cf801f2fa9aff60e9836f3ff0548ce9ca320bd.
They are explicitly recovered OpenType-metric output, not verified TeX-metric
fidelity. The error cases return `error diagnostics prevent searchable export`.
This is the existing standalone policy; no stricter rule was introduced.

The paired-helper boundary is a separate source-inspected contract. Existing
`pipeline_frame::pair(..., require_tex_metrics=true)` refuses tfm_missing,
required_metrics_unavailable, or any error with `reference metrics unavailable`.
`helper_candidate::bind` already selects that mode, so neither warning fallback
nor blocking metric failure can reach that helper's searchable export. With the
flag false, pairing remains transport validation and grants no resource readiness.
These paired-rule conclusions are read-only assessments of the exact unchanged
Rust source pinned in report.json, not execution of a newly built paired CLI on
these inputs. Only the standalone exporter was invoked on the four new cases.

Framing matters:10pt requested and received the v2 capability and has two actual
stdout frames. The unchanged12pt request never negotiated that capability and has
one stdout result; its `.v2.json` is an explicit `--v2` side file. Passing that file
as a negotiated sibling in permissive pairing would fail acceptance mismatch;
`None` is the correct transport shape. Do not fabricate capabilities or a helper
envelope to turn debug side output into an actual negotiated delivery.

The original recovered inputs remain unchanged, diagnostics remain visible, and
no native helper route, signed app, reference parity or new export policy is claimed.
