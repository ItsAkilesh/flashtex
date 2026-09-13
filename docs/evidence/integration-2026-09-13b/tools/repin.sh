#!/bin/sh
# Stage 2: re-pin vendored siblings under crates/render-pipeline/vendor/ to the
# current HEAD, the way crates/render-pipeline/vendor/VENDORING.md prescribes
# (a `git archive` export plus a PIN file holding the source SHA).
#
#   sh repin.sh compiler math-layout paragraph-layout microtype \
#               tex-expansion tex-text-encoding tex-boxes
#
# On the kabir-claude integration lane these re-pins are deliberately NOT
# committed; run this after a fresh checkout to rebuild the working tree state
# that render-pipeline is gated against.
set -eu
W=${FT_WORKTREE:-$(git rev-parse --show-toplevel)}
V=$W/crates/render-pipeline/vendor
SHA=$(git -C "$W" rev-parse HEAD)
for name in "$@"; do
  d=$V/$name
  rm -rf "$d"
  mkdir -p "$d"
  git -C "$W" archive "$SHA:crates/$name" | tar -x -C "$d"
  printf '%s\n' "$SHA" > "$d/PIN"
  echo "re-pinned $name -> $SHA"
done
