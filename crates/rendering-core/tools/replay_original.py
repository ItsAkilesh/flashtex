#!/usr/bin/env python3
"""Pinned established-fixture acceptance; uses existing producer, exporter and Poppler.
Exit 0=fixture acceptance, 2=refused input/configuration, 3=observed mismatch,
4=unknown (missing tool/version/timeout), 1=execution error. Always writes report.
No downloads, installs, source repair, new font parser or rasterizer.
"""
import argparse
import hashlib
import io
import json
import os
from pathlib import Path
import shutil
import subprocess
import tarfile
import tempfile

PRODUCER = "65dbe7da7a182e99322070e2c9763cc3b69a342b"
CONSUMER = "f0e5a7d8b9c5daacf67516a14dc0ee2a1aafaa59"
FIXTURE_MANIFEST_SHA256 = "4cb8dec58408efa560288adfe848f3d7fedf6bb0967a6b2160f633f89df4a412"
ASSETS = {
    "fonts/tfm/public/lm/ec-lmr12.tfm": "299021120f0a29ef61278a2363903bd8defbb8faaade458eb79067342aecb56f",
    "fonts/tfm/public/lm/rm-lmr12.tfm": "9d4e3d8e39a41b93d91f79c1c47d2297efb7b1af220b94860693c08361f227aa",
    "fonts/tfm/public/lm/rm-lmr8.tfm": "80bcbfd844d2310ac1d3bead45aee25e91b1a4a0a60ff1771959b9a1e90ec1a2",
    "fonts/tfm/public/lm/rm-lmr6.tfm": "eb0bfdf8db3ae1409639fac9c88f84923872500d882d9ff8dc37aff445c723fe",
    "doc/fonts/lm/GUST-FONT-LICENSE.TXT": "49ea6cb9257bbee0a3979c48a774cd221550ac1c20c95549efe45fc99cc18050",
}
VERSIONS = {"cargo": "cargo 1.98.1 (797e8a9bc 2026-08-05)", "rustc": "rustc 1.98.1 (48a229cea 2026-09-01)", "pdftotext": "pdftotext version 26.01.0", "pdftoppm": "pdftoppm version 26.01.0"}

class Outcome(Exception):
    def __init__(self, code, status, reason):
        self.code, self.status, self.reason = code, status, reason
        super().__init__(reason)

def sha(data):
    return hashlib.sha256(data).hexdigest()

def pinned(path, expected):
    try:
        if path.stat().st_size > 64 * 1024 * 1024:
            raise Outcome(2, "refused", f"asset too large: {path.name}")
        data = path.read_bytes()
    except OSError as error:
        raise Outcome(2, "refused", f"asset unavailable: {path}: {error}") from error
    if sha(data) != expected:
        raise Outcome(2, "refused", f"asset digest mismatch: {path}")
    return data

def command(args, cwd, env, input_bytes=None, accepted=(0,), timeout=180):
    try:
        r = subprocess.run(args, cwd=cwd, env=env, input=input_bytes, capture_output=True, timeout=timeout)
    except (FileNotFoundError, subprocess.TimeoutExpired) as error:
        raise Outcome(4, "unknown", f"tool unavailable/timeout: {args[0]}: {error}") from error
    if r.returncode not in accepted:
        raise Outcome(1, "error", f"{args[0]} exited {r.returncode}: {(r.stderr.decode(errors='replace')[:1500]+r.stderr.decode(errors='replace')[-1000:])}")
    return r

