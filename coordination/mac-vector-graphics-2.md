# mac-vector-graphics-2 — GH46 JSON nesting-depth bound (crates/vector-graphics)

Lane: Claude Code subagent of `mac-claude-a` on mac-m1max-a. Scope override from
parent: this lane MAY edit `crates/vector-graphics` (unowned crate, GH46) and
nothing else under `crates/`.

## Coverage audit (2026-09-12, ≤10 min)

Searched `crates/vector-graphics/{src,tests,docs,README.md}`,
`apps/mac/Tests/FlashTeXMacTests/*`, `docs/evidence/*`,
`tools/native-validation/mac-live/reports/20260912T110944Z.md`,
`docs/contracts/runtime-v1-display-list-v2.md` for `depth`, `nest`,
`NestingTooDeep`, `MAX_JSON_DEPTH`, `MAX_GROUP_DEPTH`, `#46`.

Already covered (none of it addresses recursion depth):

- `crates/vector-graphics/src/json.rs::tests::parses_strings_and_numbers` — flat
  parse, `[1,]` and trailing-garbage rejection only.
- `crates/vector-graphics/src/json.rs::tests::number_formatting_round_trips`.
- `crates/vector-graphics/tests/primitives.rs::json_round_trip_is_exact` — one
  group level.
- `crates/vector-graphics/tests/primitives.rs::json_reader_rejects_bad_input` —
  format/version/kind errors, no nesting.
- `crates/vector-graphics/tests/primitives.rs::nested_group_clips_intersect_in_device_space`
  — two-level group clip semantics, not depth.
- No Mac shell test, evidence file or mac-live report references this crate.
- The display-list contract states no nesting bound (only byte-size limits at
  lines 126–129), and the crate README states none.

Conclusion: the gap (GH46) is entirely uncovered; implementing.

Prior art: Daniel (`d-q222`) reports a local, unpushed branch
`agent/daniel-parent/vg-ds-fix` (e4b8b877) with `MAX_JSON_DEPTH = 1000` /
`MAX_GROUP_DEPTH = 64` and the measurement that ~175 nested groups already
overflow a debug `cargo test` worker thread at 200. Not on origin; not consumed.
This lane implements independently, re-measures, and credits that finding.

## Result (2026-09-12T18:36Z)

Fix commit: `f5b370fe1e4a4005fe078e178c4dd34fe1a52bde` on
`agent/mac-vector-graphics-2/json-depth` (base `origin/agent/mac-claude-a/mac-shell`
9ba9851c). Files: `crates/vector-graphics/src/json.rs`,
`crates/vector-graphics/src/display_list.rs`, `crates/vector-graphics/README.md`,
`crates/vector-graphics/tests/json_depth.rs` (new). No parent-retained file and
nothing else under `crates/` touched; no diffs for the parent to apply.

Pre-fix reproduction at 9ba9851c (temporary test, release): 100 000 nested arrays
-> `thread has overflowed its stack`, SIGABRT (exit 134).

Design:
- `json::MAX_JSON_DEPTH = 256` on `Parser` (`enter`/`leave` around
  `array_body`/`object_body`), `display_list::MAX_GROUP_DEPTH = 64` on
  `read_items`/`read_item` (group depth threaded through). Both return
  `JsonError::NestingTooDeep { depth, limit }` before recursing further.
- `JsonError` is now `enum { Message(String), NestingTooDeep { depth, limit } }`;
  every existing message and `Display` string is unchanged. `JsonError(..)`
  tuple construction was only used inside the crate.
- `DisplayList::validate` gains `ValidationError::NestingTooDeep { id, depth, limit }`
  and does not descend into an over-limit group, so validate is bounded.
  `flatten`, `bounds`/`hit` (via flatten), `pdf::content_stream`, and the JSON
  writers still recurse over in-process trees; they are bounded by the same
  limit for any tree that came from `read_display_list` or passed `validate()`
  (documented in json.rs module docs and the README). Not converted to
  iteration: those walkers take no external input and the change would be out
  of proportion for this bounded lane.
- Limit rationale: contract and README state no bound. Measured on
  mac-m1max-a with the constants temporarily lifted, debug `cargo test`
  worker thread: groups 190 ok / 200 overflow; raw arrays 1200 ok / 1500
  overflow. Release: groups 1000 ok / 5000 overflow. 64 groups is ~3x under
  the tightest measurement; 256 raw levels is ~4.7x under, and the deepest
  legal document is 2*64+2 = 130 levels. d-q222's independent 175/200
  measurement on GH46 is credited on the constant.

Tests (`crates/vector-graphics/tests/json_depth.rs`, 12):
`limits_are_documented_and_consistent`, `nested_arrays_at_limit_parse`,
`nested_arrays_over_limit_return_typed_error`, `nested_objects_at_limit_and_over`,
`arrays_and_objects_share_one_depth_counter`, `depth_counter_resets_between_siblings`,
`nested_groups_at_limit_read_and_validate`, `nested_groups_over_limit_return_typed_error`,
`validate_reports_in_memory_trees_over_the_group_limit`,
`hundred_thousand_deep_arrays_return_error_in_bounded_time`,
`hundred_thousand_deep_groups_return_error_in_bounded_time`,
`unterminated_deep_input_is_rejected_by_depth_not_end_of_input`.

Verification at f5b370fe: `cargo test` and `cargo test --release` 14 unit +
12 json_depth + 26 primitives = 52 pass; `cargo clippy --all-targets -- -D warnings`
clean; `cargo fmt --check` clean; `cargo doc --no-deps` no warnings.
Performance: no benchmark/fixture exists in the crate; a scratch harness
(128 KB, 200-group document, 300 parses, release, interleaved before/after
from a `git archive 9ba9851c` copy) gave medians 1.427/1.400/1.398 ms before vs
1.401/1.437/1.398 ms after — unchanged within noise; p90 dominated by machine
load (load average 40-120 during the runs).

Limitations: measurements are from one machine/toolchain (Rust stable on
macOS arm64, Xcode 26.3 host); the non-reader walkers are bounded by contract
rather than by an internal counter; no Swift-side tests (the crate is not wired
into the Mac shell).

## Checkpoint

- branch: `agent/mac-vector-graphics-2/json-depth` tip f5b370fe + this
  coordination commit; base 9ba9851c (consumed main through the parent tip)
- dirty files: none after this commit
- next commands: parent posts SHA on #46 and merges; nothing pending here
- ownership: crates/vector-graphics only (parent override for this lane)
- billing: shared Claude Max quota via parent mac-claude-a; no purchases
