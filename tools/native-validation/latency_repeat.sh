#!/usr/bin/env bash
# Repeated compiler-latency measurement for non-regression: builds the compiler
# from a ref in a scratch worktree, runs e2e_latency.py R times (fresh worker
# process each run, N requests each) over a document, and reports per-run
# min/median/max plus the spread of the medians across runs. Optionally compares
# the median-of-medians against a previous latency report.
#
# Usage: latency_repeat.sh [--compiler-ref <ref>] [--document <file>] [--runs 3] [--count 20]
#                          [--reports-dir <dir>] [--scratch <dir>] [--prev <latency-*.md>]
set -euo pipefail
HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO="$(git -C "$HERE" rev-parse --show-toplevel)"
COMPILER_REF="origin/main"
DOCUMENT=""
RUNS=3
COUNT=20
REPORTS_DIR="$HERE/reports"
SCRATCH="${TMPDIR:-/tmp}/flashtex-validation"
PREV=""
while [[ $# -gt 0 ]]; do
  case "$1" in
    --compiler-ref) COMPILER_REF="$2"; shift 2 ;;
    --document) DOCUMENT="$2"; shift 2 ;;
    --runs) RUNS="$2"; shift 2 ;;
    --count) COUNT="$2"; shift 2 ;;
    --reports-dir) REPORTS_DIR="$2"; shift 2 ;;
    --scratch) SCRATCH="$2"; shift 2 ;;
    --prev) PREV="$2"; shift 2 ;;
    *) echo "unknown argument: $1" >&2; exit 2 ;;
  esac
done
STAMP="$(date -u +%Y%m%dT%H%M%SZ)"
RUN="$SCRATCH/latency-$STAMP"
mkdir -p "$RUN" "$REPORTS_DIR"
SHA="$(git -C "$REPO" rev-parse --verify "$COMPILER_REF^{commit}")"
WT="$RUN/compiler"
cleanup() { git -C "$REPO" worktree remove --force "$WT" >/dev/null 2>&1 || true; git -C "$REPO" worktree prune >/dev/null 2>&1 || true; }
trap cleanup EXIT
git -C "$REPO" worktree add --detach "$WT" "$SHA" >/dev/null
( cd "$WT/crates/compiler" && cargo build --release 2>&1 | tail -1 )
BIN="$WT/crates/compiler/target/release/flashtex-compiler"
[[ -n "$DOCUMENT" ]] || DOCUMENT="$WT/apps/mac/Samples/demo.tex"
[[ -f "$DOCUMENT" ]] || DOCUMENT="$HERE/oracle-samples/demo.tex"
[[ -n "$PREV" ]] || PREV="$(ls -1 "$REPORTS_DIR"/latency-*.md 2>/dev/null | sort | tail -1 || true)"

LOAD="$(sysctl -n vm.loadavg 2>/dev/null | tr -d '{}')"
for i in $(seq 1 "$RUNS"); do
  python3 "$HERE/e2e_latency.py" --compiler "$BIN" --document "$DOCUMENT" --count "$COUNT" --json "$RUN/run$i.json" >"$RUN/run$i.txt"
done
OUT="$REPORTS_DIR/latency-$STAMP.md"
python3 - "$RUN" "$RUNS" "$OUT" "$COMPILER_REF" "$SHA" "$DOCUMENT" "$LOAD" "$PREV" <<'PY'
import json, os, re, statistics, sys
run, runs, out, ref, sha, doc, load, prev = sys.argv[1], int(sys.argv[2]), sys.argv[3], sys.argv[4], sys.argv[5], sys.argv[6], sys.argv[7], sys.argv[8]
rows = [json.load(open(os.path.join(run, f"run{i}.json"))) for i in range(1, runs + 1)]
medians = [r["median_ms"] for r in rows]
mins = [r["min_ms"] for r in rows]
maxs = [r["max_ms"] for r in rows]
allsamples = [s for r in rows for s in r["samples_ms"]]
mom = statistics.median(medians)
spread = max(medians) - min(medians)
sd = statistics.pstdev(medians) if len(medians) > 1 else 0.0
cv = (sd / mom * 100) if mom else 0.0
L = ["# Compiler latency, repeated runs", "",
     f"- Generated: {os.path.basename(out)[8:-3]}; compiler `{ref}` = `{sha}`; document `{doc}` ({rows[0]['document_bytes']} bytes, {rows[0]['pages']} page(s), status {','.join(rows[0]['statuses'])})",
     f"- {runs} runs x {rows[0]['count']} requests, fresh worker process per run, JSON Lines pipe round trip (no UI); load average before runs: {load.strip()}", "",
     "| Run | min ms | median ms | max ms |", "|---|---|---|---|"]
for i, r in enumerate(rows, 1):
    L.append(f"| {i} | {r['min_ms']:.3f} | {r['median_ms']:.3f} | {r['max_ms']:.3f} |")
L += ["", f"- Median of medians: **{mom:.3f} ms**; spread of medians (max - min): {spread:.3f} ms; population SD of medians: {sd:.3f} ms (CV {cv:.1f}%)",
      f"- Across all {len(allsamples)} samples: min {min(allsamples):.3f} / median {statistics.median(allsamples):.3f} / p95 {sorted(allsamples)[int(0.95 * len(allsamples)) - 1]:.3f} / max {max(allsamples):.3f} ms",
      "- First request of each run (cold worker): " + ", ".join("%.3f" % r["samples_ms"][0] for r in rows) + " ms"]
verdict = "no previous latency report"
if prev and os.path.exists(prev):
    t = open(prev, encoding="utf-8").read()
    m = re.search(r"Median of medians: \*\*([\d.]+) ms\*\*", t)
    if m:
        pm = float(m.group(1))
        ratio = mom / pm if pm else float("inf")
        verdict = f"vs `{os.path.basename(prev)}` median-of-medians {pm:.3f} ms -> {mom:.3f} ms (x{ratio:.2f}); " + ("within 2x, no regression flagged" if 0.5 <= ratio <= 2.0 else "**outside 2x, flag for a human**")
L += ["", f"- Non-regression: {verdict}", "",
      "Numbers depend on machine load; a 2x band is used because sub-millisecond medians jitter with concurrent builds on this Mac."]
open(out, "w", encoding="utf-8").write("\n".join(L) + "\n")
print("\n".join(L))
print(f"\nwrote {out}")
PY