def run(args, report):
    root = Path(__file__).resolve().parents[3]
    crate = root / "crates/rendering-core"
    env = dict(os.environ)
    # No network access by Cargo; all dependencies must already be available.
    env["CARGO_NET_OFFLINE"] = "true"
    env.pop("CARGO_TARGET_DIR", None)
    env["CARGO_BUILD_JOBS"] = "2"
    env["CARGO_INCREMENTAL"] = "0"
    tools = {}
    for tool, expected in VERSIONS.items():
        binary = shutil.which(tool)
        if binary is None:
            raise Outcome(4, "unknown", f"missing {tool}; install/login is not attempted")
        r = command([binary, "-v" if tool.startswith("pdf") else "--version"], root, env)
        actual = (r.stdout + r.stderr).decode().splitlines()[0]
        report.setdefault("versions", {})[tool] = actual
        if actual != expected:
            raise Outcome(4, "unknown", f"unpinned {tool} version: {actual}")
        tools[tool] = binary
        report.setdefault("tool_sha256", {})[tool] = sha(Path(binary).read_bytes())
    try:
        import PIL
    except ImportError as error:
        raise Outcome(4, "unknown", "Pillow12.1.0 unavailable") from error
    report["versions"]["Pillow"] = PIL.__version__
    if PIL.__version__ != "12.1.0":
        raise Outcome(4, "unknown", "Pillow version mismatch")
    # Reject changed consumer implementation rather than reuse stale acceptance.
    unchanged = command(["git", "diff", "--exit-code", CONSUMER, "--", "crates/rendering-core/src", "crates/rendering-core/examples/pipeline_cff_probe.rs", "crates/rendering-core/examples/pipeline_fonts_probe.rs", "crates/rendering-core/tests/math_reference.rs", "crates/rendering-core/examples/pdf_compare.rs", "crates/rendering-core/Cargo.toml", "crates/font-resources", "crates/font-engine", "crates/pdf", "crates/project-files", "crates/paragraph-layout", "crates/math-layout"], root, env, accepted=(0,1))
    if unchanged.returncode:
        raise Outcome(4,"unknown","consumer/dependency source differs from pinned acceptance baseline")
    manifest = json.loads(pinned(Path(__file__).with_name("replay-fixtures.json"), FIXTURE_MANIFEST_SHA256))
    selected = list(manifest["fixtures"]) if args.fixture == "all" else [args.fixture]
    prepared = {}
    for name in selected:
        spec = manifest["fixtures"][name]
        assets = {key: pinned(crate / value["path"], value["sha256"]) for key, value in spec["files"].items()}
        engine = json.loads(assets["reference-engine.json"])
        if engine["pdf_sha256"] != sha(assets["reference.pdf"]):
            raise Outcome(2, "refused", "reference engine/PDF digest mismatch")
        prepared[name] = (spec, assets)
    metric_bytes = {name: pinned(args.metrics_root/name, digest) for name, digest in ASSETS.items()}
    report.update(producer_commit=PRODUCER, consumer_baseline=CONSUMER,
                  fixture_manifest_sha256=FIXTURE_MANIFEST_SHA256,
                  runner_sha256=sha(Path(__file__).read_bytes()),
                  pdf_backend_commit="20e5277857b2cd37f102fb07acdd164f82bb49db",
                  metric_assets=ASSETS, fixtures={},
                  source_transformation="requests strip fixture preambles; references use recorded LM preambles")
    build_area=root/"crates/rendering-core/target/replays"
    build_area.mkdir(parents=True,exist_ok=True)
    with tempfile.TemporaryDirectory(prefix="original-",dir=build_area) as directory:
        stage = Path(directory)
        archive = command(["git", "archive", PRODUCER, "crates/render-pipeline"], root, env).stdout
        report["producer_archive_sha256"] = sha(archive)
        tarfile.open(fileobj=io.BytesIO(archive)).extractall(stage, filter="data")
        for name, data in metric_bytes.items():
            target = stage/"metrics"/name
            target.parent.mkdir(parents=True, exist_ok=True)
            target.write_bytes(data)
        env["FLASHTEX_TFM_DIRS"] = str(stage/"metrics/fonts/tfm/public/lm")
        env.pop("FLASHTEX_MAX_REPLY_BYTES", None)
        producer_manifest = stage/"crates/render-pipeline/Cargo.toml"
        command([tools["cargo"], "build", "--offline", "--quiet", "--manifest-path", str(producer_manifest), "--bin", "flashtex-render"], root, env, timeout=300)
        producer = stage/"crates/render-pipeline/target/debug/flashtex-render"
        report["producer_binary_sha256"] = sha(producer.read_bytes())
        codes = []
        for name, (spec, assets) in prepared.items():
            result = {"status": "unknown", "assets": {key:sha(data) for key,data in assets.items()},
                      "text_comparison_validity": spec["text_comparison_validity"]}
            report["fixtures"][name] = result
            try:
                measure_fixture(stage/name, root, env.copy(), tools, producer, spec, assets, result)
                codes.append(0)
            except Outcome as error:
                result.update(status=error.status,reason=error.reason)
                codes.append(error.code)
            except Exception as error:
                result.update(status="error",reason=f"{type(error).__name__}: {error}")
                codes.append(1)
        # An execution/unknown/refused outcome cannot be hidden by another mismatch.
        for code, status in [(1,"error"),(4,"unknown"),(2,"refused"),(3,"mismatch")]:
            if code in codes:
                raise Outcome(code,status,"selected fixtures include " + status + "; inspect each independent result")
        report.update(status="accepted",reason="all selected fixtures passed their limited acceptance scopes")

