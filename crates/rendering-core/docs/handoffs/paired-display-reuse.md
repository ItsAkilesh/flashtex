# Consume paired display once — FT023 revision14

The helper previously parsed the same sibling through `pipeline_frame::pair`,
then immediately reparsed it through `PipelineCff::bind`. Attribution identified
both pairing and resource binding as substantial remaining work after the Syntax
preflight change. This increment removes only that second sibling parse.

Private `PairedDisplay` owns the parsed envelope and borrows the exact immutable
original bytes for the duration of one binding call. Its constructor is the
existing pairing validator; there is no unchecked constructor, mutable accessor,
public deserialization, global cache or cross-request entry. `bind_paired` consumes
it and runs the shared profile/source/font-resource validation. Successful binding
retains the original bytes for the existing PDF owner. Refused input gains no extra
upfront whole-frame copy. The public standalone bind and pair APIs are unchanged.

The bound helper still retains its verified immutable resource snapshot. Every
export checks the caller's current session/request/source/membership snapshot as
before. No new registry generation or permission is inferred. Changing current
source text or membership invalidates export, including a newer A→B→A epoch; this
optimization cannot reinstate source-navigation authority or activate native/raw
transport.

An explicit legacy pair-then-bind comparison checks exact PDF equality and equal
refusals for absent resources, wrong font digest, invalid original GID and changed
source hash. The existing raw/escaped/duplicate and actual three-state fixtures
continue to use the strict helper path. Full tests and strict lint are required.

`tools/profile_helper_binding.py --base COMMIT --case 2 --pair-reparse --report PATH`
measures the existing largest capture in an isolated source copy. It alternates the
legacy reparse and consumed-token branch within each process, reverses order on
successive samples, and separates timing from allocation-counter passes. Legacy
first-envelope cleanup is included in its resource phase. The mode exists only in
the generated measurement copy; there is no production environment switch.
Any allocation/timing claim requires its completed report. No native or visual
parity claim follows from this measurement.

## Paired observation

On the existing 1,150,821-byte capture, 20 measured samples per branch/mode
(plus 3 warmups) show resource-binding allocation calls 46,332→898 and requested
capacity 12,081,545→1,313,397 bytes. Every other phase's allocation medians are
unchanged. Instrumented resource-phase medians are 21.00→2.04 ms, with redundant
first-envelope cleanup included in the legacy branch. These measurements do not
claim RSS reduction or whole-helper/native paint latency. Full samples and exact
source/tool hashes: `tools/evidence/paired-binding-attribution.json`; bounded
comparison: `tools/evidence/paired-allocation-comparison.json` (crate-relative).

173 Rust tests (2 explicitly ignored) and strict all-target Clippy pass before
measurement. The existing three actual raw-helper PDFs retain their pinned hashes.

The source/dependency guard was deliberately rebased to 21ce1756 and all five
established producer/reference fixtures replayed. PDF hashes, raster hashes,
extracted-text results and text-comparison validity exactly match the prior Syntax
checkpoint. Existing reference pixel differences remain 0/602/1244/979/337, including
the explicit display-math ToUnicode oracle limitation. Expected exit 3 records
those prior reference gaps; it is not a new parse-reuse regression. Evidence:
`tools/evidence/paired-five-fixtures.json` relative to crate root.
