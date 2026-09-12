# mac-preview-cache handoff — preview text cache invalidation, retention, measurement

- Updated UTC: 2026-09-12T08:55Z
- Agent / parent / machine alias: `mac-preview-cache` (Claude Code subagent) /
  parent `mac-claude-a` / `mac-m1max-a`
- Task / acceptance gate / owned paths: parent-dispatched follow-ups to the
  FT-003 lane "Remove repeated text/font work from main-thread paint with
  immutable identity cache" (parent commits 6c468c8, 552a67a, 6b43a3a; GitHub
  issue #26): (1) resource/source generation invalidation and bounded
  retention, (2) cold/warm paint workload measurement without hiding latency.
  Serves acceptance gate 5 (typing-to-visible preview latency) and the exact
  geometry requirement. Owned paths:
  `apps/mac/Sources/FlashTeXMac/PreviewTextCache.swift`,
  `apps/mac/Sources/FlashTeXMac/Fonts.swift`,
  `apps/mac/Tests/FlashTeXMacTests/PreviewTextCacheTests.swift`,
  `apps/mac/Tests/FlashTeXMacTests/PreviewFontsTests.swift`,
  `coordination/mac-preview-cache.md`, `coordination/agents/mac-preview-cache.json`.
  No Commander assignment JSON exists for this lane; it is a child of the
  parent's FT-003 allocation (issue #2 dispatch thread).
- Branch / code revision / main integrated through:
  `agent/mac-preview-cache/invalidation` (from
  `origin/agent/mac-claude-a/mac-shell` 6b43a3a) / see `git log` / origin/main
  c1baad1 merged (main adds nothing under `apps/mac`).
