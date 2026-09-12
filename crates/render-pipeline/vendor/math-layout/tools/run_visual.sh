#!/usr/bin/env bash
# Raster gate for the math visual corpus: runs the visual-oracle harness
# (tests/visual-corpus/harness on origin/agent/mac-visual-oracle/reference-raster,
# exported from a pinned commit with `git archive`; nothing in it is edited) over
# crates/math-layout/fixtures/visual, with this crate's corpus compiler as the
# FlashTeX side and the pinned crates/pdf as the PDF writer.
#
# Reference engines are oracles only. The corpus compiler is invoked through a
# wrapper that adds `layout_capabilities: ["rules-v1","font-hints-v1"]` to the
# harness's legacy request, so typed rules and font hints reach the PDF writer.
#
# Usage: tools/run_visual.sh [--harness-ref <sha>] [--pdf-ref <sha>] [--dpi 144]
#                            [--scratch <dir>] [--evidence <dir>] [--regress <prev evidence dir>]
#                            [--skip-build] [--engine pdflatex ...]
# Output: <evidence>/{report.md,metrics.json,provenance.json,images/}; exit code
# is diff.py's (0 ok, 3 regression, 4 threshold failure).
set -euo pipefail
HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
CRATE="$(dirname "$HERE")"
REPO="$(git -C "$CRATE" rev-parse --show-toplevel)"
HARNESS_REF="db182361a7923d911e7beb8883b2febea0b19055"   # origin/agent/mac-visual-oracle/reference-raster
PDF_REF="4bd8c2e79f66c161b7fb6438f3262f8d980eb8c9"       # origin/agent/mac-pdf/pdf-output: rules-v1 + font-hints-v1
DPI=144; SCRATCH="${TMPDIR:-/tmp}/flashtex-math-visual"; EVIDENCE=""; REGRESS=""; SKIP_BUILD=0
ENGINES=()
while [[ $# -gt 0 ]]; do
  case "$1" in
    --harness-ref) HARNESS_REF="$2"; shift 2 ;;
    --pdf-ref) PDF_REF="$2"; shift 2 ;;
    --dpi) DPI="$2"; shift 2 ;;
    --scratch) SCRATCH="$2"; shift 2 ;;
    --evidence) EVIDENCE="$2"; shift 2 ;;
    --regress) REGRESS="$2"; shift 2 ;;
    --skip-build) SKIP_BUILD=1; shift ;;
    --engine) ENGINES+=("$2"); shift 2 ;;
    -h|--help) sed -n '2,17p' "$0"; exit 0 ;;
    *) echo "unknown argument: $1" >&2; exit 2 ;;
  esac
