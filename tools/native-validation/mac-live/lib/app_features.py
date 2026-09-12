#!/usr/bin/env python3
"""Packaged-app feature checks that need no UI scripting.

render-attach: launches FlashTeX.app's executable directly (never activated:
  FLASHTEX_NO_ACTIVATE=1) with FLASHTEX_AUTOATTACH=1 and
  FLASHTEX_COMPILER=<bundled flashtex-render>, exactly what File > Attach Render
  Pipeline resolves to, and waits for FLASHTEX_LOG to show
  `launched ... (preview face: latin-modern)` and `revision 1: ok`; also checks
  the flashtex-render child under the app pid, then terminates the app.

exact-export: runs the bundled `flashtex-pdf-exact from-v2 <fixture> --out <pdf>
  --font-dir <fonts>` on the checked-in display-list-v2 fixture and reads the
  result back through PDFKit (a probe compiled from lib/pdfkit_probe.swift):
  page count must equal the fixture's, and the extracted text must contain the
  fixture's words.

Stdlib only; hashes via lib/hashes.py.
"""
import argparse
import json
import os
import re
import signal
import subprocess
import sys
import time

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from hashes import describe  # noqa: E402


def wait_for(path, needles, timeout):
    """Returns {needle: seconds-until-seen or None}."""
    seen = {n: None for n in needles}
    t0 = time.monotonic()
    while time.monotonic() - t0 < timeout and any(v is None for v in seen.values()):
        try:
            text = open(path, encoding="utf-8", errors="replace").read()
        except OSError:
            text = ""
        for n in needles:
            if seen[n] is None and n in text:
                seen[n] = round(time.monotonic() - t0, 3)
        time.sleep(0.1)
    return seen


def render_attach(a):
    macos = os.path.join(a.app, "Contents", "MacOS")
    app_bin = os.path.join(macos, "FlashTeX")
    render = os.path.join(macos, "flashtex-render")
    os.makedirs(a.work, exist_ok=True)
    log = os.path.join(a.work, "flashtex.log")
    open(log, "w").close()
    env = dict(os.environ)
    env.update({"FLASHTEX_AUTOATTACH": "1", "FLASHTEX_NO_ACTIVATE": "1", "FLASHTEX_COMPILER": render,
                "FLASHTEX_LOG": log, "FLASHTEX_BRIDGE_STORE": os.path.join(a.work, "captures"),
                "FLASHTEX_TRANSCRIPT": os.path.join(a.work, "transcript.jsonl")})
    env.pop("FLASHTEX_PREVIEW_FACE", None)
    env.pop("FLASHTEX_PREVIEW_CONTROLLER", None)
    rec = {"app": app_bin, "render": describe(render), "env": {k: env[k] for k in ("FLASHTEX_AUTOATTACH", "FLASHTEX_NO_ACTIVATE", "FLASHTEX_COMPILER", "FLASHTEX_LOG")},
           "checks": []}
    if not os.path.isfile(render):
        rec["checks"].append({"name": "bundled flashtex-render present", "ok": False, "detail": render})
        return rec
    rec["checks"].append({"name": "bundled flashtex-render present", "ok": True, "detail": render})
    t0 = time.monotonic()
    p = subprocess.Popen([app_bin], env=env, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL, cwd=macos)
    rec["pid"] = p.pid
    needles = ["(preview face: latin-modern)", "revision 1: ok", "status: attached:"]
    seen = wait_for(log, needles, a.timeout)
    rec["log_seen_after_s"] = seen
    child = subprocess.run(["pgrep", "-P", str(p.pid), "-x", "flashtex-render"], capture_output=True, text=True).stdout.split()
    rec["render_child_pids"] = child
    rec["app_alive_after_wait"] = p.poll() is None
    launched = [l for l in open(log, encoding="utf-8", errors="replace").read().splitlines() if "launched" in l or "revision 1" in l or "attached" in l][:12]
    rec["log_excerpt"] = launched
    rec["checks"].append({"name": "app launched flashtex-render with preview face latin-modern (FLASHTEX_LOG 'preview face: latin-modern')", "ok": seen["(preview face: latin-modern)"] is not None, "detail": "seen after %s s" % seen["(preview face: latin-modern)"]})
    rec["checks"].append({"name": "flashtex-render attached (status line)", "ok": seen["status: attached:"] is not None, "detail": "seen after %s s" % seen["status: attached:"]})
    rec["checks"].append({"name": "first compile through flashtex-render painted (revision 1: ok)", "ok": seen["revision 1: ok"] is not None, "detail": "seen after %s s" % seen["revision 1: ok"]})
    rec["checks"].append({"name": "flashtex-render child process under the app pid", "ok": bool(child), "detail": "pids %s" % child})
    rec["checks"].append({"name": "app still running after the wait (not activated)", "ok": p.poll() is None, "detail": "exit %s" % p.poll()})
    try:
        p.send_signal(signal.SIGTERM)
        p.wait(timeout=10)
    except Exception:
        p.kill()
    rec["app_exit"] = p.returncode
    rec["elapsed_s"] = round(time.monotonic() - t0, 3)
    return rec


