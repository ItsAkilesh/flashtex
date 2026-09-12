#!/usr/bin/env bash
# Orchestrate the visual corpus: build FlashTeX compiler(s) and flashtex-pdf from
# pinned git refs (exported with `git archive`, no worktrees), compile the shared
# rasterizer, render every fixture through the reference engines and FlashTeX,
# diff, and write tests/visual-corpus/evidence/<UTC>/{report.md,metrics.json,
# provenance.json,images/}. Reference engines are oracles only; FlashTeX never
# invokes them.
#
# Usage:
#   run.sh [--compiler-ref <label>=<ref>[:<crate dir>:<binary>] ...]
#              default: main=origin/main de1020c=de1020c, plus
#              pipeline=origin/agent/mac-render-pipeline/unified:crates/render-pipeline:flashtex-render
#              whenever that branch exists and contains the crate (same JSON Lines interface)
#          [--pdf-ref <ref>]                     default: 5b5f7b5
#          [--dpi 144] [--threshold 32] [--scratch <dir>] [--evidence-root <dir>]
#          [--engine pdflatex ...] [--thresholds <json>] [--regress <prev evidence dir>|auto]
#          [--native <dir>]   pre-captured native preview rasters (see capture_native.sh)
#          [--app <FlashTeXMac binary>]  capture the native preview during this run (needs Screen Recording)
#          [--skip-build]     reuse binaries already in <scratch>/builds
#          [--profile <json>] pinned raw-PDF SHA-256 profile (default harness/reference-profile.json)
#          [--pin-profile]    explicitly re-baseline the profile from this run's PDFs
#          [--gate]           exit 5 when an exact-equality gate fails (default: report only)
#          [--exact-route auto|none|<pdf ref>[:<render ref>]]  third candidate column `exact`:
#                             flashtex-render --v2 (built from the pipeline ref) -> flashtex-pdf-exact from-v2
#                             (built from <pdf ref>, default origin/agent/mac-pdf/v2-adapter, crates/pdf).
#                             auto (default) adds it whenever both build; a build failure is reported.
#          [--font-dir <dir>] extra font directory for from-v2 (default: the TeX Live Latin Modern dirs it finds itself)
#          [--oracle-profile <json>] pinned ESTABLISHED-ENGINE profile (default harness/oracle-profile.json):
#                             SHA-256 of the oracle PDF per fixture/oracle with engine version, distribution,
#                             fonts, preamble, flags and render environment. Candidate bytes are compared to it raw.
#          [--pin-oracle]     write that profile from THIS run's fresh oracle renders (never from reused PDFs);
#                             a pin run is a baseline for the oracle-bytes gate, not a pass
#          [--reference-from <evidence dir>|auto|none]  stored oracle renders to reuse when an engine
#                             is not installed (default auto: every evidence dir that has
#                             references/manifest.json, newest first). A run that renders
#                             references itself writes them to <evidence>/references/ so later
#                             runs can reuse them with the original engine/version pins.
set -euo pipefail
HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO="$(git -C "$HERE" rev-parse --show-toplevel)"
COMPILER_REFS=(); PDF_REF="5b5f7b5"; DPI=144; THRESHOLD=32
SCRATCH="${TMPDIR:-/tmp}/flashtex-visual-corpus"; EVROOT="$REPO/tests/visual-corpus/evidence"
ENGINES=(); THRESHOLDS="$HERE/thresholds.json"; REGRESS=""; NATIVE=""; SKIP_BUILD=0; APP=""
PROFILE="$HERE/reference-profile.json"; PIN=0; GATE=0; REF_FROM=()
ORACLE_PROFILE="$HERE/oracle-profile.json"; PIN_ORACLE=0
EXACT_ROUTE="auto"; EXACT_PDF_REF="origin/agent/mac-pdf/v2-adapter"; EXACT_RENDER_REF=""; FONT_DIRS=()
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
    --app) APP="$2"; shift 2 ;;
    --skip-build) SKIP_BUILD=1; shift ;;
    --profile) PROFILE="$2"; shift 2 ;;
    --pin-profile) PIN=1; shift ;;
    --gate) GATE=1; shift ;;
    --reference-from) REF_FROM+=("$2"); shift 2 ;;
    --oracle-profile) ORACLE_PROFILE="$2"; shift 2 ;;
    --pin-oracle) PIN_ORACLE=1; shift ;;
    --exact-route) EXACT_ROUTE="$2"; shift 2 ;;
    --font-dir) FONT_DIRS+=("$2"); shift 2 ;;
    -h|--help) sed -n '2,17p' "$0"; exit 0 ;;
    *) echo "unknown argument: $1" >&2; exit 2 ;;
  esac
