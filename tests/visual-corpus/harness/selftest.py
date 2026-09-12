#!/usr/bin/env python3
"""Self-test for the visual-corpus harness (stdlib only; no TeX engine, no cargo).

Checks, on synthetic rasters:
  1. registration: a known (dx,dy) translation is recovered by the ink-projection
     correlation and the post-registration error is ~0 while the raw error is not;
  2. regions: text-area/header/footer/display boxes get raw and registered metrics;
  3. PNG provenance: tEXt Description/Software chunks survive a write and read back;
  4. reference reuse: render_reference.sh reuses a stored PDF only when fixture
     SHA-256 and preamble match, and reports "reference unavailable" otherwise
     (exercised through the script with a fake --texbin, so no engine is needed);
  5. overlay footer: when a rasterize binary is given (--rasterize), `annotate`
     grows the image by a footer band and stores the text in the PNG.

Usage: selftest.py [--rasterize <bin>]   exit 0 = all green
"""
import argparse
import hashlib
import json
import os
import shutil
import subprocess
import sys
import tempfile

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
import diff as D  # noqa: E402


def synthetic_page(w=306, h=396, shift=(0, 0), extra=()):
    """White page with 'text lines' (dark bars) and a centred 'display' block."""
    dx, dy = shift
    buf = bytearray(b"\xff" * (w * h))
    bars = [(36 + dx, 50 + dy + i * 14, 36 + 200 + dx, 50 + dy + i * 14 + 6) for i in range(6)]
    bars.append((110 + dx, 150 + dy, 200 + dx, 170 + dy))  # display-ish block
    bars += list(extra)
    for x0, y0, x1, y1 in bars:
        for y in range(max(0, y0), min(h, y1)):
            for x in range(max(0, x0), min(w, x1)):
                buf[y * w + x] = 0
    return D.Gray(w, h, bytes(buf))


