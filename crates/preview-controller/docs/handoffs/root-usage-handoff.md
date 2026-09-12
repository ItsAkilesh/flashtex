# Root usage-exhaustion and engineering takeover packet

Prepared at 2026-09-12T17:11:22.360129+00:00. The user's latest request is frequent commits and detailed
recovery documentation before usage ends. No remaining-quota telemetry is available;
do not claim exhaustion has happened. This packet records a clean, terminal checkpoint.

## Authority and what takeover means

The current global Commander is `claude` on `mac-m5pro-kabir`, session
`session_01Xd5Hmwh5GHNTiAmHUJ1MZu`. Published claim:
`abbe88a5275b89d99357815846de3cbe76a91810`, descendant of the explicit outgoing
handoff `7f7d4186f3f6b98a770635b2ee158c8bd1d165f5`.
The old Astra `/root/runtime_validator` completed its transfer, stopped all three
Linux control services, and is quiesced. Do not restart it when root becomes unavailable.
Root exhaustion is an engineering vacancy, not global Commander failure.

Kabir's launchd monitor is deterministic and has no model calls. It monitors Git;
it cannot automatically revive a terminated Claude inference session. No guaranteed
uptime or remotely callable Linux hosted-agent service has been established.
Check current authority on main because these facts can change after this packet.

## Exact root state and recovery commands

Local worktree `/home/natkarri/flashtex-preview-performance`; branch
`agent/commander-preview-performance/preview-performance`.
Before this documentation commit local and remote HEAD matched
`3c071dc8f03b4fd0daec252f5da92ebcc335b693`. Latest product evidence is5b250b22.
The documentation commit containing this packet is the new branch tip; use Git,
not an invented self-referential hash, to identify it. Root owns preview-controller,
edit-ledger and its own coordination/agents/commander-preview-performance.json.
Do not mutate global authority, queues or other subsystem source files.

```sh
git fetch origin main agent/commander-preview-performance/preview-performance
git show origin/main:coordination/authority.json
git show origin/main:coordination/assignments/FT-048.json
git show origin/agent/commander-preview-performance/preview-performance:coordination/agents/commander-preview-performance.json
git status --short
git log -5 --oneline
```

On another computer, preserve existing work and create a separate worktree from the
published root branch. Ask the current Commander to fence the engineering ownership
before continuing changes; do not race an unverified original process. On this host,
inspect actual tool/process state first. No root build/probe or paid request is pending.
Do not rerun terminal session63185: it completed successfully.

## Completed work and exact evidence

| Deliverable | Revision / location | Proven result |
| --- | --- | --- |
| Current-base Text-only patch |8b676e72/8dd9170d; text-bc737-reconciliation | Applies to exactbc737126; three sources plus seven-test fixture; preserves existing symbols/headings/delimiters |
| Compiler combination validation |5b250b22; text-bc737-validation |111pass,3existingignored across13suites; strict Clippy;10complete protocol replies equal prior reference |
| Bounded protocol probe |e975f38f; ../../tools/bounded_protocol_probe.py |8child-lifecycle tests; deadlines include blocked writes/partial reads; request identity and EOF checks |
| Helper ownership audit |80713a9e | Metadata edits already avoid response source clones; remaining runtime Request copy has immutable-validation purpose |
| Proof alternative review |6fc8172e plus runtime9a749554 | Compact proof reduces retention but worsens preparation; no adoption |
| Producer integration review |f9c2edb5; text-producer-65ce-review.md | Feature/cache/hash/shift and recovery coverage limits made explicit |
| Text bounds defect |0f91b645; text-producer-bounds.md; GH43 | u16 aliasing and invalid handle boundaries identified; producer owner fixed at73217526 |

Paths in this table are relative to this handoffs directory unless explicit.
The probe lives under crates/preview-controller/tools, not an external install.
The fixed Text producer remains separate from compiler adoption. Its default feature
tests do not prove a newly vendored Text-enabled compiler compiles until the cache
Text arms and feature integration are included. Do not apply the historical full
producer adapter.patch over the new TextSink conversion; it contains an obsolete
before-build typeset hunk. Preserve distinct Text hash tag and source-span shifts.

Current Text patch base is `bc73712632345dea800267561e7fec7135cfe465`.
Older hw1-current-compiler cumulative patch is against1c02a2bc and is historical:
DO NOT stack it with the current-base Text-only patch or overwrite newer owner fixes.
Read the current patch README for actual existing test target names.

Local validation scratch: `/home/natkarri/flashtex-text-bc737-build-svnqscot`.
It contains the tested debug compiler at `crates/compiler/target/debug/flashtex-compiler`,
validation logs and protocol-replay output. Its binary/source hashes are published in
text-bc737-validation/evidence.json. A Mac must rebuild native binaries; local Linux
executables are not portable artifacts. Source, patch, logs and captured replies
needed for the current result are in Git; no scratch directory is required to adopt.

## Tests to repeat only after relevant changes

