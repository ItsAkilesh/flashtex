#!/usr/bin/env python3
"""Measure the reference companion client against the real Mac listener,
cross-process, and write a Markdown evidence file.

The Mac side is `NearbyReferenceClientTests.testServeForExternalClient`
(apps/mac, loopback only) serving a real `NearbyState` with a fresh pairing
code; this script drives the built `nearby-client` binary from outside that
process — Bonjour discovery, pairing, sends, a mid-serve listener outage
(bounded reconnect with re-browse), a revoked pairing, and a vanished Mac.

    python3 apps/mac/tools/nearby-client/scripts/measure-native.py \
        --out docs/evidence/nearby-client-recovery-<UTC>.md

Standard library only. Nothing leaves loopback; no secrets are written (the
pairing code and PSK stay in the temporary store that is deleted at the end).
"""
import argparse
import json
import os
import shutil
import subprocess
import sys
import tempfile
import time
from datetime import datetime, timezone

HERE = os.path.dirname(os.path.abspath(__file__))
PKG = os.path.dirname(HERE)                       # apps/mac/tools/nearby-client
MAC = os.path.dirname(os.path.dirname(PKG))       # apps/mac
REPO = os.path.dirname(os.path.dirname(MAC))      # repo root
PNG_B64 = ("iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAIAAACQd1PeAAAADElEQVR4nGP4//8/AAX+Av4N70a4AAAAAElFTkSuQmCC")


def sh(cmd, cwd=None, timeout=600):
    t0 = time.monotonic()
    p = subprocess.run(cmd, cwd=cwd, capture_output=True, text=True, timeout=timeout)
    return p.returncode, (p.stdout + p.stderr), time.monotonic() - t0


