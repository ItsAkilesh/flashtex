# Astra integration checkpoint, September 12

This is integration evidence, not a completion or parity certificate. The local
team remains one Commander plus three product engineers; three other agents are
paused by the user. Remote task reports do not prove remote process liveness.

## Published checkpoints

- `cab39a2`: runtime transcript negotiation and same-revision request-ID stale
  suppression. 50 focused tests (3 optional skips), 210 repository Python tests
  (10 optional skips) using the declared jsonschema test environment.
- `1b17778`: editor helper/search/history, review inbox, exact CFF/TrueType outline
  consumers and font resources. Combined Rust suites: rendering84, fonts74,
  jobs39, index65, ledger48, document-runtime13 and controller21 pass. The three
  ignored original-compiler runtime/controller gates were separately run and pass.
  Font oracle probes remain optional; these counts do not claim native rendering.
- `10e14f6`: rooted file export (`d92db37` project-files, `0032100` controller).
  Linux project-files47 and controller26 tests pass, including actual helper
  export/conflict/restart and original compiler. GH18's reproduced symlink-parent
  escape is fixed. Noncooperative filesystem writers and post-rename uncertainty
  remain explicit limitations; no unconditional filesystem CAS guarantee.
- `a6d6478`: compiler `9026d8a` negotiated rule/font output and recovery. 83 tests
  pass, 2 optional ignored; strict lint passes. A real compiler request accepting
  rules-v1/font-hints-v1 passes the transcript validator.
- `cfdda8b`: dispatcher retries only read-only shared remote-ref compare-and-swap
  fetch races, at most three times. 24 tests pass. Authentication, publication,
  arbitrary locks and uncertain paid calls retain their hard stops.

## Corpus remains incomplete

Pinned `9026d8a1b9ffdcfa4d01f3cdb4d83c5e6096849f` against prior compiler
`3ae7d9b76e6503fcb60f33cb2258b78250baf8bc` loses no passing checks. It still has
4 passing, 7 unverified, 2 unsupported and 1 failing case (`comments-escapes`).
The gate records source/output checks independently and does not interpret an
`ok` compiler status as full TeX compatibility or reference PDF equality.

## Held candidates

Mac shell `3e26ca6` has remote native/packaging evidence, but is held under GH19:
PDF export performs blocking pipe I/O and wait-before-drain on the main actor;
opening another file replaces dirty source without a preservation/decision gate.
The Linux source checker has zero known bridge-shape failures but cannot prove
native behavior. Existing Mac workers own the fixes; staffing has not increased.

A later Rust candidate containing controller reload `61d29a0`, font cache
`63a9371`, mixed renderer `b18285c` and PDF `4bd8c2e` passes fonts79, renderer88
and Linux PDF41 tests. Its controller suite fails one stalled-output backpressure
case under concurrent builds (25 pass, 1 fail, 2 ignored). The exact failed case
passes in isolation; the owner is investigating. This candidate is not promoted
on the strength of an isolated retry. Real compiler-to-negotiated-PDF `--verify`
succeeds, which proves transport/export acceptance only, not reference parity.

## Measurement boundaries

Raw PDF byte equality, zero-pixel raster equality and typing-to-visible latency
are separate acceptance gates. Compiler/helper timings exclude native paint.
Nonzero font/line/rule deltas remain exact-parity failures. Completion forecasts
must use pinned comparisons and observed mismatch closure rather than speculative
calendar estimates. Improvement continues until explicit user stop.

## Subsequent backpressure resolution and promotion

Main `4de2c9e` promotes the later Rust batch after replacing the controller
candidate with `3e8861e`. The owner found a concrete separate hole: a single
blocked large reply never filled the output queue, so queue saturation alone
could not terminate it. A writer deadline and focused single-reply regression
fix that hole. The full rebuilt controller suite passes31 tests including the
real compiler, and atomic project membership passes66 index tests. The original
concurrent-load failure is not retroactively attributed with certainty. Mac
GH19 remains an independent held integration.
