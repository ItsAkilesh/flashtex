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