def git(*args):
    return subprocess.run(["git", *args], cwd=REPO, capture_output=True, text=True).stdout.strip()


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--out", required=True, help="Markdown evidence path to write")
    ap.add_argument("--serve-seconds", type=float, default=48)
    ap.add_argument("--restart-at", type=float, default=18)
    ap.add_argument("--down-seconds", type=float, default=3)
    ap.add_argument("--forget-at", type=float, default=34)
    ap.add_argument("--sends", type=int, default=5)
    a = ap.parse_args()

    started_utc = datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")
    sha = git("rev-parse", "HEAD")
    dirty = git("status", "--porcelain", "--", "apps/mac")
    sw = sh(["swift", "--version"])[1].splitlines()[0]
    sw_ver = sh(["sw_vers", "-productVersion"])[1].strip()
    hw = sh(["sysctl", "-n", "machdep.cpu.brand_string"])[1].strip()

    lines = [f"# nearby-client native recovery measurement — {started_utc}", "",
             f"Commit: `{sha}`{' (apps/mac dirty)' if dirty else ''}; macOS {sw_ver}; {hw}; `{sw}`.",
             "Mac side: `NearbyReferenceClientTests.testServeForExternalClient` (real `NearbyState` + `NearbyListener`, loopback only).",
             "Client side: `.build/debug/nearby-client` run as a separate process (this script).",
             "Numbers are wall-clock seconds for whole CLI invocations (process start → exit), one machine, loopback.", ""]

    # 1. Build the client binary and the Mac test bundle.
    rc, out, dt = sh(["swift", "build", "--product", "nearby-client"], cwd=PKG)
    if rc != 0:
        print(out); sys.exit(rc)
    client = os.path.join(PKG, ".build", "debug", "nearby-client")
    lines.append(f"- built `nearby-client` (debug) in {dt:.1f}s")
    rc, out, dt = sh(["swift", "build", "--build-tests"], cwd=MAC, timeout=1800)
    if rc != 0:
        print(out); sys.exit(rc)
    lines.append(f"- built apps/mac tests in {dt:.1f}s")

    tmp = tempfile.mkdtemp(prefix="nearby-native-")
    info_path = os.path.join(tmp, "serve.json")
    store = os.path.join(tmp, "pairs.json")
    png = os.path.join(tmp, "dot.png")
    with open(png, "wb") as f:
        import base64
        f.write(base64.b64decode(PNG_B64))
    env = dict(os.environ, FLASHTEX_NEARBY_SERVE_INFO=info_path, FLASHTEX_NEARBY_SERVE_SECONDS=str(a.serve_seconds),
               FLASHTEX_NEARBY_SERVE_RESTART_AT=str(a.restart_at), FLASHTEX_NEARBY_SERVE_DOWN_SECONDS=str(a.down_seconds),
               FLASHTEX_NEARBY_SERVE_FORGET_AT=str(a.forget_at), FLASHTEX_NEARBY_SERVE_NAME="FlashTeX Native " + str(os.getpid()))
    serve = subprocess.Popen(["swift", "test", "--skip-build", "--filter", "NearbyReferenceClientTests/testServeForExternalClient"],
                             cwd=MAC, env=env, stdout=subprocess.PIPE, stderr=subprocess.STDOUT, text=True)
    t_serve0 = time.monotonic()
    while not os.path.exists(info_path):
        if serve.poll() is not None:
            print(serve.stdout.read()); sys.exit(1)
        time.sleep(0.05)
    time.sleep(0.2)
    with open(info_path) as f:
        info = json.load(f)
    t_info = time.monotonic() - t_serve0  # includes test bundle launch
    fp, name, port0 = info["fp"], info["name"], info["port"]
    lines += ["", f"Serving as `{name}` fp `{fp}` on port {port0} (loopback_only={info.get('loopback_only')}), pinned destination "
              f"`{info['destination'].get('destination_id')}` @ rev {info['destination'].get('base_revision')}; "
              f"serve info available {t_info:.2f}s after launching the test runner.", ""]
    # The serve loop's t=0 is roughly when info was written.
    t_zero = time.monotonic()
    results = []

    def run(label, args, expect):
        t0 = time.monotonic() - t_zero
        rc, out, dt = sh([client] + args + ["--store", store], timeout=120)
        ok = rc == expect
        results.append((label, rc, expect, dt, t0, out))
        lines.append(f"| {label} | {rc} | {expect} | {dt:.3f} | +{t0:.1f} | {'ok' if ok else 'UNEXPECTED'} |")
        return rc, out, dt

    lines += ["| step | exit | expected | seconds | at +t | verdict |", "|---|---|---|---|---|---|"]
    # 2. Pair via Bonjour.
    run("pair via Bonjour (`--mac <fp>`), bootstrap TLS, hello → pair_psk, destination_query", ["pair", "--code", info["code"], "--name", "Native CLI", "--mac", fp, "--seconds", "10", "-v"], 0)
    # 3. Sends via Bonjour and direct.
    for i in range(a.sends):
        run(f"send #{i+1} via Bonjour (browse by fp, reconnect with pair_psk, capture_submit → capture_received)",
            ["send", "--image", png, "--instructions", "native", "--capture-id", f"native-bonjour-{i+1}", "--mac", fp, "--seconds", "10"], 0)
    for i in range(a.sends):
        run(f"send #{i+1} direct `--host 127.0.0.1 --port {port0}` (no browse)",
            ["send", "--image", png, "--capture-id", f"native-direct-{i+1}", "--mac", fp, "--host", "127.0.0.1", "--port", str(port0)], 0)
    run("send: identical retry of native-direct-1 (Mac de-duplicates, acknowledged again)",
        ["send", "--image", png, "--capture-id", "native-direct-1", "--mac", fp, "--host", "127.0.0.1", "--port", str(port0)], 0)
    run("send: same capture_id, different payload → capture_id_conflict (terminal, exit 2)",
        ["send", "--image", png, "--instructions", "changed", "--capture-id", "native-direct-1", "--mac", fp, "--host", "127.0.0.1", "--port", str(port0)], 2)
    # 4. Outage: wait until the Mac is down, then send with a bounded reconnect that re-browses.
    wait = a.restart_at + 0.4 - (time.monotonic() - t_zero)
    if wait > 0:
        time.sleep(wait)
    rc, out, dt = run(f"send during a {a.down_seconds:.0f}s listener outage (port released, new ephemeral port after): "
                      "`--attempts 8 --retry-delay 0.5 --max-delay 2 --seconds 2`, re-browse by fp before every attempt",
                      ["send", "--image", png, "--capture-id", "native-outage-1", "--mac", fp, "--seconds", "2",
                       "--attempts", "8", "--retry-delay", "0.5", "--max-delay", "2"], 0)
    outage_lines = [l for l in out.splitlines() if l.startswith(("attempt", "browsing", "found", "hello_ack", "capture_received", "error"))]
    # 5. Revoked pairing.
    wait = a.forget_at + 0.5 - (time.monotonic() - t_zero)
    if wait > 0:
        time.sleep(wait)
    rc, out_rev, dt = run("send after the Mac forgot the pairing (`--attempts 5`): refused at the TLS handshake, no retry, exit 3",
                          ["send", "--image", png, "--capture-id", "native-revoked-1", "--mac", fp, "--seconds", "5", "--attempts", "5", "--retry-delay", "0.2"], 3)
    revoked_lines = [l for l in out_rev.splitlines() if l.startswith(("attempt", "error", "hint"))]
    # 6. Mac gone.
    serve.wait(timeout=a.serve_seconds + 60)
    serve_out = serve.stdout.read()
    rc, out_gone, dt = run("send with the Mac gone, direct port: `--attempts 3 --retry-delay 0.2` → unreachable ×3, exit 4",
                           ["send", "--image", png, "--capture-id", "native-gone-1", "--mac", fp, "--host", "127.0.0.1", "--port", str(port0),
                            "--attempts", "3", "--retry-delay", "0.2"], 4)
    gone_lines = [l for l in out_gone.splitlines() if l.startswith(("attempt", "error", "hint"))]
    rc, out_gone2, dt = run("send with the Mac gone, Bonjour: `--attempts 2 --seconds 1 --retry-delay 0` → no matching Mac ×2, exit 4",
                            ["send", "--image", png, "--capture-id", "native-gone-2", "--mac", fp, "--seconds", "1", "--attempts", "2", "--retry-delay", "0"], 4)
    gone2_lines = [l for l in out_gone2.splitlines() if l.startswith(("attempt", "error", "hint"))]

    lines += ["", "## Outage recovery transcript (client)", "", "```"] + outage_lines + ["```", ""]
    lines += ["## Revoked pairing transcript (client)", "", "```"] + revoked_lines + ["```", ""]
    lines += ["## Mac gone transcripts (client)", "", "```"] + gone_lines + ["---"] + gone2_lines + ["```", ""]
    log_path = info_path + ".log"
    if os.path.exists(log_path):
        with open(log_path) as f:
            mac_log = [l.rstrip("\n") for l in f]
        lines += ["## Mac-side log (NearbyState, timestamps relative to serve start)", "", "```"] + mac_log + ["```", ""]
    summary = [l for l in serve_out.splitlines() if "Executed" in l or "error" in l.lower()]
    lines += ["## Serve harness result", "", "```"] + summary[-3:] + ["```", ""]
    bad = [r for r in results if r[1] != r[2]]
    lines += ["## Verdict", "", f"{len(results) - len(bad)}/{len(results)} steps exited as expected." +
              ("" if not bad else " Unexpected: " + "; ".join(f"{b[0]} (exit {b[1]})" for b in bad))]
    for b in bad:
        lines += ["", f"### Output of unexpected step: {b[0]}", "", "```", b[5].strip(), "```"]
    os.makedirs(os.path.dirname(os.path.abspath(a.out)), exist_ok=True)
    with open(a.out, "w") as f:
        f.write("\n".join(lines) + "\n")
    shutil.rmtree(tmp, ignore_errors=True)
    print("\n".join(lines))
    sys.exit(1 if bad else 0)


if __name__ == "__main__":
    main()
