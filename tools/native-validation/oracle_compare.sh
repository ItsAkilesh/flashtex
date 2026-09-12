#!/usr/bin/env bash
# Reference-oracle comparison: pdflatex (a real TeX, used ONLY as a measuring
# stick, never by the product) versus the FlashTeX compiler + flashtex-pdf.
#
# Builds the compiler(s) and the PDF crate from git refs in temporary detached
# worktrees, compiles oracle_extract.swift (PDFKit word boxes), runs
# oracle_compare.py over tools/native-validation/oracle-samples/, and leaves
# reports/oracle-<UTC>.md + .json.
#
# Usage:
#   oracle_compare.sh [--repo <path>] [--pdflatex /Library/TeX/texbin/pdflatex]
#                     [--compiler-ref <label>=<ref> ...]   (default: main=origin/main)
#                     [--pdf-ref <ref>]                     (default: origin/agent/mac-pdf/pdf-output)
#                     [--scratch <dir>] [--reports-dir <dir>] [--keep] [--only <sample> ...]
set -euo pipefail

HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO="$(git -C "$HERE" rev-parse --show-toplevel)"
PDFLATEX="/Library/TeX/texbin/pdflatex"
PDF_REF="origin/agent/mac-pdf/pdf-output"
SCRATCH="${TMPDIR:-/tmp}/flashtex-validation"
REPORTS_DIR=""
KEEP=0
COMPILER_REFS=()
ONLY=()
while [[ $# -gt 0 ]]; do
  case "$1" in
    --repo) REPO="$(cd "$2" && pwd)"; shift 2 ;;
    --pdflatex) PDFLATEX="$2"; shift 2 ;;
    --compiler-ref) COMPILER_REFS+=("$2"); shift 2 ;;
    --pdf-ref) PDF_REF="$2"; shift 2 ;;
    --scratch) SCRATCH="$2"; shift 2 ;;
    --reports-dir) REPORTS_DIR="$2"; shift 2 ;;
    --keep) KEEP=1; shift ;;
    --only) ONLY+=(--only "$2"); shift 2 ;;
    -h|--help) sed -n '2,15p' "$0"; exit 0 ;;
    *) echo "unknown argument: $1" >&2; exit 2 ;;
  esac
done
[[ ${#COMPILER_REFS[@]} -gt 0 ]] || COMPILER_REFS=("main=origin/main")
[[ -n "$REPORTS_DIR" ]] || REPORTS_DIR="$REPO/tools/native-validation/reports"
[[ -x "$PDFLATEX" ]] || { echo "pdflatex not found at $PDFLATEX" >&2; exit 2; }

STAMP="$(date -u +%Y%m%dT%H%M%SZ)"
RUN_DIR="$SCRATCH/oracle-$STAMP"
mkdir -p "$RUN_DIR" "$REPORTS_DIR"
WORKTREES=()
cleanup() {
  [[ $KEEP -eq 1 ]] && { echo "keeping $RUN_DIR"; return; }
  for wt in "${WORKTREES[@]:-}"; do
    [[ -n "$wt" && -d "$wt" ]] && { git -C "$REPO" worktree remove --force "$wt" >/dev/null 2>&1 || rm -rf "$wt"; }
  done
  git -C "$REPO" worktree prune >/dev/null 2>&1 || true
}
trap cleanup EXIT

echo "pdflatex: $("$PDFLATEX" --version | head -1)"
COMPILER_ARGS=()
LABELS=()
for spec in "${COMPILER_REFS[@]}"; do
  label="${spec%%=*}"; ref="${spec#*=}"
  sha="$(git -C "$REPO" rev-parse --verify "$ref^{commit}")"
  wt="$RUN_DIR/compiler-$label"
  git -C "$REPO" worktree add --detach "$wt" "$sha" >/dev/null
  WORKTREES+=("$wt")
  echo "== compiler $label = $ref ($sha): cargo build --release"
  ( cd "$wt/crates/compiler" && cargo build --release 2>&1 | tail -1 )
  COMPILER_ARGS+=(--compiler "$label=$wt/crates/compiler/target/release/flashtex-compiler")
  LABELS+=(--label "compiler-$label=$ref @ $sha")
done

pdf_sha="$(git -C "$REPO" rev-parse --verify "$PDF_REF^{commit}")"
pdf_wt="$RUN_DIR/pdf"
git -C "$REPO" worktree add --detach "$pdf_wt" "$pdf_sha" >/dev/null
WORKTREES+=("$pdf_wt")
echo "== flashtex-pdf = $PDF_REF ($pdf_sha): cargo build --release"
( cd "$pdf_wt/crates/pdf" && cargo build --release 2>&1 | tail -1 )
PDF_BIN="$pdf_wt/crates/pdf/target/release/flashtex-pdf"

echo "== swiftc oracle_extract.swift"
swiftc -O "$HERE/oracle_extract.swift" -o "$RUN_DIR/oracle_extract"

echo "== oracle_compare.py"
python3 "$HERE/oracle_compare.py" \
  --pdflatex "$PDFLATEX" --extract "$RUN_DIR/oracle_extract" --pdf-bin "$PDF_BIN" \
  "${COMPILER_ARGS[@]}" --samples "$HERE/oracle-samples" --reports-dir "$REPORTS_DIR" \
  --scratch "$RUN_DIR/work" "${LABELS[@]}" ${ONLY[@]+"${ONLY[@]}"} \
  --label "pdf-crate=$PDF_REF @ $pdf_sha" \
  --label "suite=$(git -C "$REPO" rev-parse HEAD)" \
  --label "toolchain=$(swift --version 2>&1 | head -1); $(cargo --version); $(python3 --version); macOS $(sw_vers -productVersion)"