def exact_export(a):
    macos = os.path.join(a.app, "Contents", "MacOS")
    tool = os.path.join(macos, "flashtex-pdf-exact")
    os.makedirs(a.work, exist_ok=True)
    out = os.path.join(a.work, "exact-export.pdf")
    rec = {"tool": describe(tool), "fixture": describe(a.fixture), "font_dir": a.font_dir, "checks": []}
    if not os.path.isfile(tool):
        rec["checks"].append({"name": "bundled flashtex-pdf-exact present", "ok": False, "detail": tool})
        return rec
    rec["checks"].append({"name": "bundled flashtex-pdf-exact present", "ok": True, "detail": tool})
    fixture = json.load(open(a.fixture))
    payload = fixture.get("payload", fixture)
    expected_pages = len(payload.get("pages", []))
    argv = [tool, "from-v2", a.fixture, "--out", out, "--font-dir", a.font_dir]
    rec["argv"] = argv
    t0 = time.monotonic()
    p = subprocess.run(argv, capture_output=True, text=True, timeout=120)
    rec["exit"] = p.returncode
    rec["stdout_tail"] = p.stdout[-800:]
    rec["stderr_tail"] = p.stderr[-800:]
    rec["ms"] = round((time.monotonic() - t0) * 1000, 3)
    rec["checks"].append({"name": "flashtex-pdf-exact from-v2 exit 0", "ok": p.returncode == 0, "detail": (p.stderr or p.stdout)[-200:]})
    rec["pdf"] = describe(out) if os.path.isfile(out) else None
    rec["checks"].append({"name": "PDF written", "ok": bool(rec["pdf"] and rec["pdf"]["bytes"]), "detail": out})
    probe = None
    if a.probe and os.path.isfile(out):
        pr = subprocess.run([a.probe, out], capture_output=True, text=True, timeout=60)
        try:
            probe = json.loads(pr.stdout.strip().splitlines()[-1])
        except Exception:
            probe = {"error": (pr.stderr or pr.stdout)[-300:]}
    rec["pdfkit"] = probe
    text = (probe or {}).get("text", "") or ""
    norm = re.sub(r"\s+", " ", text)
    rec["checks"].append({"name": "PDFKit page count == fixture pages (%d)" % expected_pages, "ok": bool(probe) and probe.get("pages") == expected_pages, "detail": (probe or {}).get("pages")})
    for word in a.expect_text:
        rec["checks"].append({"name": "PDFKit text contains %r" % word, "ok": word in norm, "detail": norm[:160]})
    rec["pdfkit_text"] = norm[:1000]
    return rec


def main():
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    sub = ap.add_subparsers(dest="cmd", required=True)
    r = sub.add_parser("render-attach")
    r.add_argument("--app", required=True)
    r.add_argument("--work", required=True)
    r.add_argument("--out", required=True)
    r.add_argument("--timeout", type=float, default=45)
    e = sub.add_parser("exact-export")
    e.add_argument("--app", required=True)
    e.add_argument("--fixture", required=True)
    e.add_argument("--font-dir", required=True)
    e.add_argument("--probe", default="", help="compiled pdfkit_probe binary")
    e.add_argument("--work", required=True)
    e.add_argument("--out", required=True)
    e.add_argument("--expect-text", action="append", default=[])
    a = ap.parse_args()
    rec = render_attach(a) if a.cmd == "render-attach" else exact_export(a)
    rec["passed"] = sum(1 for c in rec["checks"] if c["ok"])
    rec["failed"] = sum(1 for c in rec["checks"] if not c["ok"])
    json.dump(rec, open(a.out, "w"), indent=1, sort_keys=True, default=str)
    for c in rec["checks"]:
        print("    %s %s -- %s" % ("PASS" if c["ok"] else "FAIL", c["name"], str(c["detail"])[:160]))
    print("%s: %d passed, %d failed" % (a.cmd, rec["passed"], rec["failed"]))
    return 0 if rec["failed"] == 0 else 1


if __name__ == "__main__":
    sys.exit(main())