def measure_fixture(stage, root, env, tools, producer, spec, assets, report):
    from PIL import Image, ImageChops
    stage.mkdir()
    fonts = stage/"fonts"
    fonts.mkdir()
    for name in spec["font_files"]:
        (fonts/name).write_bytes(assets[name])
    (stage/"LICENSE").write_bytes(assets["GUST-FONT-LICENSE.TXT"])
    request = stage/"request.jsonl"
    request.write_bytes(assets["request.jsonl"])
    reference = stage/"reference.pdf"
    reference.write_bytes(assets["reference.pdf"])
    env["FLASHTEX_FONT_DIRS"] = str(fonts)
    env["FLASHTEX_LM_DIR"] = str(fonts)
    display = stage/"display.json"
    reply = command([str(producer), "--font-dir", str(fonts), "--secnumdepth", "0", "--v2", str(display)], root, env, assets["request.jsonl"])
    response = json.loads(reply.stdout)
    wire = json.loads(display.read_bytes())
    if response["payload"]["status"] != "ok" or response["payload"]["diagnostics"] or wire["payload"]["diagnostics"]:
        raise Outcome(2, "refused", "producer diagnostics/status prevent reference acceptance")
    if len(wire["payload"]["pages"]) != 1:
        raise Outcome(3,"mismatch","pinned fixture must have exactly one page")
    report["display_sha256"] = sha(display.read_bytes())
    consumer_manifest = root/"crates/rendering-core/Cargo.toml"
    prefix = stage/"original"
    command([tools["cargo"], "run", "--offline", "--quiet", "--manifest-path", str(consumer_manifest), "--example", "pipeline_fonts_probe", "--", str(display), str(request), str(fonts), str(stage/"LICENSE"), str(prefix), "--searchable"], root, env)
    pdf = stage/"original.pdf"
    comparison = stage/"comparison.json"
    command([tools["cargo"], "run", "--offline", "--quiet", "--manifest-path", str(consumer_manifest), "--example", "pdf_compare", "--", str(pdf), str(reference), str(comparison)], root, env, accepted=(0,3,4))
    result = json.loads(comparison.read_bytes())
    report["pdf_comparison"] = result
    if result["truncated"] or result["parsed_operators_equal"] is None:
        raise Outcome(4, "unknown", "PDF comparison unsupported or truncated")
    for label, source in [("original",pdf),("reference",reference)]:
        command([tools["pdftotext"],"-enc","UTF-8",str(source),str(stage/(label+".txt"))],root,env)
        command([tools["pdftoppm"],"-r","144","-singlefile","-png",str(source),str(stage/label)],root,env)
    a = Image.open(stage/"original.png").convert("RGB")
    b = Image.open(stage/"reference.png").convert("RGB")
    if a.size != (1224,1584) or b.size != a.size:
        raise Outcome(3,"mismatch","unexpected fixture page geometry")
    different = sum(value != (0,0,0) for value in ImageChops.difference(a,b).get_flattened_data())
    equal_text = (stage/"original.txt").read_bytes() == (stage/"reference.txt").read_bytes()
    report.update(pdf_sha256=sha(pdf.read_bytes()),text_equal=equal_text,raster={"dpi":144,"width":a.width,"height":a.height,"different_pixels":different,"original_rgb_sha256":sha(a.tobytes()),"reference_rgb_sha256":sha(b.tobytes())}, scope="one selected pinned fixture; no global compatibility claim")
    report["raster"]["equal"] = different == 0
    report["extracted_text"] = {"equal": equal_text,
        "original_sha256": sha((stage/"original.txt").read_bytes()),
        "reference_sha256": sha((stage/"reference.txt").read_bytes())}
    decide_measurement(different, equal_text, report["text_comparison_validity"]["status"])
    report.update(status="accepted",reason="zero diagnostics; exact fixture raster and limited linear text equality")

def decide_measurement(different, equal_text, text_validity):
    if different:
        raise Outcome(3,"mismatch","observed raster differs; text validity is separate")
    if text_validity != "limited_linear_text":
        raise Outcome(4,"unknown","raster matches but reference text oracle is incomplete or unverified")
    if not equal_text:
        raise Outcome(3,"mismatch","observed extracted text differs within limited comparison scope")

def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--metrics-root",type=Path,required=True)
    parser.add_argument("--report",type=Path,required=True)
    parser.add_argument("--fixture", choices=("plain","inline-math","display-math","wrapping","ligatures","all"), default="plain")
    args=parser.parse_args()
    report={"format":"flashtex-original-reference-replay-v2","status":"unknown"}
    code=0
    try:
        run(args,report)
    except Outcome as error:
        code=error.code
        report.update(status=error.status,reason=error.reason)
    except Exception as error:
        code=1
        report.update(status="error",reason=f"{type(error).__name__}: {error}")
    args.report.parent.mkdir(parents=True,exist_ok=True)
    args.report.write_text(json.dumps(report,indent=2)+"\n")
    print(json.dumps({"status":report["status"],"reason":report.get("reason"),"report":str(args.report)}))
    return code
if __name__=="__main__":
    raise SystemExit(main())
