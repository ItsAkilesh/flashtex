#!/usr/bin/env bash
# Orchestrate the visual corpus: build FlashTeX compiler(s) and flashtex-pdf from
# pinned git refs (exported with `git archive`, no worktrees), compile the shared
# rasterizer, render every fixture through the reference engines and FlashTeX,
# diff, and write tests/visual-corpus/evidence/<UTC>/{report.md,metrics.json,
# provenance.json,images/}. Reference engines are oracles only; FlashTeX never
# invokes them.
#
# Usage:
#   run.sh [--compiler-ref <label>=<ref> ...]   default: main=origin/main de1020c=de1020c
#          [--pdf-ref <ref>]                     default: 5b5f7b5
#          [--dpi 144] [--threshold 32] [--scratch <dir>] [--evidence-root <dir>]
#          [--engine pdflatex ...] [--thresholds <json>] [--regress <prev evidence dir>|auto]
#          [--native <dir>]   pre-captured native preview rasters (see capture_native.sh)
#          [--skip-build]     reuse binaries already in <scratch>/builds
set -euo pipefail
HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO="$(git -C "$HERE" rev-parse --show-toplevel)"
COMPILER_REFS=(); PDF_REF="5b5f7b5"; DPI=144; THRESHOLD=32
SCRATCH="${TMPDIR:-/tmp}/flashtex-visual-corpus"; EVROOT="$REPO/tests/visual-corpus/evidence"
ENGINES=(); THRESHOLDS="$HERE/thresholds.json"; REGRESS=""; NATIVE=""; SKIP_BUILD=0
while [[ $# -gt 0 ]]; do
  case "$1" in
    --compiler-ref) COMPILER_REFS+=("$2"); shift 2 ;;
    --pdf-ref) PDF_REF="$2"; shift 2 ;;
    --dpi) DPI="$2"; shift 2 ;;
    --threshold) THRESHOLD="$2"; shift 2 ;;
    --scratch) SCRATCH="$2"; shift 2 ;;
    --evidence-root) EVROOT="$2"; shift 2 ;;
    --engine) ENGINES+=("$2"); shift 2 ;;
    --thresholds) THRESHOLDS="$2"; shift 2 ;;
    --regress) REGRESS="$2"; shift 2 ;;
    --native) NATIVE="$2"; shift 2 ;;
    --skip-build) SKIP_BUILD=1; shift ;;
    -h|--help) sed -n '2,17p' "$0"; exit 0 ;;
    *) echo "unknown argument: $1" >&2; exit 2 ;;
  esac