Apply the Text patch to an isolated checkout of the exact base after checking it.
On a newer compiler revision reconcile first. Then use the explicit test command in
text-bc737-reconciliation/README.md and the bounded probe instructions. Existing
text-bc737-validation logs are completed evidence; don't spend quota/build time
repeating unchanged tests merely because a new agent resumed.

The ten requests/expected replies are in hw1-text-session. Probe output directories
must be NEW; refusal of an existing path prevents stale success evidence. Retain
stderr and partial-response artifacts on failure. Probe cleanup covers its direct
child, not arbitrary descendant processes. Any compiler diagnostic change must be
reviewed semantically; never replace expected output only to turn tests green.

## Pending gates and proposed next work

1. Commander/compiler owner reviews and adopts the Text patch, then publishes an
   exact authoritative compiler revision. No adoption has been confirmed here.
2. Existing producer owner integrates Text hashing/shift arms and feature/vendor pin,
   preserving the slot-index fix73217526 and existing optimized JSON/protocol.
3. Runtime owner can then validate one existing HW1 plus two UTF8/Text edits against
   the exact adopted producer: complete fresh/persistent outputs and source-bound
   sibling/currentness checks. No duplicate broad benchmark sweep is needed.
4. Renderer proposed an immediately eligible bounded task: pass six existing Text
   owner after/display.json artifacts through current immutable CFF binder/PDF writer,
   check extracted spaces/ffi and source integrity with pinned resources. Await the
   Commander's dispatch; do not duplicate producer/paragraph/font ownership.
5. Root can review the resulting integration or take a fresh scoped preview task.

Producer GH43 fix review: comment5647336825 supports the corrected original bounds,
but new run_of lookup scans all runs per painted glyph. That is a static complexity
finding, not a measured regression. Existing owner should use many-distinct-run
acceptance before claiming responsiveness. Shaper expansion bounds and full
source-bound oversized diagnostic delivery remain qualified in the review.
Renderer4614248e found a122743tick superscript baseline change in owner before/after
artifacts despite unchanged wording. Pixel comparison and searchable-space checks
remain necessary. Last historical four-page native measurement296ms median/405ms
p95 is NOT sub200ms acceptance or a current measurement.

## Other retained workers and delivery

- Runtime handle `/root/compiler_corpus`, branch
  agent/commander-runtime-display/display-runtime, worktree
  /home/natkarri/flashtex-runtime-display, product9a749554/report e6e99ccc.
  Completed proof experiment, no jobs; no production API changes.
- Renderer handle `/root/supervisor_api_review`, branch
  agent/commander-render-core/rendering-core, product4614248e.
  Completed review, no jobs. No queued implementation authorization at checkpoint.

Within this live thread root can use followup_task to resume a completed retained
handle for an exact Commander assignment. send_message alone does not resume it.
An agent on another computer cannot assume access to these handles. It can instead
consume Git handoffs and have its own machine lead allocate a replacement worker
within current authorized capacity after ownership transfer. Do not restore drained
bridge/project-index workers or invent fresh quotas to compensate.

Remote machine status must be read anew. Historical Daniel reportcc5c05be confirmed
parent work, not16livechildren; FT039/043/044 were machine-owner halted, FT030 had a
local tool refusal. Do not bypass those holds. Jaysen target and any new Daniel
allocation are current Commander's responsibility, not root's authority.

## Billing, provenance and checkpoint discipline

Local Claude remains API-only with no verified funded allocation. Do not switch to
local subscription/OAuth/extra-usage routes, buy usage, or retry unresolved paid calls.
Root used hosted allocation only; remaining quota unknown. Other machines' scoped
plan authorizations are not transferable funding. Preserve actual reports and holds.

User permits direct Git after observed Cursor quota limits. Use truthful actual
implementation/executor trailers and local authenticated Git user as coauthor;
never pretend Cursor executed a direct commit. Do not force-push or rewrite peers.
Publish coherent completed changes every10–15minutes while doing sustained work,
and immediately when an interface, blocker or handoff changes another worker's next
step. Save dirty files, actual process handles, pending operations and evidence
before compaction or interruption. Check remote publication success, not only commit.

## Communication links and copyable revival instruction

- Commander status/proposal request: GH1#5647359032.
- Actual first task pickup: GH1#5647374314.
- Other lane proposals: GH1#5647378201.
- Completed validation/adoption request: GH1#5647387156.
- Local next-dispatch blocker: GH1#5647403918. No dispatch after completed work was
  visible at checkpoint; this is not evidence the remote Commander is offline.
- GH43: https://github.com/flash-tex/flashtex/issues/43.

> You are replacing the root ENGINEERING lane, not taking global command. Read
> AGENTS.md, current main authority, this packet, current assignment and own worker
> report. Preserve any uncommitted work. Reconcile exact pending process/publication
> state; all jobs listed here were terminal when recorded. Consume5b250b22 Text
> validation and current-base patch; do not repeat old patches or unchanged tests.
> Contact the current Commander with your actual session/capacity and one concrete
> next action, obtain/follow the fenced assignment, then publish implementation,
> evidence and a fresh checkpoint. Never infer takeover permission from silence.
