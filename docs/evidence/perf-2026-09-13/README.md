# FT-065 compiler hyperoptimization: evidence, 2026-09-13

Agent `kabir-claude` (Claude Opus 5 subagent), machine `mac-m5pro-kabir`
(Apple M5 Pro, 15 logical CPUs, 24 GB). Branch
`agent/kabir-claude/compiler-perf`, base `origin/main` `f6645607`.

The Mac app launches `flashtex-render` (crates/render-pipeline) by default
(`6d2f5628`), so everything below measures that worker's real request path,
`protocol::handle_line`: request JSON parse, parse, adapt, typeset, page
build, display-list assembly, v1 fallback and reply JSON. UI paint and IPC
are not included; mac-claude-a measures those with `tools/typing-bench`.

Other agents were compiling on this machine throughout. Every row records the
1/5/15-minute load averages before and after it. The **same-run** tables below
ran the base and final builds back to back at load ≈3.4–3.8.

## Commits

| SHA | What |
| --- | --- |
| `59074193` | Daniel's compile-latency work (`9ec29873` + `24f285af`) squashed onto main, stray session trailer dropped. crates/compiler tests: all suites ok, 0 failed. |
| `f1edf45d` | adapter: macro definitions indexed once per request (the main fix); vendor/compiler re-pinned to `59074193`; `examples/perf_bench.rs`. |
| `8e723a6e` | display: `DisplayList::write_json` writes the display-list-v2 envelope directly instead of building a `Value` tree. |
| `a0a599f6` | compiler json: `write_integer` pushes the digits directly instead of validating them with `str::from_utf8`. |
| `e75940d7` | vendor/compiler re-pinned to `a0a599f6`. |

## Harness

`crates/render-pipeline/examples/perf_bench.rs` covers HW1 (`fixtures/real-world/hw1`), HW2 (from
`agent/kabir-claude/hw2-math-final`, passed with `--hw2`) and the 500 KB
scaling document. That document uses the same deterministic generator as
`crates/compiler/src/bin/scaling_bench.rs`; `gen500.py` here writes the
identical bytes.

Scenarios:

- **full**: a fresh `RenderCache` per request, with fonts already loaded.
- **type-paragraph**, **type-inline-math**, **type-display-eq**: one more
  character per step at a fixed place, against a warm cache.
- **delete-restore-line**: alternately delete and restore one line.

Each scenario runs with the Mac's default capabilities (`rules-v1`,
`font-hints-v1`) and again as `+v2`, which adds `display-list-v2`.

Every reply line is hashed with SHA-256, and each scenario gets one digest over
all its replies. `--digests` writes these digests and `--check` compares
against a saved set. `--verify-fresh` compares each warm reply with a cacheless
render of the same text.

```sh
cd crates/render-pipeline
cargo build --release --example perf_bench --example stages
git show origin/agent/kabir-claude/hw2-math-final:fixtures/real-world/hw2/HW2.tex > /tmp/HW2.tex
./target/release/examples/perf_bench --steps 40 --only HW --verify-fresh --hw2 /tmp/HW2.tex --digests hw.txt
./target/release/examples/perf_bench --steps 6 --only 500KB --digests 500.txt   # digests depend on --steps
./target/release/examples/stages <file.tex> 5                                    # stage split
```

## Byte-identical output

The base build (`f6645607`) produced the digests in `raw/digests-hw-base.txt`
(HW, 40 steps) and `raw/digests-500-base.txt` (500 KB, 6 steps).

- **All 30 scenario digests** from the final build match base: 20 for HW1/HW2
  and 10 for 500 KB, each with and without `+v2`. The check was repeated after
  every commit.
- **`--verify-fresh`** (every warm reply equals a cacheless render) passed on
  base and after every commit.
- **Unit test** `display::tests::write_json_matches_the_value_tree` covers
  string escapes, all provenance kinds, rules, fractional paint, negative and
  2^40 ticks, and an empty list.
- **Test suites**: crates/render-pipeline `cargo test --release` gives 84
  passed, 0 failed (one test was already ignored before these changes). The
  crates/compiler suites, including `json_fast_paths_match_the_formatting_machinery`
  and `incremental_json_identity`, pass.

## Results (same run, p50 / p95 ms)

