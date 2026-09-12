# Bridge context checkpoint

Owner: /root/bridge_context, linux-primary, hosted Astra. Assigned directly by Root;
Sol confirmed ownership of context.rs, context tests and minimal lib.rs wiring.
Branch: agent/bridge-context/project-context; base b5ca96b.

Implemented: complete balanced supported macro/package declaration excerpts from
same-project uploaded snapshots; literal input/include connected context includes
parent preamble. Provenance gives path, snapshot revision and source line. Comments,
escaped commands and common verbatim environments do not create declarations or
include edges. UTF-8 source budget remains 16 KiB, selection 8 KiB. Oversized bodies
are omitted whole, with explicit incomplete-context notice. Scan bounded to 128
files, 1 MiB total, 256 KiB per file, 256 declarations/inputs per scanned file.

Wire Context fields unchanged. Mac should upload relevant project snapshots through
existing document_open before conversion, keeping revisions current. The bridge
never reads the filesystem to follow includes. Parent revision changes are visible
in declaration provenance even if destination document revision is unchanged.

This is lexical evidence, not evaluated scope: dynamic paths, conditionals,
catcodes, expansion, package loading and missing snapshots are not evaluated.
Definition ordering does not claim macro execution order. Current root-relative
literal .tex resolution prefers an uploaded project-root path, then the includer's
directory; parent traversal is rejected. Nearby transport and actual compiler
proposal validation remain separate integration work.

Validation: cargo test --manifest-path crates/bridge/Cargo.toml: 25 passed
(13 bridge, 3 CLI, 9 new context). cargo fmt applied. Corpus worker preparing
independent real-candidate fixture evaluation. No provider calls; hosted usage
unknown; no API/subscription grants consumed by an auxiliary CLI. Cursor commit
publication uses user's standing authorized route; cost unknown.

Next: Sol integrates reviewed candidate into main, publishes consumer contract
update from this note; corpus-owned fixtures exercise public context::build API.
ETA for local next acceptance: 5–15 minutes, depending independent fixture findings.
Resume: inspect git status and origin candidate; run cargo test with absolute
/home/natkarri/.cargo/bin/cargo; coordinate Sol before changing shared lib.rs.

## Follow-up: revision-checked edit request

Root assigned Bridge::edit clippy repair after context publication 7d4e0af.
Replaced seven scalar arguments with public serializable EditRequest; JSON Lines
wire schema unchanged. CLI decodes that same struct, internal receipt confirmation
and Rust tests use it. No mutation/revision/anchor behavior changed.
Acceptance: cargo test (25 pass), cargo clippy --all-targets -- -D warnings (pass),
cargo fmt --check (pass). Corpus separately verified 9 real-candidate context
checks at 7d4e0af, linked in GitHub issue #12 comment5643704321.

Next investigation: proposal compiler validation requires compiling a temporary
whole-project snapshot containing the proposed anchored edit, with exact revision
and result correlation; compare baseline/new diagnostics and surface unsupported
constructs, never equate recovered output with successful validation. Current
compiler main API is protocol::handle_line over runtime-v1 JSON; no external TeX
engine needed. Implementation ownership must be allocated before cross-module work.

## Follow-up: persisted dependency freshness

Root assigned dependency fingerprints and stale preparation protection. Context
has additive serde-default dependencies entries (path/revision/full source hash),
computed for exactly the uploaded literal include-connected component. Proposal
journal persists them; preparation verifies the current set. Legacy fingerprints,
missing dependency snapshots and changed content/revisions return explicit
proposal_context_stale. Explicit conversion refreshes an unprepared stale proposal;
unchanged retries use cache, unrelated files do not invalidate. Issued edits never
trigger a replacement provider call; reconcile receipt before a new capture.

Tests cover root-only macro edit/reconversion/new review, unchanged retry, unrelated
source, restart with missing dependency, reused revision but altered source hash,
legacy context and prepared-edit reconciliation. No live provider request made.
Final acceptance for fingerprint increment: 31 tests pass (19 bridge, 3 CLI,
9 context), clippy --all-targets -- -D warnings passes, fmt --check passes.
Failed refresh is explicitly tested to retain old stale context and block prepare.