done
[[ ${#COMPILER_REFS[@]} -gt 0 ]] || COMPILER_REFS=("main=origin/main" "de1020c=de1020c")
[[ ${#ENGINES[@]} -gt 0 ]] || ENGINES=(pdflatex xelatex lualatex)
STAMP="$(date -u +%Y%m%dT%H%M%SZ)"
WORK="$SCRATCH/run-$STAMP"; BUILDS="$SCRATCH/builds"
EVIDENCE="$EVROOT/$STAMP"
mkdir -p "$WORK" "$BUILDS" "$EVIDENCE"
echo "work: $WORK"; echo "evidence: $EVIDENCE"

# --- builds (git archive of the pinned commit; no checkout/worktree side effects)
build_crate() {  # label ref crate-dir binary
  local label="$1" ref="$2" crate="$3" bin="$4" sha dir
  sha="$(git -C "$REPO" rev-parse --verify "$ref^{commit}")"
  dir="$BUILDS/$label-$sha"
  if [[ $SKIP_BUILD -eq 0 || ! -x "$dir/$crate/target/release/$bin" ]]; then
    rm -rf "$dir"; mkdir -p "$dir"
    git -C "$REPO" archive "$sha" "$crate" | tar -x -C "$dir"
    echo "== cargo build --release: $label = $ref @ $sha"
    ( cd "$dir/$crate" && cargo build --release 2>&1 | tail -1 )
  fi
  echo "$sha $dir/$crate/target/release/$bin"
}
COMPILER_ARGS=(); COMPILER_JSON="["
for spec in "${COMPILER_REFS[@]}"; do
  label="${spec%%=*}"; ref="${spec#*=}"
  read -r sha bin < <(build_crate "$label" "$ref" crates/compiler flashtex-compiler | tail -1)
  COMPILER_ARGS+=(--compiler "$label=$bin")
  subject="$(git -C "$REPO" log -1 --format=%s "$sha")"
  COMPILER_JSON+="{\"label\":\"$label\",\"ref\":\"$ref\",\"sha\":\"$sha\",\"note\":$(python3 -c 'import json,sys;print(json.dumps(sys.argv[1]))' "$subject")},"
done
COMPILER_JSON="${COMPILER_JSON%,}]"
read -r pdf_sha PDF_BIN < <(build_crate pdf "$PDF_REF" crates/pdf flashtex-pdf | tail -1)

echo "== swiftc rasterize.swift"
swiftc -O "$HERE/rasterize.swift" -o "$WORK/rasterize"

# --- render both sides
ENGINE_ARGS=(); for e in "${ENGINES[@]}"; do ENGINE_ARGS+=(--engine "$e"); done
"$HERE/render_reference.sh" --out "$WORK/reference" --rasterize "$WORK/rasterize" --dpi "$DPI" "${ENGINE_ARGS[@]}"
"$HERE/render_flashtex.sh" --out "$WORK/flashtex" --rasterize "$WORK/rasterize" --pdf-bin "$PDF_BIN" \
  "${COMPILER_ARGS[@]}" --dpi "$DPI" --embed-font auto

# --- provenance
python3 - "$WORK" "$EVIDENCE" "$REPO" "$HERE" "$COMPILER_JSON" "$PDF_REF" "$pdf_sha" "$PDF_BIN" "$DPI" "$NATIVE" <<'PY'
import hashlib, json, os, platform, subprocess, sys
work, ev, repo, here, compilers, pdf_ref, pdf_sha, pdf_bin, dpi, native = sys.argv[1:11]
def sh(*cmd):
    try: return subprocess.run(cmd, capture_output=True, text=True).stdout.strip()
    except Exception as e: return f"unavailable: {e}"
fixtures = []
for f in sorted(os.listdir(os.path.join(here, "fixtures"))):
    if not f.endswith(".tex"): continue
    p = os.path.join(here, "fixtures", f)
    meta_p = p[:-4] + ".meta.json"
    meta = json.load(open(meta_p)) if os.path.exists(meta_p) else {}
    fixtures.append({"name": f, "sha256": hashlib.sha256(open(p, "rb").read()).hexdigest(),
                     "meta_sha256": hashlib.sha256(open(meta_p, "rb").read()).hexdigest() if os.path.exists(meta_p) else None,
                     "purpose": meta.get("purpose"), "meta": meta})
engines = json.load(open(os.path.join(work, "reference", "engines.json")))
pre, flags = {}, []
for fx in sorted(os.listdir(os.path.join(work, "reference"))):
    d = os.path.join(work, "reference", fx)
    if not os.path.isdir(d): continue
    for e in os.listdir(d):
        ej = os.path.join(d, e, "engine.json")
        if os.path.exists(ej):
            j = json.load(open(ej))
            if j.get("preamble"): pre[e] = j["preamble"]; flags = j.get("flags", flags)
try:
    import PIL; pillow = PIL.__version__
except Exception:
    pillow = "not importable"
embed = None
for fx in os.listdir(os.path.join(work, "flashtex")):
    for c in os.listdir(os.path.join(work, "flashtex", fx)):
        bj = os.path.join(work, "flashtex", fx, c, "build.json")
        if os.path.exists(bj):
            s = json.load(open(bj)).get("pdf_stderr", "")
            if "embedding subset of" in s: embed = s.split("note: ")[-1][:120]
prov = {
    "generated_utc": os.path.basename(ev), "machine": "mac-m1max-a",
    "os": "macOS " + sh("sw_vers", "-productVersion") + " " + platform.machine(),
    "swift": sh("swift", "--version").splitlines()[0] if sh("swift", "--version") else "?",
    "cargo": sh("cargo", "--version"), "python": platform.python_version(), "pillow": pillow,
    "suite_branch": sh("git", "-C", repo, "rev-parse", "--abbrev-ref", "HEAD"), "suite_sha": sh("git", "-C", repo, "rev-parse", "HEAD"),
    "input_main_sha": sh("git", "-C", repo, "rev-parse", "origin/main"),
    "harness_files_sha256": {f: hashlib.sha256(open(os.path.join(here, f), "rb").read()).hexdigest()
                             for f in sorted(os.listdir(here)) if os.path.isfile(os.path.join(here, f))},
    "engines": {k: v for k, v in engines.items() if not k.startswith("_")},
    "packages": engines.get("_packages", {}), "system_fonts": engines.get("_system_fonts", {}),
    "preambles": pre, "engine_flags": flags,
    "compilers": json.loads(compilers),
    "pdf_writer": {"ref": pdf_ref, "sha": pdf_sha, "path": pdf_bin, "embed": True, "embed_font": embed},
    "dpi": float(dpi), "color_space": "sRGB IEC61966-2.1", "pixel_format": "RGBA8, white opaque background",
    "rasterizer": "tests/visual-corpus/harness/rasterize.swift (CoreGraphics CGPDFDocument -> CGBitmapContext; CoreText for preview-equivalent)",
    "fixtures": fixtures, "work_dir": work,
    "native_preview": None,
    "limitations": [
        "Reference engines and fonts: pdflatex uses the psnfss `times` package (URW Nimbus Roman clone); xelatex and lualatex use "
        "the macOS system `Times New Roman` TrueType via fontspec. Neither is byte-identical to the Times-Roman standard-14 face "
        "CoreGraphics substitutes when rasterizing the FlashTeX PDF.",
        "Compiler build `main` (origin/main) has no math support: math fixtures compile with status `recovered` and the math is "
        "rendered as plain text; `de1020c` typesets math with Unicode symbols and U+2500 rule runs.",
        "The preview-equivalent raster re-implements the app's CoreText draw; it is not a capture of the SwiftUI preview.",
        "Metrics are for these fixtures, this DPI, these builds and this machine only.",
    ],
}
if native and os.path.isdir(native) and os.path.exists(os.path.join(native, "capture.json")):
    prov["native_preview"] = json.load(open(os.path.join(native, "capture.json")))
json.dump(prov, open(os.path.join(ev, "provenance.json"), "w"), indent=1, ensure_ascii=False)
PY

# --- diff + report
DIFF_ARGS=(--reference "$WORK/reference" --flashtex "$WORK/flashtex" --evidence "$EVIDENCE" --dpi "$DPI" \
  --threshold "$THRESHOLD" --provenance "$EVIDENCE/provenance.json")
[[ -f "$THRESHOLDS" ]] && DIFF_ARGS+=(--thresholds "$THRESHOLDS")
[[ -n "$NATIVE" ]] && DIFF_ARGS+=(--native "$NATIVE")
if [[ "$REGRESS" == "auto" ]]; then
  REGRESS="$(ls -d "$EVROOT"/*/ 2>/dev/null | grep -v "$STAMP" | sort | tail -1 || true)"
fi
[[ -n "$REGRESS" ]] && DIFF_ARGS+=(--regress "${REGRESS%/}")
set +e
python3 "$HERE/diff.py" "${DIFF_ARGS[@]}"
rc=$?
set -e
cp "$THRESHOLDS" "$EVIDENCE/thresholds.used.json" 2>/dev/null || true
echo "report: $EVIDENCE/report.md (diff exit $rc)"
exit $rc