done
[[ ${#ENGINES[@]} -gt 0 ]] || ENGINES=(pdflatex)
STAMP="$(date -u +%Y%m%dT%H%M%SZ)"
WORK="$SCRATCH/run-$STAMP"; BUILDS="$SCRATCH/builds"
[[ -n "$EVIDENCE" ]] || EVIDENCE="$CRATE/docs/visual-evidence/$STAMP"
mkdir -p "$WORK" "$BUILDS" "$EVIDENCE"
echo "work: $WORK"; echo "evidence: $EVIDENCE"

HARNESS_SHA="$(git -C "$REPO" rev-parse --verify "$HARNESS_REF^{commit}")"
PDF_SHA="$(git -C "$REPO" rev-parse --verify "$PDF_REF^{commit}")"

# --- harness export (read-only copy of the pinned commit)
HDIR="$BUILDS/harness-$HARNESS_SHA"
if [[ ! -d "$HDIR/tests/visual-corpus/harness" ]]; then
  mkdir -p "$HDIR"
  git -C "$REPO" archive "$HARNESS_SHA" tests/visual-corpus/harness | tar -x -C "$HDIR"
fi
HARNESS="$HDIR/tests/visual-corpus/harness"

# --- flashtex-pdf from the pinned commit
PDIR="$BUILDS/pdf-$PDF_SHA"
PDF_BIN="$PDIR/crates/pdf/target/release/flashtex-pdf"
if [[ $SKIP_BUILD -eq 0 || ! -x "$PDF_BIN" ]]; then
  rm -rf "$PDIR"; mkdir -p "$PDIR"
  git -C "$REPO" archive "$PDF_SHA" crates/pdf | tar -x -C "$PDIR"
  echo "== cargo build --release: flashtex-pdf @ $PDF_SHA"
  ( cd "$PDIR/crates/pdf" && cargo build --release 2>&1 | tail -1 )
fi

# --- corpus compiler from this checkout
echo "== cargo build --release: flashtex-math-corpus (this checkout)"
( cd "$CRATE" && cargo build --release --bin flashtex-math-corpus 2>&1 | tail -1 )
CORPUS_BIN="$CRATE/target/release/flashtex-math-corpus"
WRAPPER="$WORK/flashtex-math-corpus-negotiated"
cat > "$WRAPPER" <<EOF
#!/usr/bin/env bash
# Adds the negotiated capabilities to the harness's legacy compile request.
python3 -c '
import json, sys
for line in sys.stdin:
    if not line.strip(): continue
    req = json.loads(line)
    req.setdefault("payload", {})["layout_capabilities"] = ["rules-v1", "font-hints-v1"]
    sys.stdout.write(json.dumps(req, ensure_ascii=False) + "\n")
' | "$CORPUS_BIN"
EOF
chmod +x "$WRAPPER"

echo "== swiftc rasterize.swift"
swiftc -O "$HARNESS/rasterize.swift" -o "$WORK/rasterize"

ENGINE_ARGS=(); for e in "${ENGINES[@]}"; do ENGINE_ARGS+=(--engine "$e"); done
"$HARNESS/render_reference.sh" --out "$WORK/reference" --rasterize "$WORK/rasterize" --dpi "$DPI" \
  --fixtures "$CRATE/fixtures/visual" "${ENGINE_ARGS[@]}"
"$HARNESS/render_flashtex.sh" --out "$WORK/flashtex" --rasterize "$WORK/rasterize" --pdf-bin "$PDF_BIN" \
  --compiler "math=$WRAPPER" --fixtures "$CRATE/fixtures/visual" --dpi "$DPI" --embed-font auto

# --- provenance
python3 - "$WORK" "$EVIDENCE" "$REPO" "$CRATE" "$HARNESS_SHA" "$PDF_SHA" "$DPI" <<'PY'
import hashlib, json, os, platform, subprocess, sys
work, ev, repo, crate, hsha, psha, dpi = sys.argv[1:8]
def sh(*cmd):
    try: return subprocess.run(cmd, capture_output=True, text=True).stdout.strip()
    except Exception as e: return f"unavailable: {e}"
fx = []
d = os.path.join(crate, "fixtures", "visual")
for f in sorted(os.listdir(d)):
    if f.endswith(".tex"):
        p = os.path.join(d, f); mp = p[:-4] + ".meta.json"
        meta = json.load(open(mp)) if os.path.exists(mp) else {}
        fx.append({"name": f, "sha256": hashlib.sha256(open(p, "rb").read()).hexdigest(),
                   "purpose": meta.get("purpose"), "meta": meta})
engines = json.load(open(os.path.join(work, "reference", "engines.json")))
prov = {
    "generated_utc": os.path.basename(ev), "machine": "mac-m1max-a",
    "os": "macOS " + sh("sw_vers", "-productVersion") + " " + platform.machine(),
    "swift": (sh("swift", "--version").splitlines() or ["?"])[0], "cargo": sh("cargo", "--version"),
    "python": platform.python_version(),
    "suite_sha": sh("git", "-C", repo, "rev-parse", "HEAD"), "harness_sha": hsha, "pdf_writer_sha": psha,
    "compilers": [{"label": "math", "ref": "HEAD (this checkout)", "sha": sh("git", "-C", repo, "rev-parse", "HEAD"),
                   "crate": "crates/math-layout", "binary": "flashtex-math-corpus",
                   "note": "declared-corpus lookup with rules-v1 + font-hints-v1 negotiated by a wrapper"}],
    "pdf_writer": {"ref": "origin/agent/mac-pdf/pdf-output", "sha": psha, "embed": True,
                   "embed_font": "auto (Latin Modern via font hints; see build.json pdf_stderr)"},
    "suite_branch": sh("git", "-C", repo, "rev-parse", "--abbrev-ref", "HEAD"),
    "input_main_sha": sh("git", "-C", repo, "rev-parse", "origin/main"),
    "engines": {k: v for k, v in engines.items() if not k.startswith("_")}, "packages": engines.get("_packages", {}),
    "dpi": float(dpi), "fixtures": fx, "work_dir": work,
    "limitations": [
        "The reference preambles are the harness's (12pt times / lmodern); the corpus compiler lays out with Computer Modern metrics "
        "(CmMathMetrics::latex_12pt). Latin Modern text glyph heights and italic corrections differ from cmr12 in places, and the "
        "times variant's text font is not CM at all, so raster metrics mix font differences with layout differences; the structural "
        "gate (tools/structural_gate.py, real CM oracle) isolates layout.",
        "The harness's preview-equivalent raster draws every text item in Times-Roman and skips typed rule items; the export side "
        "(flashtex-pdf) draws typed rules and resolves Latin Modern font hints, substituting the document face for the math symbol "
        "and extension families with a warning.",
        "Metrics are for these fixtures, this DPI, these builds and this machine only.",
    ],
}
json.dump(prov, open(os.path.join(ev, "provenance.json"), "w"), indent=1, ensure_ascii=False)
PY

DIFF_ARGS=(--reference "$WORK/reference" --flashtex "$WORK/flashtex" --evidence "$EVIDENCE" --dpi "$DPI" \
  --threshold 32 --provenance "$EVIDENCE/provenance.json" --thresholds "$CRATE/fixtures/visual/raster-thresholds.json" \
  --images-for-sides export --images-for-engines pdflatex-lm --max-png-bytes 45000)
[[ -n "$REGRESS" ]] && DIFF_ARGS+=(--regress "${REGRESS%/}")
set +e
python3 "$HARNESS/diff.py" "${DIFF_ARGS[@]}"
rc=$?
set -e
cp "$CRATE/fixtures/visual/raster-thresholds.json" "$EVIDENCE/thresholds.used.json"
echo "report: $EVIDENCE/report.md (diff exit $rc)"
exit $rc