HW1 and HW2, 40 steps (`raw/final-hw-base.txt` vs `raw/final-hw-new.txt`, load 3.6):

| scenario | base f6645607 | final e75940d7 | speed-up |
| --- | ---: | ---: | ---: |
| HW1 full | 2.15 / 2.63 | 1.45 / 1.67 | 1.5x |
| HW1 type-paragraph | 1.61 / 1.75 | 0.91 / 0.98 | 1.8x |
| HW1 type-inline-math | 1.60 / 1.71 | 0.86 / 0.97 | 1.9x |
| HW1 type-display-eq | 1.59 / 1.69 | 0.87 / 0.95 | 1.8x |
| HW1 delete-restore-line | 1.55 / 1.64 | 0.81 / 0.89 | 1.9x |
| HW2 full | 2.27 / 2.46 | 1.55 / 1.70 | 1.5x |
| HW2 type-paragraph | 1.71 / 1.84 | 1.03 / 1.15 | 1.7x |
| HW1 full +v2 | 6.04 / 6.28 | 1.98 / 2.11 | 3.1x |
| HW1 type-paragraph +v2 | 5.68 / 5.88 | 1.50 / 1.62 | 3.8x |
| HW1 delete-restore-line +v2 | 5.53 / 5.96 | 1.42 / 1.52 | 3.9x |
| HW2 full +v2 | 6.08 / 6.39 | 2.13 / 2.23 | 2.9x |
| HW2 type-paragraph +v2 | 5.67 / 5.93 | 1.61 / 1.73 | 3.5x |

500 KB, 6 steps (5 for full; `raw/final-500-*.txt`, load 3.4):

| scenario | base f6645607 | final e75940d7 | speed-up |
| --- | ---: | ---: | ---: |
| 500KB type-paragraph | 6135 / 6155 | 97.7 / 107.8 | 63x |
| 500KB type-paragraph +v2 | 6206 / 6241 | 98.9 / 101.2 | 63x |
| 500KB type-inline-math | 5436 (earlier run, load 9) | 97.0 / 98.3 | ~56x |
| 500KB full | 5803 (earlier run, load 8) | 159.7 / 175.8 | ~36x |
| 500KB full +v2 | 4633 (earlier run, load 6) | 153.5 / 154.8 | ~30x |

The earlier base runs are in `raw/bench-500-base.txt`. A full base 500 KB pass
takes about 10 minutes, so only type-paragraph was repeated in the same run.

Cold first request, fonts not loaded:

- **In-process** (`FontSet` construction plus the first HW1 request with
  `display-list-v2`): 16.5–23.6 ms on the final build, against 17.0–31.9 ms on
  base under varying load. Font loading dominates; the stage split below shows
  typeset at 22–29 ms on the first request and 0.2 ms after.
- **Whole process** (`flashtex-render` answering one HW1 request): about 0.01 s
  wall on both builds. `/usr/bin/time` only resolves to 10 ms.

Warm stage split, 500 KB one-word edit (`examples/stages`, ms):

| stage | base | final |
| --- | ---: | ---: |
| parse | 13 | 14 |
| adapt | 4 390 | 20 |
| typeset (shape, break, pages) | 26 | 30 |
| assemble v2 | 22 | 26 |
| v1 | 3.5 | 4 |
| json (20 MB) | 8 | 8.5 |

## Profiles and flamegraphs

Captures come from macOS `sample` (1 ms interval) on a running `perf_bench`,
folded with `inferno-collapse-sample` and drawn with `inferno-flamegraph`.
`hotspots.py <folded>` prints the inclusive-time tables below. The base HW1
`+v2` capture had the bench's own reply hashing, which is outside the timed
region, removed from the folded stacks. Later captures ran with `--no-hash`.

| file | what |
| --- | --- |
| `flamegraphs/fg-500k-keystroke-base.svg` | 500 KB warm keystroke, base |
| `flamegraphs/fg-500k-keystroke-after1.svg` | the same after `f1edf45d` |
| `flamegraphs/fg-500k-keystroke-after2.svg` | the same after `8e723a6e` |
| `flamegraphs/fg-hw1-keystroke-v2-base.svg` | HW1 warm keystroke, `+display-list-v2`, base |
| `flamegraphs/fg-hw1-keystroke-v2-after2.svg` | the same after `8e723a6e` |
| `folded/*.txt` | folded stacks for each capture |

