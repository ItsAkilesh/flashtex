#!/usr/bin/env bash
# Builds the pinned `flashtex-assistant-context` helper that ProposalPreview
# consumes (review/approve interface published on
# origin/agent/commander/assistant-context, c8a3e1b interface / c93bf0d fixture)
# from a scratch `git archive` under apps/mac/build, the way
# tools/typing-bench/run.sh builds flashtex-render. Nothing in the checkout is
# modified. The tests find the result through
# `ProposalPreview.ExplanationConfiguration.locateHelper()`; `swift test` skips
# the real-helper cases when it is absent. The archive also carries
# conversion-jobs and bridge: cargo resolves the helper's optional (unbuilt)
# `grok` path dependencies even when that feature is off.
#
# Usage: Tests/FlashTeXMacTests/Fixtures/build_assistant_context.sh [<git ref>]
# Env:   FLASHTEX_ASSISTANT_CONTEXT_REF  default c93bf0d7fd6c44bc656fdcb905e9a102e6cfad40
set -euo pipefail
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
MAC="$(cd "$SCRIPT_DIR/../../.." && pwd)"
ROOT="$(cd "$MAC/../.." && pwd)"
REF="${1:-${FLASHTEX_ASSISTANT_CONTEXT_REF:-c93bf0d7fd6c44bc656fdcb905e9a102e6cfad40}}"
SCRATCH="$MAC/build/assistant-context"
if ! git -C "$ROOT" cat-file -e "$REF^{commit}" 2>/dev/null; then
  echo "build_assistant_context.sh: $REF not found; run: git fetch origin agent/commander/assistant-context" >&2
  exit 1
fi
SHA="$(git -C "$ROOT" rev-parse "$REF")"
rm -rf "$SCRATCH"
mkdir -p "$SCRATCH"
git -C "$ROOT" archive "$SHA" crates/assistant-context crates/edit-ledger crates/project-files crates/conversion-jobs crates/bridge | tar -x -C "$SCRATCH"
echo "$SHA" > "$SCRATCH/SOURCE_SHA"
cargo build --release --offline --manifest-path "$SCRATCH/crates/assistant-context/Cargo.toml" --bin flashtex-assistant-context 2>&1 | tail -1
BIN="$SCRATCH/crates/assistant-context/target/release/flashtex-assistant-context"
echo "built $BIN from $SHA"
shasum -a 256 "$BIN"
