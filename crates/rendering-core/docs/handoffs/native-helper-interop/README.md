# Native helper-display migration gates (FT023r11)

At this inspection, `agent/mac-helper-display/route` has no published remote ref.
The active remote session is ae7ddcfb739815fac; absence of a ref does not imply
that it is idle or its unpublished work has these bugs. Fallback review pins
current mac-shell1fd5ba0b and direct live consumer9dbdb01. File hashes and exact
scope are in inspection.json. No Swift/native authoritative files were changed.

## Concrete published font gap: issue31

GlyphRunRenderer.swift V2FontStore hashes discovery Data, retains URL/hash/length,
then resolve reopens the path for first CGFont creation without rechecking those
new bytes against the cached identity. It caches the result under the old hash.
Glyph count, UPEM and PostScript name do not prove immutable byte identity.
Verify raw hash/length of the SAME Data passed to CGDataProvider, or retain the
original verified discovery bytes. Cached verified CGFont can legitimately serve
old frames after disk replacement. A minimal mutation case is recorded; native
CoreGraphics behavior still needs a Mac test. This is not a claim that an actual
replacement font was painted during this Linux review.

The store also indexes historical SHA(bytes+face0). Current helper65dbe7d
requires raw-byte SHA. Keep any legacy file-profile compatibility explicit;
do not allow it to satisfy helper current-resource acceptance. The existing
Rust binder refuses that substituted digest.

## Migration acceptance mapping

| Concern | Published direct route | Required helper-route gate |
|---|---|---|
| Generations | worker request/project/compile revision | keep editor version2 distinct from compile3; also bind helper session/membership generation |
| Source | direct navigateV2 checks selected document hash | bind complete path/hash/length/text and editor source-version snapshot; no authority from event alone |
| Async completion | delivery checks load ticket; newest arrival queues behind in-flight load | recheck current helper session/project/membership/compile/source identities immediately before installing as current; a ticket alone is insufficient |
| Historical frame | old verified frame may remain explicitly stale | historical display is allowed only under that policy, never a current-navigation/export authority grant |
| Font | old and raw hash aliases; first-resolve reopening gap | actual immutable full bytes/rawSHA/face/UPEM/GID/PS identity; issue31 fix |
| Navigation | direct hash guard then navigateExactly | helper event source_actions_enabled=false remains false; validated geometry alone never upgrades edit/navigation authority |
| Failure/decline | experimental pane can show v1-only/refused/stale state | preserve required v1 preview/ACK; malformed, missing-resource, stale or declined optional candidate must not erase current v1 or silently substitute fonts |

`cases.json` uses the unchanged actual8876279 step1 event/source fixture and
JSON-pointer/current-snapshot mutations. It is test data, not a new protocol or
validator. The helper_candidate regression feeds all cases to the existing Rust
binder: correct editor2/compile3 accepts; stale editor/session/membership/compile,
changed bytes, engine-hash substitution and source-action promotion refuse.
The existing raw runtime tests separately cover declined/failed/legacy no-sibling
behavior. Native owner should consume these same expectations with the actual
Swift/controller queue, including delay-validation→edit/close/reopen→callback.

The direct consumer's coalescing may intentionally publish an older verified
frame while preparing a queued one; this review does not relabel that historical
policy as a confirmed current-route bug. New helper currentness must instead
match the stronger state above. Test retained stale view separately from current
paint, navigation and export. No native route is enabled by this handoff.

A minimal reviewable `font-byte-identity.patch` is included against exact
mac-shell1fd5ba0b. It verifies the newly read Data's length/raw digest before
constructing CGFont from that same Data. `git apply --check` passes in an isolated
copy of the exact source. Swift compilation and native mutation/callback tests
remain owner-required; this patch has not changed authoritative native code.

Linux verification after the timing-window release: all four helper_candidate
tests pass, including the eight migration cases; strict all-target Clippy passes.
These validate the Rust reference expectations, not unpublished native behavior.
