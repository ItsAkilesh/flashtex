# Source validation reuse: no production change justified

Read current runtime lib.rs submit_internal/encode/validate_reply_value and
raw_display::Parsed::validate/display_candidate::validate after completed proof
experiment9a749554. No new workload, production edit or timing sweep.

The accepted v1 path never hashes full source. Admission validates paths, aggregate
source bytes and framing while serializing borrowed Strings. Its temporary path
map is discarded. v1 builds a borrowed path-to-text BTreeMap and performs exact
UTF8 range checks. A promised sibling uses either raw or Value validation, never
both: each hashes each declared document once after exact membership lookup.
Candidate SourceBinding retains the checked hash, so candidate take does not
rehash. Dispatch metadata-budget calculation uses counts/path lengths, not source
hashes. There is no repeated same-request full-source digest to cache on the valid
v1/v2 pair path.

The v1 and sibling maps repeat construction. Retaining a borrowed map inside the
owning Pending would require self-references; an owned path/index map adds storage
and lifecycle complexity. With one sibling per request, amortization is limited.
Lazily building the v1 map helps empty/source-free failure responses but introduces
a branch at every sourced-item lookup. No measured normal-path benefit justifies
that tradeoff. Eager SHA/proof preprocessing is already rejected by the previous
experiment and is not repeated.

One defensive asymmetry remains: Value sibling validation computes the digest
before checking declared revision/length, whereas raw validation short-circuits
those checks first. Reordering could avoid hashing malformed responses but does
not improve valid compilation; existing refusal outcome and immutable ownership
are correct. It is not promoted as a responsiveness fix. No concrete lazy/reuse
optimization warrants a runtime API or retained-state change at this checkpoint.
