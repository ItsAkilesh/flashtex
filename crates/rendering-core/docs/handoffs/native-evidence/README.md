# Published native evidence: scope and remaining gates

This read-only review pins the actual source/report hashes in `review.json`.
No native binaries or tests were executed by the reviewer.

## What the230-gate report establishes

The validation5a13599 report ran2026-09-12 11:09:44–11:19:14Z. It built app7ccbd7e9,
helpers1884986f, and direct render producer6e696616. Merging that report into newer
mac-shell5bc3fc0/e50f0e60 does not rerun tests on those newer sources.

The report explicitly separates direct compiler, direct render, and durable
controller routes. The controller benchmark and multi-document helper check use
the main `flashtex-compiler`, not the newly configured raw display-list helper
route. `\input{chapter}` durability on that route therefore does not establish
native raw-candidate binding, currentness, or CFF paint acceptance. The new actual
Linux capture test remains useful interoperability input, but is not a Mac run.

The completed-snapshot60KB30ms row shows first-paint p95=258ms, but current-paint
p95=2900ms and max3188ms (107 historical paints,19 current). Baseline current-paint
p95=206ms. Historical rows are explicitly not latency-gated by the230PASS verdict.
This is a demonstrated limitation of that opt-in snapshot policy under those old
binaries and recorded load, not a measurement of current optimized main. Keep it
OFF pending a new comparable current-source native run. One run/cell, programmatic
insertText, CoreAnimation commit rather than scan-out, and uncontrolled concurrent
machine load further limit conclusions. Even the baseline206ms row does not prove
the requested sub200ms target in that case.

## GH31 published fix review

Owner branch `agent/mac-helper-display/route` now publishes a9b55af7. Its resolve
path reads the file once, checks length and rawSHA against the immutable discovery
record, then creates CGDataProvider/CGFont from the same Data. Only verified font
bytes enter the cache; cached CGFont identity remains immutable. This addresses
the demonstrated changed-file-before-first-resolve gap without replacing retained
frames. Both prior hash spellings remain discovery lookup aliases; the raw identity
is checked before loading. This source review does not broaden legacy alias policy.

The new owner tests append a zero byte while keeping parsed font metadata stable,
expect refusal under both spellings, restore original bytes to resolve, and retain
the same previously verified CGFont object after disk changes. The commit reports
that the refusal test fails before the fix and passes afterward. This reviewer
inspected those tests but did not execute them. The fix is not an ancestor of the
reviewed e50f0e60 shell. Integration plus the owner's native test output/binary
provenance remain the next acceptance gate; this commit alone is not publication
of the planned configure_display_candidates route.

## GH36 remains a distinct asset gate

No `origin/agent/mac-packaging-tfm/*` artifact was published in the inspected refs.
A session identifier/assignment is not a packaged result. The older validation
bundle JSON inventories binaries/components, not TFM/font resource coverage, and
its report does not establish a no-host-TeX environment. Thus it cannot close the
verified packaging gap. Use the exact pinned manifest/verifier and candidate
patches in `../native-assets`; require applied owner SHAs, exact packaged resource
hashes, bundled producer discovery with explicit overrides preserved, and actual
app-only/nohostTeX direct+helper results. Preserve GH34's separately corrected Linux
fixture scope.

## Chronology and owner actions

Issue2 comment5646190139 has GitHub created/updated13:30:18Z but says observed13:40Z
and lists starts after13:30. The shell handoff similarly labels a later observed
time. These discrepancies prevent treating the stated census as exact live-process
evidence; they do not invalidate separately pinned Git artifacts or the older
report's explicit run times. Commander requested a corrected census.

Existing owners should (1) integrate and publish GH31 native test provenance,
(2) publish the actual helper-display route and run the current raw multi-source
fixture through native guards, and (3) apply GH36 assets/discovery with the no-TeX
bundle gate. No new parallel owner, speculative patch, or duplicated validation
runner is needed.