Hotspots, base, 500 KB keystroke (5073 samples):

| inclusive % | self % | frame |
| ---: | ---: | --- |
| 98.6 | 0.0 | `adapter::adapt_cached` |
| 98.3 | 95.9 | `adapter::macro_body` |
| 49.2 | 0.0 | `adapter::items_from_inlines` |
| 2.1 | 2.1 | `memcmp` |

Hotspots, base, HW1 keystroke `+v2` (2016 samples): `display::to_json`
building the `Value` tree takes 25%, dropping that tree 26%, and
`json::Value::set` (BTreeMap inserts with owned keys) 22%. `render_cached`,
i.e. layout, is only 34%.

Hotspots, final (`8e723a6e`), 500 KB keystroke (2825 samples): the cost is now
spread out.

| inclusive % | self % | frame |
| ---: | ---: | --- |
| 94.1 | 0.0 | `protocol::handle_line` |
| 22.9 | 0.1 | `typeset::assemble` |
| 21.4 | 3.6 | `Clone::clone` (cached `GlyphRun`/`BoxRec`/`Line` copies) |
| 18.7 | 0.1 | `adapter::adapt_cached` |
| 17.4 | 1.8 | `compiler::parser::parse_project` (whole-document lex and parse) |
| 16.4 | 0.1 | `typeset::build` |
| 14.2 | 8.1 | `free` |
| 13.1 | 6.3 | `incremental::place_item` |
| 7.3 | 7.3 | `memmove` |

Hotspots, final, HW1 keystroke `+v2` (4122 samples): `typeset::build` 33%
(re-typesetting the edited paragraph: `hlist`, `text_box`, `layout_paragraph`),
`assemble` 19%, `adapt_cached` 18%, `adapter::find_command` 9% (the per-request
preamble scans), `display::write_json` 9%.

## Optimizations landed

1. **Macro-definition index** (`f1edf45d`, `adapter.rs`).
   - *Cause*: `macro_body` scanned the whole source for four definition commands
     on every call. `items_cached` calls it through `is_invocation_span` for every
     inline of every block, including warm requests, so the cost was
     O(source × inlines).
   - *Fix*: the identical scan now runs once per document per `adapt_cached`
     call. A scope guard registers the request's `texts`, and the index is keyed
     by their address and length while they are borrowed, so no key can outlive
     its bytes. The pick rule is unchanged, and callers outside the scope still
     scan.
   - *Effect*: 500 KB edits 63x faster, HW1/HW2 edits about 1.8x.
2. **Direct display-list-v2 writer** (`8e723a6e`, `display.rs`, `protocol.rs`,
   `flashtex-render --v2`). Keys are emitted in the `BTreeMap` order the tree
   used, through the same json writers. HW1 `+v2` edits are 3.8x faster.
3. **Daniel's compile-latency squash** (`59074193`), re-vendored into the
   pipeline (`f1edf45d`). It brings the json integer, hundredths and unescaped
   string fast paths, and in-place block shifting in the compiler's own
   `Session`.
4. **`write_integer` without `str::from_utf8`** (`a0a599f6`, re-pinned in
   `e75940d7`). Interleaved A/B, HW1 full `+v2`, p50 of 200 requests:
   1.94/1.91/1.98 ms before, 1.89/1.83/1.86 ms after.

Tried and reverted: building `place_item`'s output in one pass instead of
clone-then-mutate. An interleaved A/B on the 500 KB edit showed no gain beyond
noise (89.5/94.2/91.2 ms vs 89.5/93.7/94.1 ms).

## Targets not met

| target | status |
| --- | --- |
| HW1 full render < 20 ms | Met with fonts loaded (1.45 ms). Cold first request is 16.5–23.6 ms, dominated by font loading; see font-metric caches below. |
| warm single keystroke on 500 KB < 5 ms | **Not met: 98 ms** (base 6.1 s). |

The 500 KB request still does work proportional to the document on every
keystroke:

- the whole document is re-lexed and re-parsed;
- every cached block's records are cloned and relocated;
- all 408 pages of display items are re-placed;
- a 20 MB `compile_result` is serialized and would be sent to the Mac.

