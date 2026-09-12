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
