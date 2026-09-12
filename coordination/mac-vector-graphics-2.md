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

## Checkpoint

- branch: `agent/mac-vector-graphics-2/json-depth` from
  `origin/agent/mac-claude-a/mac-shell` 9ba9851c (consumed main: see parent tip)
- dirty files: (see git status at each refresh)
- next commands: `cd crates/vector-graphics && cargo test && cargo test --release && cargo clippy --all-targets -- -D warnings && cargo fmt --check`
- decisions: typed `JsonError` enum (`Message(String)` + `NestingTooDeep { depth, limit }`),
  raw-JSON limit 256, group limit 64 (see rationale in json.rs docs once landed)
