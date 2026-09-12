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
