# Opt-in helper display-list forwarding (FT-048r6 / FT-049r1)

Status: helper implementation checkpoint using runtime f58b656. Default OFF.
Runtime owner: FT-049. Helper owner: FT-048. Native acceptance remains separate.

The producer contract is runtime-v1 `display-list-v2`: an accepted, nonfailed
compile result is immediately followed by exactly one matching `display_list`
JSON line. A declined capability yields ordinary v1 output only. Unsupported
producers remain usable. The runtime validates request/project/revision and exact
source byte lengths and SHA256 bindings, while downstream rendering validation
must check fonts, resources, operations and features. A candidate is untrusted.

## Negotiation and lifetime

Helper command `configure_display_candidates` requires
`capability:"display-candidates-v1"`, `enabled:boolean`, and explicit
`renderer_support_confirmed:true` when enabling. Default is OFF. The client waits
for its result before expecting any candidate. Enabling must also enroll the
producer layout capability through the existing controller negotiation, preserving
other requested layout capabilities. Historical completed-snapshot retention and
display candidates are initially mutually exclusive; reject conflicting enable
requests without changing either policy.

Disable, restart, close, layout changes and source/membership changes invalidate
pending candidates. Enabling cannot resurrect work submitted before that enable
operation. Runtime acceptance of a sibling remains bounded by the existing active
request timeout: a promised but missing sibling must not stall forever.

## Delivery

Keep required v1 preview/ACK/error FIFO behavior. Before taking a runtime candidate,
check the existing optional output slot's admission condition. This allows the
runtime's single candidate slot to retain the candidate while the v1 fallback
frame finishes writing, without adding a second helper pending-value slot.

Move the candidate envelope into a helper `update` with a payload:

- `kind:"display_candidate"`, `untrusted:true`, `source_actions_enabled:false`;
- original `request_id`, `project_id`, `compile_revision`;
- exact controller `source_versions` and `membership_generation`; source hash
  bindings remain inside the original v2 envelope;
- `display_list`: the original v2 envelope, moved without a full-value clone.

The outer helper envelope retains its existing `session_id`. Recheck the current
controller generation and source snapshot before admission. Serialize under the
complete helper frame limit. An oversized optional candidate is dropped; required
replies continue intact. Admission races may also drop candidates. Do not transform
these drops into successful v2 delivery claims.

Both optional features share the existing one-frame output slot, with mutually
exclusive opt-ins initially. Required output clears any queued optional frame.
An optional write already started cannot be preempted; retain the existing stalled
writer watchdog. No hard end-to-end latency claim follows from queue boundedness.

## Native gate

The client accepts only its current helper session/request/source identity and
requested format. Validate the entire v2 display envelope and font assets off the
UI thread, then recheck source/session identity immediately before paint. Candidate
receipt alone never authorizes source navigation or establishes pixel parity.
Retain the v1 fallback when optional frames are absent, invalid or dropped.

Implementation gates: default/declined producer compatibility, exact sibling
correlation and timeout, stale/toggled-session refusal, source hash validation,
required-output priority, bounded oversized-frame handling, restart/close cleanup,
and a real native helper-route measurement. Existing direct-Mac producer results
do not prove this helper route works.

The configuration result includes `enabled` and a separate `preview_error` if the
policy was accepted but its fresh compile could not be submitted. A failed
configuration precondition does not change either optional mode. Disable removes
the layout capability and requests a fresh v1 compile. Restart requires a fresh
opt-in. Layout configuration cannot request display-list-v2 while the mode is OFF.

`take_current_display_payload` checks the runtime candidate against current
controller request ID, compile generation, submitted membership/version snapshot,
source hashes and byte lengths. The envelope's document revision is a compiler
generation per the sibling contract; the outer `source_versions` map carries
individual durable document revisions. They must not be equated.

Initial actual-helper fixture verifies matching v1-before-candidate delivery,
individual source revisions, explicit confirmation, mutually exclusive policies,
and restart reset. Its payload intentionally lacks renderer-valid resources: this
is transport evidence, not rendering evidence. Remaining native and malformed/
backpressure integration gates above must be completed before release activation.

Additional helper subprocess checks pause the fixture between its v1 result and
sibling using a filesystem release signal, persist a newer edit before release,
and verify only the newer source hash/version reaches optional output. A corrupt
sibling hash triggers preview failure without exposing a candidate; a subsequent
edit still persists and survives helper kill/reopen. These checks exercise the real
helper/runtime boundary with a transport fixture, not a real renderer.

Both optional routes use the same complete-frame serializer/admission boundary.
Its tests include compact JSON numbers that expand on reserialization: exceeding
the output budget publishes no partial optional bytes and leaves the following
required ACK intact. Exact newline-inclusive limits and obsolete epochs are also
checked. These are boundary tests, not a full-sized native output stress test.

With existing `diagnostic_timings:true`, stderr includes `phase:optional_output`,
`kind`, `outcome` (`admitted`, `busy_or_obsolete`, or `serialization_refused`) and
`serialization_ms`. These diagnostics contain no source text or payload data.
Admission is queue acceptance, not proof of native receipt or paint. As with other
phase diagnostics, clients must drain stderr while enabled.

Full-size helper failure-path tests now cover a roughly 5.6 MB numeric sibling that
fits the runtime's default input bound but expands beyond the real 16 MiB helper
output bound. The test waits for explicit `serialization_refused` diagnostics,
then verifies an intact edit ACK and exact durable reopen. Another test keeps
stdout open but unread after startup, admits a 1 MiB optional display frame, and
verifies the existing stalled-writer watchdog terminates the helper while source
remains intact. These use transport fixtures and do not establish native paint
performance or renderer validity.

Timing interpretation: `compiler_poll` diagnostics include every eventful owner
poll and eventless polls lasting at least 1 ms. Candidate-only processing or
stale-value destruction can occur without a v1 event; older eventful-only traces
must not be summed as complete owner CPU/wall time. Ordinary fast idle polls stay
unlogged. Poll duration includes scheduling and owner work, not just source hashing,
and does not include all background decoder CPU. Fine-grained candidate timing is
owned by the runtime and remains separate from native renderer timing.

With `diagnostic_timings:true`, helper stderr now emits `phase:display_transport`
and the runtime's source-bound scalar `profile`, once per request/revision/display
epoch. No repeated idle-poll measurements are emitted. A cleared runtime profile
resets deduplication. Profile fields are request/project/revision/epoch, framed byte
count, JSON parse time, decoder queue wait, owner delivery wait and source-binding
validation time. They contain no document text or renderer resources. They remain
transport measurements; native font/render validation and paint are not included.

The actual oversized-frame helper test verifies exactly one logged profile after
idle and document-query polls, framed bytes above5MB, and finite nonnegative stage
durations before confirming that durable editing still works. Use these stages
alongside optional-output serialization and end-to-end receipt timings; do not
infer a full latency budget by summing phases from different requests or processes.