JSON serialization alone is about 8.5 ms at that size. Getting under 5 ms needs
the deferred architectural changes below.

## Deferred (conflict with in-flight branches, or architectural)

- **Incremental reparse** reusing unchanged blocks by byte range (about 15 ms
  of the 98 ms). Touches `compiler::lexer`/`parser`, which amsmath-envs,
  hw2-math-final and tikz-min change. Also needed: an AST arena for the
  compiler parser, same files.
- **Shared cached pages**: `Rc`-shared placed lines and `GlyphRun`s instead of
  clone plus relocate (`Clone` 21%, `place_item` 13%, `assemble` 23%). Changes
  `typeset.rs`/`incremental.rs`/`display.rs` data types, which hw2-math-final
  and graphics-floats (new item kinds) are editing.
- **Delta replies** (display-list-v2-delta). The Mac side has an isolated
  proposal (`RenderingV2Fast.swift`,
  `crates/render-pipeline/docs/proposals/display-list-v2-delta.md`). It needs
  a protocol contract change and Mac cooperation. Without it no 500 KB
  keystroke can go below the 20 MB serialization plus IPC cost.
- **Per-request preamble scans**: `class_options`, `package_options`,
  `parindent`, `parskip`, `counter` and friends each call `find_command` over
  the whole entry source (9% of an HW1 `+v2` edit). They could be memoized like
  the macro index, but amsmath-envs and hyphenation edit this adapter region.
- **Parallel page assembly**: display items use `Rc<str>` and the cache uses
  `RefCell`, so threads need `Arc`/`Sync` changes across display, fonts and
  incremental.
- **Faster cache hasher** (SipHash is 5–6% of a 500 KB edit): cache keys are
  bare `u64` values with no equality check, so a weaker hash would risk wrong
  output. This needs key verification first.
- **Font metric caches** for the cold first request (typeset 22–29 ms on
  request 1, 0.2 ms after): not investigated further here. The cost sits in
  font loading and `typeset.rs`, a conflict area.

## typing-bench on mac-m1max-a (for mac-claude-a)

`tools/typing-bench/run.sh` uses `FLASHTEX_RENDER` when set. Build base and
final in separate worktrees so the default `crates/render-pipeline/target`
binary cannot be picked up by mistake:

```sh
git fetch origin
git worktree add /tmp/ft065-base f6645607
git worktree add /tmp/ft065-final origin/agent/kabir-claude/compiler-perf
cargo build --release --manifest-path /tmp/ft065-base/crates/render-pipeline/Cargo.toml --bin flashtex-render
cargo build --release --manifest-path /tmp/ft065-final/crates/render-pipeline/Cargo.toml --bin flashtex-render
FLASHTEX_RENDER=/tmp/ft065-base/crates/render-pipeline/target/release/flashtex-render \
  tools/typing-bench/run.sh --producers render --out docs/evidence/typing-bench-ft065-base.md
FLASHTEX_RENDER=/tmp/ft065-final/crates/render-pipeline/target/release/flashtex-render \
  tools/typing-bench/run.sh --producers render --out docs/evidence/typing-bench-ft065-final.md
# worker-side numbers on the M1 Max, with the byte-identity check against this report's digests:
git -C /tmp/ft065-final show origin/agent/kabir-claude/hw2-math-final:fixtures/real-world/hw2/HW2.tex > /tmp/HW2.tex
(cd /tmp/ft065-final/crates/render-pipeline && cargo run --release --example perf_bench -- \
  --steps 40 --only HW --verify-fresh --hw2 /tmp/HW2.tex \
  --check /tmp/ft065-final/docs/evidence/perf-2026-09-13/raw/digests-hw-base.txt)
```

The digests depend on the installed Latin Modern fonts and TFMs, so a
different TeX Live may give different digests on both builds alike. In that
case compare base and final on the same machine with `--digests` then
`--check`. The typing-bench seeds (`demo`, `body60k`, `fixture`) are small, so
expect the largest typing-bench change on `body60k`.

## Coordination notes

`python3 scripts/coord.py ack` was not run: `kabir-claude` has no registration
on main, and per instructions this lane did not create one.