def check(cond, msg):
    print(("ok   " if cond else "FAIL ") + msg)
    return bool(cond)


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--rasterize", default=None)
    args = ap.parse_args()
    ok = True
    dpi = 72.0  # 1 px = 1 pt for the synthetic pages

    # 1. registration
    ref = synthetic_page()
    ours = synthetic_page(shift=(7, -5))
    reg = D.registration_diagnostics(ref, ours, dpi)
    ok &= check(reg.get("available") and reg["shift_px"] == [7, -5], f"registration recovers (7,-5): got {reg.get('shift_px')}")
    raw = D.pair_metrics(ref, ours)
    ok &= check(raw["diff_mean"] > 0 and reg["registered_diff_mean"] == 0.0,
                f"raw error {raw['diff_mean']} > 0, post-registration error {reg['registered_diff_mean']} == 0")
    ok &= check(reg["registered_ssim_8x8_mean"] >= raw["ssim_8x8_mean"], "SSIM after registration >= raw SSIM")
    # a genuine rendering difference must survive registration
    ours2 = synthetic_page(shift=(7, -5), extra=[(36 + 7, 250 - 5, 236, 256 - 5)])
    reg2 = D.registration_diagnostics(ref, ours2, dpi)
    ok &= check(reg2["shift_px"] == [7, -5] and reg2["registered_diff_mean"] > 0,
                f"extra line: shift still {reg2['shift_px']}, rendering error {reg2['registered_diff_mean']} > 0 (separated from offset)")

    # 2. regions
    sc = dpi / 72
    regions = [{"name": "text-area", "pt": [36, 36, 270, 360], "px": [36, 36, 270, 360]},
               {"name": "header-band", "pt": [0, 0, 306, 36], "px": [0, 0, 306, 36]},
               {"name": "footer-band", "pt": [0, 360, 306, 396], "px": [0, 360, 306, 396]},
               {"name": "display-1", "pt": [106, 146, 204, 174], "px": [106, 146, 204, 174]}]
    rm = D.region_metrics(ref, ours, reg["shift_px"], regions)
    names = [r["name"] for r in rm]
    ok &= check(names == ["text-area", "header-band", "footer-band", "display-1"], f"region names {names}")
    ta = rm[0]
    ok &= check(ta["raw"]["diff_mean"] > 0 and ta["registered"]["diff_mean"] == 0.0,
                f"text-area raw {ta['raw']['diff_mean']} -> registered {ta['registered']['diff_mean']}")
    ok &= check(rm[1]["raw"]["diff_mean"] == 0.0, "header band is empty on both sides")
    words = [{"page": 1, "x": 110, "right": 200, "top": 150, "bottom": 170},
             {"page": 1, "x": 36, "right": 236, "top": 50, "bottom": 56}]
    boxes = D.display_boxes_from_words(words, 1, dpi, left_pt=36, right_pt=270)
    ok &= check(len(boxes) == 1 and boxes[0]["name"] == "display-1" and boxes[0]["pt"][0] == 106.0,
                f"display box derived from indented words: {boxes}")

    # 3. PNG tEXt provenance
    with tempfile.TemporaryDirectory() as td:
        png = os.path.join(td, "t.png")
        D.write_png(png, ref.w, ref.h, D.overlay_rgb(ref, ours), None, footer="fixture x sha256 abc | run test", rasterize=None)
        t = D.png_text(png)
        ok &= check(t.get("Description") == "fixture x sha256 abc | run test" and "Software" in t, f"tEXt chunks read back: {sorted(t)}")

        # 5. overlay footer via rasterize annotate (optional)
        if args.rasterize:
            out = os.path.join(td, "a.png")
            r = subprocess.run([args.rasterize, "annotate", png, out, "fixture x | oracle y | page 1"], capture_output=True, text=True)
            ok &= check(r.returncode == 0 and os.path.exists(out), f"rasterize annotate exit {r.returncode} {r.stderr.strip()[:80]}")
            if r.returncode == 0:
                info = json.loads(r.stdout)
                ok &= check(info.get("band_px", 0) > 0, f"footer band {info.get('band_px')} px appended")
                ok &= check(b"fixture x | oracle y | page 1" in open(out, "rb").read(), "footer text present in PNG metadata")

        # 4. reference reuse decision through render_reference.sh with no engine
        fx = os.path.join(td, "fixtures"); os.makedirs(fx)
        tex = os.path.join(fx, "99-selftest.tex")
        open(tex, "w").write("\\documentclass{article}\n\\begin{document}\nSelf test.\n\\end{document}\n")
        sha = hashlib.sha256(open(tex, "rb").read()).hexdigest()
        store = os.path.join(td, "ev", "20000101T000000Z", "references"); os.makedirs(os.path.join(store, "99-selftest", "pdflatex"))
        pdf = os.path.join(store, "99-selftest", "pdflatex", "main.pdf")
        open(pdf, "wb").write(b"%PDF-1.4 not a real pdf\n")
        pre = ("\\documentclass[12pt]{article}\n\\usepackage[T1]{fontenc}\n\\usepackage{times}\n\\usepackage[margin=1in]{geometry}\n"
               "\\setlength{\\parindent}{0pt}\n\\setcounter{secnumdepth}{0}\n\\pagestyle{empty}\n")
        ej = {"engine": "pdflatex", "variant": "times", "available": True, "exit": 0, "preamble": pre, "flags": ["-x"],
              "fixture_sha256": sha, "pdf_sha256": hashlib.sha256(open(pdf, "rb").read()).hexdigest(),
              "engine_version": "pdfTeX selftest", "rendered_in_run": "20000101T000000Z", "path": "/nowhere/pdflatex"}
        json.dump(ej, open(os.path.join(store, "99-selftest", "pdflatex", "engine.json"), "w"))
        json.dump({"run": "20000101T000000Z", "engines": {"pdflatex": {"available": True, "version": "pdfTeX selftest"}}, "entries": {}},
                  open(os.path.join(store, "manifest.json"), "w"))
        fake_raster = os.path.join(td, "rasterize"); open(fake_raster, "w").write("#!/bin/sh\necho '{}'\n"); os.chmod(fake_raster, 0o755)
        out = os.path.join(td, "out")
        r = subprocess.run(["bash", os.path.join(HERE, "render_reference.sh"), "--out", out, "--rasterize", fake_raster, "--fixtures", fx,
                            "--texbin", os.path.join(td, "no-texbin"), "--engine", "pdflatex",
                            "--reference-from", os.path.join(td, "ev", "20000101T000000Z")], capture_output=True, text=True)
        ok &= check(r.returncode == 0, f"render_reference.sh without an engine exits 0 ({r.stderr.strip()[-120:]})")
        e_times = json.load(open(os.path.join(out, "99-selftest", "pdflatex", "engine.json")))
        e_lm = json.load(open(os.path.join(out, "99-selftest", "pdflatex-lm", "engine.json")))
        ok &= check(e_times.get("reused") and e_times["reused_from"]["run"] == "20000101T000000Z"
                    and e_times["reused_from"]["engine_version"] == "pdfTeX selftest", "matching store entry is reused with original pins")
        ok &= check(os.path.exists(os.path.join(out, "99-selftest", "pdflatex", "main.pdf")), "reused PDF copied")
        ok &= check(e_lm.get("available") is False and "reference unavailable" in e_lm.get("reason", ""),
                    f"no stored entry -> reference unavailable: {e_lm.get('reason', '')[:70]}")
        eng = json.load(open(os.path.join(out, "engines.json")))
        ok &= check(eng["pdflatex"]["available"] is False and eng["pdflatex"].get("reused_from", {}).get("run") == "20000101T000000Z",
                    "engines.json records the missing engine and its reuse source")
        # fixture changed -> not reused
        open(tex, "a").write("% changed\n")
        shutil.rmtree(out)
        r = subprocess.run(["bash", os.path.join(HERE, "render_reference.sh"), "--out", out, "--rasterize", fake_raster, "--fixtures", fx,
                            "--texbin", os.path.join(td, "no-texbin"), "--engine", "pdflatex",
                            "--reference-from", os.path.join(td, "ev", "20000101T000000Z")], capture_output=True, text=True)
        e_times = json.load(open(os.path.join(out, "99-selftest", "pdflatex", "engine.json")))
        ok &= check(e_times.get("available") is False and "fixture changed" in e_times.get("reason", ""),
                    "changed fixture SHA-256 is NOT reused (reference unavailable, reason names the SHAs)")

    print("selftest:", "PASS" if ok else "FAIL")
    return 0 if ok else 1


if __name__ == "__main__":
    sys.exit(main())
