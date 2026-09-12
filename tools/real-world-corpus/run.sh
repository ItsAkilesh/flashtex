#!/bin/bash
# Convenience wrapper: runs run.py with this checkout's compiler, the pinned
# flashtex-render scratch build and the pdf routes. Every path can be
# overridden through the environment; missing binaries are reported, not built.
#
#   FLASHTEX_COMPILER   crates/compiler/target/release/flashtex-compiler
#   FLASHTEX_RENDER     tools/real-world-corpus/target/render-pipeline-<sha>/crates/render-pipeline/target/release/flashtex-render
#   FLASHTEX_PDF_EXACT  crates/pdf/target/release/flashtex-pdf-exact
#   FLASHTEX_PDF        crates/pdf/target/release/flashtex-pdf
#   FLASHTEX_RENDER_SHA short SHA the scratch render build came from (default 9aaec57a)
#
# To (re)build the pinned render worker from origin/agent/mac-render-pipeline/unified:
#   git archive <sha> crates/render-pipeline | tar -x -C tools/real-world-corpus/target/render-pipeline-<sha>
#   cargo build --release --manifest-path tools/real-world-corpus/target/render-pipeline-<sha>/crates/render-pipeline/Cargo.toml --bin flashtex-render
set -u
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
SHA="${FLASHTEX_RENDER_SHA:-9aaec57a}"
COMPILER="${FLASHTEX_COMPILER:-$ROOT/crates/compiler/target/release/flashtex-compiler}"
RENDER="${FLASHTEX_RENDER:-$ROOT/tools/real-world-corpus/target/render-pipeline-$SHA/crates/render-pipeline/target/release/flashtex-render}"
PDF_EXACT="${FLASHTEX_PDF_EXACT:-$ROOT/crates/pdf/target/release/flashtex-pdf-exact}"
PDF_V1="${FLASHTEX_PDF:-$ROOT/crates/pdf/target/release/flashtex-pdf}"
exec python3 "$ROOT/tools/real-world-corpus/run.py" \
  --compiler "$COMPILER" --compiler-note "crates/compiler of this checkout @ $(git -C "$ROOT" rev-parse --short HEAD)" \
  --render "$RENDER" --render-note "origin/agent/mac-render-pipeline/unified @ $SHA (git-archive scratch build)" \
  --pdf-exact "$PDF_EXACT" --pdf-v1 "$PDF_V1" "$@"
