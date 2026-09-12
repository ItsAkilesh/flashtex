# Opt-in source-bound display candidates

`Session::set_display_candidates_enabled(true)` allows explicit request capability
`display-list-v2`; the default is false. It is mutually exclusive with historical
completed-snapshot retention. Ordinary requests and declined capabilities retain
legacy behavior. A validated nonfailed result accepting the capability emits its
usual v1 Preview first, then holds the active wire request for exactly one contiguous
v2 display_list sibling. Failed results require no sibling. The original dispatch
timeout covers both lines; missing/interleaved/malformed/duplicate/unsolicited
siblings fail the session. No next request dispatch occurs before the sibling.

`take_current_display_candidate()` moves the one retained
`UntrustedDisplayCandidate`. Getters expose original request/project/revision,
source bindings and read-only envelope; `into_envelope()` moves the Value without
cloning it. This is ONLY transport correlation: native/core validation must still
validate complete rendering structure, features, geometry, font assets and raw
font-byte hashes before rendering. Unknown rendering content is intentionally not
interpreted here. There is no render-ready or source-navigation authority.

Each source path occurs exactly once and matches the request's complete document
set, raw UTF8 SHA256, byte length and compile revision. The latter is NOT an
individual editor-document version. Helpers must retain their original controller
source-version snapshot and bind it by path/hash/length; they must also validate
their own session, membership and display generation immediately before delivery.
The Mac contract's historical SHA(bytes plus face) paragraph is obsolete; corrected
producer65dbe7d uses raw font bytes. This runtime does not validate those font hashes.

Submit, close, failure and display-policy changes clear retained candidates.
Policy changes increment an epoch captured at submission, so toggling cannot
resurrect an older result. Superseded/cancelled siblings are correlated and drained
without retention. Currentness is per project; an outer helper must enforce its
selected-project/display generation. Restart requires a new Session. A caller
changing layout or source membership without submitting must invalidate via the
policy setter (including setting the current boolean again).

Memory/lifecycle: existing four-raw-frame queue, one decoding/validation permit
and joined decoder shutdown are unchanged. Each line retains max_frame bounds.
One retained candidate Value can coexist with the decoder's active Value and v1
preview events; this is not a global one-Value or RSS bound. Downstream optional
output admission should be checked before taking the candidate and must preserve
required ACK/current-preview priority. No extra full candidate clone is required.

Tests use synthetic Python subprocesses, including real UTF8 hashes, malformed
and missing siblings, stale edits, close/toggle epochs and sequential dispatch.
They do not establish native acceptance, optical/font correctness, parity or
end-to-end typing latency. Helper contract reviewed at c400d0a. Existing decoder
permit/shutdown tests remain authoritative for its queue invariant.

## Actual producer transport replay

`tools/replay_display_producer.py` verifies every published producer archive file,
rejects extra source inputs, rebuilds offline, and records exact binary/asset and
request/reply hashes. It downloads or installs nothing. Run with explicit existing
producer tree, font directory, rooted TFM directory and a new output directory.
The ignored `real_display_producer` test is its runtime consumer probe.

Evidence in `benchmarks/display-producer-65dbe7d` uses the unchanged355-file
archive and pinned official LM assets. Requested output returns one accepted
source-bound sibling with no diagnostics; legacy output returns only v1; a1500-byte
producer reply budget declines v2 with a recovered warning; a1-byte budget produces
an explicit failed v1 result and no sibling. The tiny budget is a producer failure
trigger, not a claim that its failure reply itself fits one byte. All four direct
producer results equal the runtime's returned JSON values; emitted font raw-byte
hashes occur in the recorded assets. Source text is shared via the committed request
fixture. No metadata correction, font substitution or rendered-output validation
occurs. This is actual transport evidence, not native visual or timing evidence.

The helper may delay taking the runtime slot while required output is pending.
Runtime tests retain it across polls, then move it once, or clear it on close. A
failed mutually exclusive policy enable leaves the prior policy usable. Root's
unchanged dependency uptake854b041 and forwarding contractc400d0a were reviewed;
actual helper/native output admission remains their separate integration gate.

## Delayed consumer and cancellation replay

`real_display_producer` also exercises the unchanged actual producer with an
injected60ms downstream pause (not a measured native validation cost). It cancels
revision1 before polling, retains/moves revision2 exactly once, toggles policy
around revision3 while preserving its v1 fallback, and submits revisions4/5 before
consuming the next candidate. The returned revision5 has its exact edited source
hash. Revision4 may be stale in flight or coalesced before dispatch depending on
whether the previous sibling is still draining; evidence records the actual path,
rather than asserting a scheduler-dependent branch. Both must suppress its preview.

`benchmarks/display-lifecycle-65dbe7d` preserves actual candidates, outcomes and
provenance referencing the matching four-mode replay. No global retained-byte/RSS
or native latency claim follows from one runtime candidate slot. An already moved
revision2 candidate remains immutable and cannot be revoked by the runtime. The
helper/native consumer must reject it against current source/session epochs before
painting; helper8876279 checks the controller snapshot before optional admission,
while native post-validation epoch checking remains a separate requirement.

Integration6c2061c's initial parallel test run failed all seven synthetic cases at
short60–160ms observation windows; the same suite passed serially. Checkpointc47139ff
replaces positive wall-time guesses with bounded observable-outcome waits and uses
explicit /usr/bin/python3 for fake workers. Production timeouts and decoder permits
are unchanged. The real replay similarly waits for previews/candidates rather than
assuming a particular compile duration. Test failures are retained by the runner
before it returns an error. No production boundary defect was established by these
test scheduling failures.