done
if [[ ${#COMPILER_REFS[@]} -eq 0 ]]; then
  COMPILER_REFS=("main=origin/main" "de1020c=de1020c")
  PIPE_REF="origin/agent/mac-render-pipeline/unified"
  if git -C "$REPO" rev-parse --verify -q "$PIPE_REF^{commit}" >/dev/null \
     && git -C "$REPO" cat-file -e "$PIPE_REF:crates/render-pipeline/Cargo.toml" 2>/dev/null; then
    COMPILER_REFS+=("pipeline=$PIPE_REF:crates/render-pipeline:flashtex-render")
    echo "pipeline build available: $PIPE_REF"
  else
    echo "pipeline build not available yet ($PIPE_REF has no crates/render-pipeline); skipping"
  fi
fi
[[ ${#ENGINES[@]} -gt 0 ]] || ENGINES=(pdflatex xelatex lualatex)
STAMP="$(date -u +%Y%m%dT%H%M%SZ)"
WORK="$SCRATCH/run-$STAMP"; BUILDS="$SCRATCH/builds"
EVIDENCE="$EVROOT/$STAMP"
mkdir -p "$WORK" "$BUILDS" "$EVIDENCE"
echo "work: $WORK"; echo "evidence: $EVIDENCE"

# --- builds (git archive of the pinned commit; no checkout/worktree side effects)
build_crate() {  # label ref crate-dir binary -> "<sha> <bin-or-FAILED> <build log>"
  local label="$1" ref="$2" crate="$3" bin="$4" sha dir
  sha="$(git -C "$REPO" rev-parse --verify "$ref^{commit}")"
  dir="$BUILDS/$label-$sha"
  if [[ $SKIP_BUILD -eq 0 || ! -x "$dir/$crate/target/release/$bin" ]]; then
    rm -rf "$dir"; mkdir -p "$dir"
    # The whole crates/ tree is exported: some crates depend on siblings by path.
    git -C "$REPO" archive "$sha" crates | tar -x -C "$dir"
    echo "== cargo build --release: $label = $ref @ $sha" >&2
    if ! ( cd "$dir/$crate" && cargo build --release > "$dir/build.log" 2>&1 ); then
      echo "   build FAILED: $(grep -m1 -E '^error' "$dir/build.log" || tail -1 "$dir/build.log")" >&2
    fi
  fi
  if [[ -x "$dir/$crate/target/release/$bin" ]]; then
    echo "$sha $dir/$crate/target/release/$bin $dir/build.log"
  else
    echo "$sha FAILED $dir/build.log"
  fi
}
COMPILER_ARGS=(); COMPILER_JSON="["
for spec in "${COMPILER_REFS[@]}"; do
  label="${spec%%=*}"; rest="${spec#*=}"
  IFS=: read -r ref crate binname <<<"$rest"
  crate="${crate:-crates/compiler}"; binname="${binname:-flashtex-compiler}"
  read -r sha bin blog < <(build_crate "$label" "$ref" "$crate" "$binname" | tail -1)
  subject="$(git -C "$REPO" log -1 --format=%s "$sha")"
  if [[ "$bin" == "FAILED" ]]; then
    err="$(grep -m1 -E '^error' "$blog" 2>/dev/null || tail -1 "$blog" 2>/dev/null || echo unknown)"
    COMPILER_JSON+="{\"label\":\"$label\",\"ref\":\"$ref\",\"sha\":\"$sha\",\"crate\":\"$crate\",\"binary\":\"$binname\",\"build_ok\":false,\"build_error\":$(python3 -c 'import json,sys;print(json.dumps(sys.argv[1][:300]))' "$err"),\"note\":$(python3 -c 'import json,sys;print(json.dumps(sys.argv[1]))' "$subject")},"
    echo "compiler $label ($ref @ $sha) did not build; reported, not used"
    continue
  fi
  COMPILER_ARGS+=(--compiler "$label=$bin")
  COMPILER_JSON+="{\"label\":\"$label\",\"ref\":\"$ref\",\"sha\":\"$sha\",\"crate\":\"$crate\",\"binary\":\"$binname\",\"build_ok\":true,\"note\":$(python3 -c 'import json,sys;print(json.dumps(sys.argv[1]))' "$subject")},"
done
# --- exact route (third candidate column): flashtex-render --v2 -> flashtex-pdf-exact from-v2
EXACT_ARGS=()
if [[ "$EXACT_ROUTE" != "none" ]]; then
  if [[ "$EXACT_ROUTE" != "auto" ]]; then
    EXACT_PDF_REF="${EXACT_ROUTE%%:*}"; [[ "$EXACT_ROUTE" == *:* ]] && EXACT_RENDER_REF="${EXACT_ROUTE#*:}"
  fi
  [[ -n "$EXACT_RENDER_REF" ]] || EXACT_RENDER_REF="origin/agent/mac-render-pipeline/unified"
  if git -C "$REPO" rev-parse --verify -q "$EXACT_PDF_REF^{commit}" >/dev/null \
     && git -C "$REPO" rev-parse --verify -q "$EXACT_RENDER_REF^{commit}" >/dev/null \
     && git -C "$REPO" cat-file -e "$EXACT_RENDER_REF:crates/render-pipeline/Cargo.toml" 2>/dev/null; then
    read -r xr_sha xr_bin xr_log < <(build_crate exact-render "$EXACT_RENDER_REF" crates/render-pipeline flashtex-render | tail -1)
    read -r xp_sha xp_bin xp_log < <(build_crate exact-pdf "$EXACT_PDF_REF" crates/pdf flashtex-pdf-exact | tail -1)
    if [[ "$xr_bin" != "FAILED" && "$xp_bin" != "FAILED" ]]; then
      EXACT_ARGS=(--exact "exact=$xr_bin:$xp_bin")
      note="$(git -C "$REPO" log -1 --format=%s "$xp_sha")"
      COMPILER_JSON="${COMPILER_JSON%]}"; [[ "$COMPILER_JSON" != "[" ]] && COMPILER_JSON+=","
      COMPILER_JSON+="{\"label\":\"exact\",\"route\":\"exact\",\"ref\":\"$EXACT_RENDER_REF\",\"sha\":\"$xr_sha\",\"crate\":\"crates/render-pipeline\",\"binary\":\"flashtex-render --secnumdepth 0 --v2\",\"exact_pdf_ref\":\"$EXACT_PDF_REF\",\"exact_pdf_sha\":\"$xp_sha\",\"exact_pdf_crate\":\"crates/pdf\",\"exact_pdf_binary\":\"flashtex-pdf-exact from-v2\",\"build_ok\":true,\"note\":$(python3 -c 'import json,sys;print(json.dumps(sys.argv[1]))' "$note")}]"
      echo "exact route available: render $EXACT_RENDER_REF @ ${xr_sha:0:7}, pdf-exact $EXACT_PDF_REF @ ${xp_sha:0:7}"
    else
      err="$( { [[ "$xr_bin" == "FAILED" ]] && grep -m1 -E '^error' "$xr_log"; [[ "$xp_bin" == "FAILED" ]] && grep -m1 -E '^error' "$xp_log"; } 2>/dev/null | head -1 || echo unknown)"
      COMPILER_JSON="${COMPILER_JSON%]}"; [[ "$COMPILER_JSON" != "[" ]] && COMPILER_JSON+=","
      COMPILER_JSON+="{\"label\":\"exact\",\"route\":\"exact\",\"ref\":\"$EXACT_RENDER_REF\",\"sha\":\"$xr_sha\",\"exact_pdf_ref\":\"$EXACT_PDF_REF\",\"exact_pdf_sha\":\"$xp_sha\",\"build_ok\":false,\"build_error\":$(python3 -c 'import json,sys;print(json.dumps(sys.argv[1][:300]))' "$err"),\"note\":\"exact route did not build (render $xr_bin, pdf-exact $xp_bin)\"}]"
      echo "exact route did not build (render $xr_bin, pdf-exact $xp_bin); reported, not used"
    fi
  else
    echo "exact route not available ($EXACT_PDF_REF or $EXACT_RENDER_REF missing); skipping"
  fi
fi
FONT_ARGS=(); for fd in "${FONT_DIRS[@]}"; do FONT_ARGS+=(--font-dir "$fd"); done
[[ ${#COMPILER_ARGS[@]} -gt 0 ]] || { echo "no compiler built" >&2; exit 1; }
read -r pdf_sha PDF_BIN pdf_log < <(build_crate pdf "$PDF_REF" crates/pdf flashtex-pdf | tail -1)
[[ "$PDF_BIN" != "FAILED" ]] || { echo "flashtex-pdf did not build ($pdf_log)" >&2; exit 1; }

echo "== swiftc rasterize.swift"
swiftc -O "$HERE/rasterize.swift" -o "$WORK/rasterize"

# --- reference stores (reused only for engines that are not installed)
if [[ ${#REF_FROM[@]} -eq 0 || "${REF_FROM[0]}" == "auto" ]]; then
  REF_FROM=()
  for d in $(ls -d "$EVROOT"/*/ 2>/dev/null | sort -r); do
    [[ -f "${d%/}/references/manifest.json" ]] && REF_FROM+=("${d%/}")
  done
elif [[ "${REF_FROM[0]}" == "none" ]]; then
  REF_FROM=()
fi
REF_ARGS=(); for d in "${REF_FROM[@]}"; do REF_ARGS+=(--reference-from "$d"); done
MISSING=(); for e in "${ENGINES[@]}"; do [[ -x "/Library/TeX/texbin/$e" ]] || MISSING+=("$e"); done
if [[ ${#MISSING[@]} -gt 0 ]]; then
  echo "engines not installed: ${MISSING[*]}; stored references will be reused from: ${REF_FROM[*]:-(none)}"
fi

# --- render both sides
ENGINE_ARGS=(); for e in "${ENGINES[@]}"; do ENGINE_ARGS+=(--engine "$e"); done
"$HERE/render_reference.sh" --out "$WORK/reference" --rasterize "$WORK/rasterize" --dpi "$DPI" "${ENGINE_ARGS[@]}" "${REF_ARGS[@]}"

# --- durable reference store: PDFs rendered by an installed engine THIS run (never the reused ones)
python3 - "$WORK/reference" "$EVIDENCE" "$STAMP" <<'PY'
import json, os, shutil, sys
work, ev, stamp = sys.argv[1:4]
engines = json.load(open(os.path.join(work, "engines.json")))
entries = {}
for fx in sorted(os.listdir(work)):
    d = os.path.join(work, fx)
    if not os.path.isdir(d): continue
    for label in sorted(os.listdir(d)):
        ej, pdf = os.path.join(d, label, "engine.json"), os.path.join(d, label, "main.pdf")
        if not (os.path.exists(ej) and os.path.exists(pdf)): continue
        e = json.load(open(ej))
        if e.get("reused") or e.get("exit") != 0: continue
        dst = os.path.join(ev, "references", fx, label); os.makedirs(dst, exist_ok=True)
        shutil.copy2(pdf, os.path.join(dst, "main.pdf"))
        e2 = dict(e); e2["engine_version"] = engines.get(label.split("-")[0], {}).get("version"); e2["rendered_in_run"] = stamp
        json.dump(e2, open(os.path.join(dst, "engine.json"), "w"), indent=2)
        entries[f"{fx}/{label}"] = {"fixture_sha256": e.get("fixture_sha256"), "pdf_sha256": e.get("pdf_sha256"), "engine": label,
                                    "engine_version": e2["engine_version"], "variant": e.get("variant"), "exit": e.get("exit")}
if entries:
    json.dump({"schema_version": 1, "run": stamp, "machine": "mac-m1max-a",
               "note": "Reference oracle renders (PDF + engine metadata) rendered by this run; rasters/word boxes are re-derived from the PDF. "
                       "Later runs without an engine reuse these after a fixture SHA-256 + preamble check.",
               "engines": {k: v for k, v in engines.items() if not k.startswith("_")},
               "packages": engines.get("_packages"), "system_fonts": engines.get("_system_fonts"), "entries": entries},
              open(os.path.join(ev, "references", "manifest.json"), "w"), indent=1)
    print(f"reference store written: {len(entries)} fresh renders -> {ev}/references")
else:
    print("no fresh reference renders this run; no reference store written")
PY
"$HERE/render_flashtex.sh" --out "$WORK/flashtex" --rasterize "$WORK/rasterize" --pdf-bin "$PDF_BIN" \
  "${COMPILER_ARGS[@]}" "${EXACT_ARGS[@]}" "${FONT_ARGS[@]}" --reference "$WORK/reference" --dpi "$DPI" --embed-font auto

# --- native preview capture (optional; the actual SwiftUI preview, not the preview-equivalent raster)
if [[ -n "$APP" ]]; then
  NATIVE="$WORK/native"
  "$HERE/capture_native.sh" --out "$NATIVE" --app "$APP" --repo "$REPO" --rasterize "$WORK/rasterize" \
    "${COMPILER_ARGS[@]}" --dpi "$DPI" || echo "native capture failed (continuing without it)"
fi

# --- established-engine pin (explicit; from fresh renders only)
if [[ $PIN_ORACLE -eq 1 ]]; then
  python3 "$HERE/pin_oracle.py" "$WORK/reference" "$ORACLE_PROFILE" "$STAMP" "$HERE/fixtures"
fi

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
references = {"fresh": [], "reused": {}, "unavailable": [], "stores": engines.get("_reference_stores", [])}
for fx in sorted(os.listdir(os.path.join(work, "reference"))):
    d = os.path.join(work, "reference", fx)
    if not os.path.isdir(d): continue
    for e in sorted(os.listdir(d)):
        ej = os.path.join(d, e, "engine.json")
        if not os.path.exists(ej): continue
        j = json.load(open(ej))
        key = f"{fx}/{e}"
        if j.get("reused"):
            rf = j.get("reused_from", {})
            references["reused"][key] = {"run": rf.get("run"), "engine_version": rf.get("engine_version"), "pdf_sha256": j.get("pdf_sha256"),
                                         "fixture_sha256": j.get("fixture_sha256")}
        elif j.get("available") and j.get("exit") == 0:
            references["fresh"].append(key)
        else:
            references["unavailable"].append({"key": key, "reason": j.get("reason") or f"engine exit {j.get('exit')}"})
pre, flags, engine_env = {}, [], []
for fx in sorted(os.listdir(os.path.join(work, "reference"))):
    d = os.path.join(work, "reference", fx)
    if not os.path.isdir(d): continue
    for e in os.listdir(d):
        ej = os.path.join(d, e, "engine.json")
        if os.path.exists(ej):
            j = json.load(open(ej))
            if j.get("preamble"): pre[e] = j["preamble"]; flags = j.get("flags", flags); engine_env = j.get("env") or engine_env
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
    "references": references,
    "packages": engines.get("_packages", {}), "system_fonts": engines.get("_system_fonts", {}),
    "preambles": pre, "engine_flags": flags, "engine_env": engine_env,
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
        "Exact-equality gates are the only acceptance signals; thresholds, SSIM, registration and regression numbers are diagnostics.",
    ],
}
if native and os.path.isdir(native) and os.path.exists(os.path.join(native, "capture.json")):
    prov["native_preview"] = json.load(open(os.path.join(native, "capture.json")))
json.dump(prov, open(os.path.join(ev, "provenance.json"), "w"), indent=1, ensure_ascii=False)
PY

# --- diff + report
DIFF_ARGS=(--reference "$WORK/reference" --flashtex "$WORK/flashtex" --evidence "$EVIDENCE" --dpi "$DPI" \
  --threshold "$THRESHOLD" --provenance "$EVIDENCE/provenance.json" --rasterize "$WORK/rasterize")
[[ -f "$THRESHOLDS" ]] && DIFF_ARGS+=(--thresholds "$THRESHOLDS")
DIFF_ARGS+=(--profile "$PROFILE")
[[ -f "$ORACLE_PROFILE" ]] && DIFF_ARGS+=(--oracle-profile "$ORACLE_PROFILE")
[[ $PIN -eq 1 ]] && DIFF_ARGS+=(--pin-profile)
[[ $GATE -eq 1 ]] && DIFF_ARGS+=(--gate)
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
cp "$PROFILE" "$EVIDENCE/reference-profile.used.json" 2>/dev/null || true
cp "$ORACLE_PROFILE" "$EVIDENCE/oracle-profile.used.json" 2>/dev/null || true
echo "report: $EVIDENCE/report.md (diff exit $rc)"
exit $rc