- State: ready for integration (by the parent into `agent/mac-claude-a/mac-shell`).
- Ready behavior and evidence:
  - **Explicit resource generation** (`Fonts.swift`):
    `PreviewFonts.resourceGeneration: UInt64` moves when an input of face
    resolution, or of what a PostScript name denotes, changes: Latin Modern
    registration on first use (`FLASHTEX_LM_DIR`, app bundle `Fonts/`,
    repository `apps/mac/Fonts`, TeX Live roots — the directory used is
    recorded in `latinModernDirectory`), a `producerFace` change
    (`flashtex-compiler` → Times, `flashtex-render` → Latin Modern; set by
    `ShellModel.attachWorker`, unchanged), an explicit
    `overrideEnvironmentFace(_:)` replacing the once-read
    `FLASHTEX_PREVIEW_FACE`, or `invalidateResources()` for a caller that
    registered/unregistered fonts itself. Setting the same value is not a
    change. All accesses are main-thread (draw closure, shell, CoreGraphics
    export); the Rust export path never touches `PreviewFonts`.
  - **Keyed invalidation** (`PreviewTextCache.swift`): `Key` now carries
    `generation` (stamped by `key(text:postScriptName:size:)`, source-compatible
    default). Both stores (lines and fonts) retire wholesale the first time a
    lookup observes a moved generation (`syncGeneration()`, one integer compare
    per call, counted in `generationRetirements`). A stale key handed in is
    served the current generation's entry and is never inserted. There is no
    ad hoc `clear()` in the shell; `clear()` remains for tests/bench.
  - **Bounded retention**: `LRUCache<Key, Value>` (dictionary + doubly linked
    list, O(1) lookup/insert/evict, links unlinked on removal and in `deinit`)
    replaces drop-all. Lines (`capacity`, default 20 000) and fonts
    (`fontCapacity`, default 512) are bounded independently; evicting a font
    never drops a line (a `CTLine` retains its font). Lowering a bound evicts
    immediately. Counters: `hits`, `misses`, `lineEvictions`, `fontEvictions`,
    `fontsBuilt`, `generationRetirements`.
  - **Exact geometry unchanged**: keys use the exact `Double` bit pattern;
    `CTFontGetSize == requested` (10.004, 9.999999, one-ulp neighbours are
    distinct entries); cached metrics are asserted bit-identical
    (`accuracy: 0`) to an uncached construction for every demo.tex item at 1×
    and 2×.
  - **Entry points unchanged**: `line(for:)`, `rect(for:scale:)`,
    `key(text:postScriptName:size:)`, `font(_:size:)`, `PreviewHitRects`.
    `PreviewView.swift`, `ShellModel*.swift`, `ContentView.swift`,
    `FlashTeXMacApp.swift` untouched; no diff needed in parent-retained files.
  - **Measurement** (`PreviewTextCache.measure(pages:scale:capacity:fontCapacity:)`
    → `Workload`): for every text item of a compile result it times three
    passes with the identical construction (`PreviewTextCache.build`:
    attributed string + `CTLineCreateWithAttributedString` +
    `CTLineGetTypographicBounds`, plus `CTFontCreateWithName` where a font is
    not cached): *uncached* (every item from scratch, no cache), *cold* (fresh
    cache, one pass), *warm* (second pass). Counts are asserted in tests;
    timings are printed, never asserted against a tolerance. This measures line
    construction only — the per-item `CTLineDraw` and SwiftUI `Canvas` cost
    remain measured by `TypingBench` (paint hook), not hidden here.
  - **Numbers, `apps/mac/Samples/demo-result.json` (compiled demo.tex, 3 pages,
    968 text items, 520 distinct (text, face, size) keys, 3 fonts), Times face,
    Apple M1 Max, release build (`swift test -c release`), three runs, single
    pass each (no repetitions averaged):**

    | pass | scale 1× | scale 2× |
    |---|---|---|
    | uncached (font + line per item) | 10.4–14.2 ms (10.7–14.7 µs/item; first pass of the process includes CoreText's own warm-up) | 6.4–6.7 ms (6.7–6.9 µs/item) |
    | cold (fresh cache; 520 misses / 448 hits) | 2.7–3.2 ms (2.8–3.3 µs/item) | 2.65–2.74 ms (2.7–2.8 µs/item) |
    | warm (968 hits / 0 misses) | 0.20–0.22 ms (0.21–0.22 µs/item) | 0.19–0.21 ms (0.20–0.22 µs/item) |

    Debug build (`swift test`, the full-suite run): cold 3.9–6.9 ms, warm
    0.63–0.78 ms at both scales. Under a deliberately undersized bound
    (`capacity = 260` = half the distinct keys, `fontCapacity = 2`) the warm
    pass thrashes as expected: 453 hits / 515 misses, 2.8–2.9 ms release,
    `lineEvictions > 0`, count exactly 260 — the bound holds and geometry stays
    exact. For reference the parent measured the pre-cache SwiftUI `Text` path
    at ≈0.3 ms per item (≈300 ms per keystroke for demo.tex); the cached warm
    path costs ≈0.2 µs per item for lookup, so the remaining per-keystroke
    paint cost is the draw and Canvas work, to be read from `TypingBench`.
- Incomplete behavior / blockers / needs from others:
  - `FLASHTEX_PREVIEW_FACE` is still read once at process start; a runtime
    change of the environment cannot be observed by any process. The explicit
    `overrideEnvironmentFace(_:)` is the invalidation-aware replacement if the
    shell ever grows a face preference.
  - The generation retires every entry conservatively (Times lines are dropped
    when Latin Modern registers, although Core-14 names do not change meaning).
    Cost: one cold rebuild (≈3 ms for demo.tex) per resource change; resource
    changes happen at worker attach, not per keystroke.
  - No app-level (TypingBench) re-measurement was run in this lane; the parent
    owns `PreviewView`/`TypingBench` evidence.
- Interface changes / consumer actions: additive only. New:
  `PreviewFonts.resourceGeneration`, `invalidateResources()`,
  `latinModernDirectory`, `environmentFace` (read), `overrideEnvironmentFace(_:)`;
  `PreviewTextCache.Key.generation`, `init(capacity:fontCapacity:)`,
  `syncGeneration()`, `build(text:font:)`, `contains(_:)`,
  `lineKeysMostRecentFirst`, `fontKeysMostRecentFirst`, counters,
  `Workload`, `measure(...)`; `LRUCache`. `capacity`/`fontCapacity` are now
  computed properties over the LRU bounds (same get/set usage).
- Reviewed peer revisions / resulting adaptations: `origin/agent/mac-claude-a/mac-shell`
  6b43a3a (base; read PreviewView draw path and ShellModel.attachWorker to keep
  the entry points and the producer-face setter unchanged); `origin/main`
  c1baad1 (coordination/tests only, nothing under `apps/mac`; merged clean).
- Validation commands / results / artifact paths:
  - `cd apps/mac && swift build` → Build complete.
  - Full suite with real binaries from the main checkout
    (`FLASHTEX_COMPILER=…/crates/compiler/target/release/flashtex-compiler`,
    `FLASHTEX_PDF`, `FLASHTEX_BRIDGE`, `FLASHTEX_EDIT_LEDGER` likewise):
    `swift test` → **195 tests, 0 failures** (baseline on 6b43a3a before this
    lane: 185 tests, 0 failures). New tests: 8 in `PreviewTextCacheTests`
    (LRU order/bound, lowering capacity, font-independent eviction,
    generation stamping/retirement, producer-face switch never serves the old
    face, real `CTFontManagerUnregisterFontURLs`/re-register of the LM masters
    proving the same name denotes a different font across the generation,
    demo.tex cold/warm exactness, undersized bound), 2 in
    `PreviewFontsTests` (generation moves only on real changes; registration
    records its directory and moved the generation).
  - `swift test -c release --filter PreviewTextCacheTests/testDemoWorkload` ×3
    → numbers above (stdout lines prefixed `preview-text-cache`).
- Exact deadline UTC / remaining time / integration reserve: no automatic
  deadline (PROJECT.md); reserve not applicable to this bounded lane.
- ETA remaining, optimistic / likely / pessimistic / confidence: 0 / 0 / 0
  for this lane; parent review + merge pending.
- Resource pool / allocation ID / maximum: shared Claude Max 20x quota on
  mac-m1max-a via parent mac-claude-a / `claude-mac20x-preview-cache` /
  unknown (shared, not a separate balance).
- Confirmed spend / estimated usage / in-flight reservation / remaining:
  unknown / one subagent session / none / unknown.
- Billing evidence / freshness / unknowns: none available to a subagent.
- Child tasks and their deducted allocations: none.
- Dirty files / unpushed work / running jobs: none after the final push.
- Decisions / failed approaches / linked findings: generation kept in the
  key *and* as a store tag (structural guarantee plus bounded residency);
  true LRU chosen over a two-slab generational cache to avoid confusing it
  with the resource generation; the block-based
  `CTFontManagerUnregisterFontURLs` was needed in the test (the pointer
  variant is deprecated and ambiguous with `nil`).
- Exact next action or command: parent reviews and merges
  `agent/mac-preview-cache/invalidation` into `agent/mac-claude-a/mac-shell`,
  then re-runs `tools/typing-bench/run.sh` for paint-level evidence.
- Resume reading list: this file, `apps/mac/Sources/FlashTeXMac/PreviewTextCache.swift`
  header comment, GitHub issue #26, parent handoff `coordination/mac-claude-a.md`.
