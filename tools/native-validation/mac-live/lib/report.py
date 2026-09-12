#!/usr/bin/env python3
"""Assembles reports/<UTC>.md from a run directory and asserts thresholds.json.

Inputs (all written by run.sh into the run directory): env.json, helpers.json,
app.json, bundle.json, steps.jsonl, commands.log, typing-bench/typing-bench-*/
*.json, launch-check.json (+ launch-check.md), capture-cycle.json. Missing
inputs are reported as failed gates, never silently skipped. Exit 1 on any
failed gate. Stdlib only.
"""
import argparse
import glob
import json
import os
import sys


def load(path, default=None):
    try:
        return json.load(open(path, encoding="utf-8"))
    except Exception:
        return default


def ms(v):
    if v is None:
        return "—"
    return "%.0f" % v if v >= 10 else "%.1f" % v


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--run-dir", required=True)
    ap.add_argument("--thresholds", required=True)
    ap.add_argument("--out", required=True)
    a = ap.parse_args()
    rd = a.run_dir
    th = json.load(open(a.thresholds))
    env = load(os.path.join(rd, "env.json"), {})
    helpers = load(os.path.join(rd, "helpers.json"), {})
    app = load(os.path.join(rd, "app.json"), {})
    bundle = load(os.path.join(rd, "bundle.json"), {})
    launch = load(os.path.join(rd, "launch-check.json"))
    cycle = load(os.path.join(rd, "capture-cycle.json"))
    steps = []
    try:
        steps = [json.loads(l) for l in open(os.path.join(rd, "steps.jsonl")) if l.strip()]
    except Exception:
        pass
    commands = ""
    try:
        commands = open(os.path.join(rd, "commands.log"), encoding="utf-8").read().strip()
    except Exception:
        pass
    utc = env.get("utc", os.path.basename(rd))
    gates = []  # (section, name, ok, detail)

    def gate(section, name, ok, detail=""):
        gates.append((section, name, bool(ok), str(detail)))
        return bool(ok)

    # ---------------------------------------------------------------- builds
    sec = "build"
    gate(sec, "helpers built from %s (pinned clone HEAD %s, %s dirty entries)" % (helpers.get("ref"), (helpers.get("scratch_head") or "?")[:7], helpers.get("scratch_dirty_entries")), helpers.get("built_ok") and all(b.get("sha256") for b in helpers.get("binaries", {}).values()),
         ", ".join("%s=%s" % (k, (v.get("sha256") or "missing")[:12]) for k, v in sorted(helpers.get("binaries", {}).items())))
    gate(sec, "app built (release) from %s" % app.get("branch"), app.get("built_ok"), (app.get("sha256") or "missing")[:12])
    bth = th.get("bundle", {}).get("gates", {})
    bbins = bundle.get("binaries", {})
    gate(sec, "FlashTeX.app packaged with %s" % ", ".join(bth.get("required_binaries", [])), bundle.get("built_ok") and all(n in bbins for n in bth.get("required_binaries", [])),
         ", ".join(sorted(bbins)))
    if bth.get("bundled_helper_sha256_equals_built_helper_sha256"):
        mism = []
        for n, b in helpers.get("binaries", {}).items():
            if n not in bth.get("required_binaries", []):
                continue
            if not b.get("sha256_content") or bbins.get(n, {}).get("sha256_content") != b.get("sha256_content"):
                mism.append(n)
        gate(sec, "bundled helpers carry the freshly built object code (signature-masked Mach-O content sha256 equal, all four)",
             bundle.get("built_ok") and not mism, "mismatch: %s" % mism if mism else "identical (make-app.sh re-signs the bundle ad hoc, so only the code-signature blob differs)")
        gate(sec, "bundled FlashTeX carries the freshly built FlashTeXMac object code (signature-masked content sha256 equal)",
             bundle.get("built_ok") and app.get("sha256_content") and bbins.get("FlashTeX", {}).get("sha256_content") == app.get("sha256_content"),
             "%s vs %s" % ((app.get("sha256_content") or "?")[:12], (bbins.get("FlashTeX", {}).get("sha256_content") or "?")[:12]))

    comp = bundle.get("components_json") or {}
    if comp:
        main_short = (env.get("sources", {}).get("main_sha") or "")[:7]
        branch_short = (env.get("sources", {}).get("branch_sha") or "")[:7]
        bad = []
        for key in ("compiler", "pdf", "bridge", "edit_ledger"):
            if (comp.get(key) or {}).get("git_sha") != main_short:
                bad.append("%s=%s" % (key, (comp.get(key) or {}).get("git_sha")))
        if (comp.get("app") or {}).get("git_sha") != branch_short:
            bad.append("app=%s" % (comp.get("app") or {}).get("git_sha"))
        gate(sec, "bundle components.json git SHAs: helpers == %s (main), app == %s (branch)" % (main_short, branch_short), not bad, "; ".join(bad) or "all match")

    # ---------------------------------------------------------- typing bench
    tb = th.get("typing_bench", {})
    tg = tb.get("gates", {})
    producers = [("compiler", "typing-bench"), ("controller", "typing-bench-controller")]
    runs = {}  # producer -> cell -> summary (first attempt wins; a retry pass fills cells the first attempt lost)
    retried = {}  # producer -> [cells taken from the retry pass]
    for prod, sub in producers:
        runs[prod] = {}
        retried[prod] = []
        for attempt, pattern in (("first", os.path.join(rd, sub, "typing-bench-*", "*.json")), ("retry", os.path.join(rd, sub, "retry", "typing-bench-*", "*.json"))):
            for f in sorted(glob.glob(pattern)):
                d = load(f)
                if not d:
                    continue
                base = os.path.basename(f)[:-5]  # compiler-demo-30ms (run.sh names the file by its producer flag)
                parts = base.split("-", 1)
                cell = parts[1] if len(parts) > 1 else base
                if cell in runs[prod]:
                    continue
                d["_attempt"] = attempt
                runs[prod][cell] = d
                if attempt == "retry":
                    retried[prod].append(cell)
    controller_present = bool(runs["controller"]) or os.path.isdir(os.path.join(rd, "typing-bench-controller"))
    for prod, sub in producers:
        sec = "typing-bench/" + prod
        pg = tg.get("producers", {}).get(prod, {})
        if prod == "controller" and not controller_present:
            continue
        for cell in tg.get("required_cells", []):
            d = runs[prod].get(cell)
            if d is None:
                gate(sec, "%s: summary present" % cell, False, "no JSON summary (run failed or timed out)")
                continue
            ev = tg.get("every_cell", {})
            k = d.get("keystroke_to_paint_ms", {})
            gate(sec, "%s: unpainted keystrokes <= %d" % (cell, ev.get("unpainted_max", 0)), d.get("unpainted", 1) <= ev.get("unpainted_max", 0), "unpainted=%s" % d.get("unpainted"))
            if ev.get("typing_budget_exhausted") is False:
                gate(sec, "%s: typing budget not exhausted" % cell, not d.get("typing_budget_exhausted"), "typed %s of %s" % (d.get("typed"), d.get("script_keystrokes")))
            if ev.get("typed_equals_script_keystrokes"):
                gate(sec, "%s: every script keystroke typed" % cell, d.get("typed") == d.get("script_keystrokes") and d.get("keystrokes") == d.get("script_keystrokes"),
                     "typed=%s keystrokes=%s script=%s" % (d.get("typed"), d.get("keystrokes"), d.get("script_keystrokes")))
            gate(sec, "%s: paints >= %d" % (cell, ev.get("paints_min", 1)), d.get("paints", 0) >= ev.get("paints_min", 1), "paints=%s%s" % (d.get("paints"), " (from the retry pass: the first attempt's app died mid-run)" if d.get("_attempt") == "retry" else ""))
            expected_producer = pg.get("expected_producer")
            if expected_producer:
                gate(sec, "%s: producer reported by the app == %s" % (cell, expected_producer), d.get("producer") == expected_producer, d.get("producer"))
            seed = cell.rsplit("-", 1)[0]
            lim = pg.get("per_seed_p50_ms_max", {}).get(seed)
            if lim is not None:
                gate(sec, "%s: keystroke->paint p50 <= %s ms" % (cell, lim), k.get("p50_ms") is not None and k["p50_ms"] <= lim, "p50=%s ms; load average at start %s" % (ms(k.get("p50_ms")), env.get("machine", {}).get("load_average_at_start")))
    targets = tb.get("targets", {})

    # ---------------------------------------------------------- launch check
    lg = th.get("launch_check", {}).get("gates", {})
    sec = "launch-check"
    if launch is None:
        gate(sec, "launch-check ran", False, "no launch-check.json (skipped or bundle missing)")
    elif launch.get("refused"):
        gate(sec, "launch-check ran", False, launch.get("reason"))
    else:
        gate(sec, "FAIL lines <= %d" % lg.get("fail_lines_max", 0), len(launch.get("fail_lines", [])) <= lg.get("fail_lines_max", 0), "; ".join(launch.get("fail_lines", [])) or "none")
        notes = launch.get("notes", [])
        for req in lg.get("required_notes", []):
            gate(sec, "note: %s" % req, any(req in n for n in notes), next((n for n in notes if req in n), "absent"))
        if lg.get("no_activate_env_required"):
            gate(sec, "launched with FLASHTEX_NO_ACTIVATE=1 (never activates the window)", launch.get("launched_with_no_activate"), "; ".join(launch.get("open_calls", []))[:300])

    # --------------------------------------------------------- capture cycle
    cg = th.get("capture_cycle", {}).get("gates", {})
    sec = "capture-cycle"
    if cycle is None:
        gate(sec, "capture cycle ran", False, "no capture-cycle.json")
    else:
        for c in cycle.get("checks", []):
            gate(sec, c["name"], c["ok"], c.get("detail", "")[:200])
        if cg.get("no_network_provider_call"):
            gate(sec, "no provider enablement (bridge without --enable-grok, no XAI_API_KEY in env)",
                 cycle.get("network", {}).get("bridge_enable_grok") is False and cycle.get("network", {}).get("xai_api_key_in_env") is False, cycle.get("network"))
        conv = (cycle.get("convert_error") or {}).get("code")
        gate(sec, "capture_convert error code == %s" % cg.get("bridge_convert_error_code"), conv == cg.get("bridge_convert_error_code"), conv)
        comp = cycle.get("compile") or {}
        gate(sec, "compiler status in %s" % cg.get("compiler_status_in"), comp.get("status") in cg.get("compiler_status_in", []), comp.get("status"))
        pdf = cycle.get("pdf") or {}
        gate(sec, "pdf page objects == compiled pages and >= %d" % cg.get("compiled_pages_min", 1),
             pdf.get("page_objects") == comp.get("pages") and (comp.get("pages") or 0) >= cg.get("compiled_pages_min", 1), "pdf=%s compiled=%s" % (pdf.get("page_objects"), comp.get("pages")))

    failed = [g for g in gates if not g[2]]
    verdict = "PASS" if not failed else "FAIL"

    # ---------------------------------------------------------------- render
    L = []
    src = env.get("sources", {})
    mach = env.get("machine", {})
    L.append("# Mac live acceptance: typing-to-paint, helper crash/restart, packaged capture cycle (%s)" % utc)
    L.append("")
    L.append("**Verdict: %s** — %d gates passed, %d failed (thresholds: `tools/native-validation/mac-live/thresholds.json`)." % (verdict, len(gates) - len(failed), len(failed)))
    L.append("")
    L.append("Runner: `tools/native-validation/mac-live/run.sh`; raw artifacts in `reports/%s/` (JSON summaries, launch-check evidence, capture-cycle transcript, logs)." % os.path.basename(rd))
    L.append("")
    L.append("## Provenance")
    L.append("")
    L.append("| item | value |")
    L.append("|---|---|")
    L.append("| driving agent | `%s` |" % env.get("agent"))
    L.append("| agent session | `%s` |" % env.get("session"))
    L.append("| run started (UTC) | %s |" % utc)
    L.append("| user / host | `%s` / `%s` |" % (env.get("user"), env.get("host")))
    L.append("| helpers source | `%s` = `%s` (%s, committed %s) |" % (src.get("main_ref"), src.get("main_sha"), src.get("main_subject"), src.get("main_commit_utc")))
    L.append("| app source | `%s` = `%s` (%s, committed %s) |" % (src.get("branch"), src.get("branch_sha"), src.get("branch_subject"), src.get("branch_commit_utc")))
    L.append("| branch contains main | %s (merge-base `%s`) |" % (src.get("branch_contains_main"), src.get("merge_base")))
    rn = env.get("runner", {})
    L.append("| runner checkout | `%s` @ `%s` (%s dirty tracked files) |" % (rn.get("checkout_branch"), rn.get("checkout_sha"), rn.get("checkout_dirty_tracked_files")))
    for f, h in sorted(rn.get("files", {}).items()):
        L.append("| runner file `%s` | sha256 `%s` |" % (f, h))
    L.append("| machine | %s, %s cores, %s GiB; macOS %s (%s); kernel %s |" % (mach.get("hardware"), mach.get("cores"), (int(mach.get("memory_bytes", 0) or 0) // (1 << 30)), mach.get("os"), mach.get("os_build"), mach.get("kernel")))
    L.append("| toolchain | %s; %s; %s; %s; python %s |" % (mach.get("xcode"), mach.get("swift"), mach.get("cargo"), mach.get("rustc"), mach.get("python3")))
    end = load(os.path.join(rd, "end.json"), {})
    L.append("| load average at start / end | %s / %s (1, 5, 15 min; other agents build and test on this machine concurrently) |" % (mach.get("load_average_at_start"), end.get("load_average_at_end", "—")))
    L.append("| run finished (UTC) | %s |" % end.get("utc_end", "—"))
    L.append("| other FlashTeX/FlashTeXMac processes at start | %s |" % (mach.get("other_flashtex_processes_at_start") or "none"))
    L.append("")
    L.append("### Binaries")
    L.append("")
    L.append("| binary | built from | git SHA | sha256 (as shipped) | sha256 (signature-masked content) | bytes | signature |")
    L.append("|---|---|---|---|---|---:|---|")
    for n, b in sorted(helpers.get("binaries", {}).items()):
        L.append("| `%s` (scratch build) | `%s` | `%s` | `%s` | `%s` | %s | %s |" % (n, b.get("crate"), b.get("git_sha"), b.get("sha256"), b.get("sha256_content"), b.get("bytes"), b.get("signature")))
    L.append("| `FlashTeXMac` (swift build -c release) | `apps/mac` | `%s` | `%s` | `%s` | %s | %s |" % (app.get("sha"), app.get("sha256"), app.get("sha256_content"), app.get("bytes"), app.get("signature")))
    for n, b in sorted(bbins.items()):
        L.append("| `FlashTeX.app/Contents/MacOS/%s` | bundle | — | `%s` | `%s` | %s | %s |" % (n, b.get("sha256"), b.get("sha256_content"), b.get("bytes"), b.get("signature")))
    L.append("")
    L.append("Signature-masked content = sha256 of the thin Mach-O bytes before the `LC_CODE_SIGNATURE` blob with the signature command's offset/size and the `__LINKEDIT` sizes zeroed (`lib/hashes.py`); `codesign --remove-signature` hashes are also in the JSON but are not stable across re-signs. As-shipped sha256 is what a user would hash.")
    L.append("")
    if bundle.get("components_json"):
        L.append("`components.json` written by make-app.sh (its `git_sha` fields come from `git rev-parse` in the pinned scratch clone next to each source binary, so they must equal the short main / branch SHAs above):")
        L.append("")
        L.append("```json")
        L.append(json.dumps(bundle["components_json"], indent=1, sort_keys=True))
        L.append("```")
        L.append("")

    L.append("## Gates")
    L.append("")
    L.append("| section | gate | result | detail |")
    L.append("|---|---|---|---|")
    for s, n, ok, d in gates:
        L.append("| %s | %s | %s | %s |" % (s, n.replace("|", "\\|"), "PASS" if ok else "FAIL", d.replace("|", "\\|").replace("\n", " ")[:220]))
    L.append("")

    L.append("## Typing bench: keystroke -> paint")
    L.append("")
    L.append("Seeds `fixture` / `demo` / `body60k`, 200 typed characters each, at 30 ms (fast typist) and 0 ms (one keystroke per run-loop turn). Producer `compiler` = the app's direct worker route with the scratch-built `flashtex-compiler`; producer `controller` = the durable helper route (`FLASHTEX_PREVIEW_CONTROLLER`, scratch-built `flashtex-preview-controller`, which owns the ledger and launches the same compiler). Definitions and limitations: `reports/%s/typing-bench*/typing-bench.md` (written by `tools/typing-bench/run.sh`)." % os.path.basename(rd))
    L.append("")
    L.append("| producer | cell | bytes | typed | paints | coalesced | unpainted | k->p p50 | p95 | p99 | max | compile p50 | compile p95 | render p50 | render p95 | gate p50 <= | project target p50 <= %s / p95 <= %s |" % (targets.get("project_typing_to_visible_p50_ms"), targets.get("project_typing_to_visible_p95_ms")))
    L.append("|---|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---|")
    for prod, sub in producers:
        if prod == "controller" and not controller_present:
            continue
        pg = tg.get("producers", {}).get(prod, {})
        for cell in tg.get("required_cells", []):
            d = runs[prod].get(cell)
            if not d:
                L.append("| %s | %s | — | — | — | — | — | — | — | — | — | — | — | — | — | — | no summary |" % (prod, cell))
                continue
            k = d.get("keystroke_to_paint_ms", {}); c = d.get("compile_ms", {}); r = d.get("render_pass_ms", {})
            seed = cell.rsplit("-", 1)[0]
            lim = pg.get("per_seed_p50_ms_max", {}).get(seed)
            tp50 = targets.get("project_typing_to_visible_p50_ms"); tp95 = targets.get("project_typing_to_visible_p95_ms")
            tmet = (k.get("p50_ms") is not None and tp50 is not None and k["p50_ms"] <= tp50) and (k.get("p95_ms") is not None and tp95 is not None and k["p95_ms"] <= tp95)
            L.append("| %s | %s | %s | %s/%s | %s | %s | %s | %s | %s | %s | %s | %s | %s | %s | %s | %s | %s |" % (
                d.get("producer", prod), cell + (" (retry)" if d.get("_attempt") == "retry" else ""), d.get("document_bytes_after"), d.get("typed"), d.get("script_keystrokes"), d.get("paints"), d.get("coalesced"), d.get("unpainted"),
                ms(k.get("p50_ms")), ms(k.get("p95_ms")), ms(k.get("p99_ms")), ms(k.get("max_ms")), ms(c.get("p50_ms")), ms(c.get("p95_ms")), ms(r.get("p50_ms")), ms(r.get("p95_ms")),
                lim if lim is not None else "— (not gated)", "met" if tmet else "not met"))
    L.append("")
    for prod, sub in producers:
        if runs[prod]:
            L.append("Elapsed per cell, %s (ms): %s." % (prod, ", ".join("%s=%s" % (cell, ms(runs[prod][cell].get("elapsed_ms"))) for cell in tg.get("required_cells", []) if cell in runs[prod])))
        if retried[prod]:
            L.append("Cells taken from the retry pass for %s (the first pass lost them when the app process disappeared mid-run — see `logs/%s.log`; other agents run kill/launch checks on this machine): %s." % (prod, sub, ", ".join(retried[prod])))
    L.append("")

    L.append("## Launch check: packaged app, child crash, app survival")
    L.append("")
    if launch and not launch.get("refused"):
        L.append("`apps/mac/scripts/launch-check.sh` (unmodified) ran with `open` shimmed to add `--env FLASHTEX_NO_ACTIVATE=1`, a private `FLASHTEX_BRIDGE_STORE` and `FLASHTEX_TRANSCRIPT`; exact `open` call: `%s`." % "; ".join(launch.get("open_calls", [])))
        L.append("")
        L.append("| observation | value |")
        L.append("|---|---|")
        for k in ("pids", "window_confirmed", "compiler_attached", "compiler_first_result", "app_survived_compiler_kill", "worker_exit_logged",
                  "bridge_attached", "app_survived_bridge_kill", "bridge_exit_logged", "quit_cleanly", "compiler_reattached_after_kill", "bridge_reattached_after_kill", "log_counts", "exit_code"):
            L.append("| %s | `%s` |" % (k, json.dumps(launch.get(k))))
        L.append("")
        L.append("Auto-relaunch of a killed child is not implemented in the shell (`ShellModel` sets `worker = nil` and reports `worker exited`); `*_reattached_after_kill` records what happened and is not a gate. The full evidence, including the app's `FLASHTEX_LOG`, is `reports/%s/launch-check.md`." % os.path.basename(rd))
        L.append("")
        L.append("Launch-check notes:")
        L.append("")
        for n in launch.get("notes", []):
            L.append("- %s" % n)
        for f in launch.get("fail_lines", []):
            L.append("- FAIL: %s" % f)
    elif launch:
        L.append("Refused: %s" % launch.get("reason"))
    else:
        L.append("Not run.")
    L.append("")

    L.append("## Packaged capture cycle through the bundled helpers")
    L.append("")
    if cycle:
        L.append("`lib/capture_cycle.py` drove `FlashTeX.app/Contents/MacOS/{flashtex-bridge,flashtex-edit-ledger,flashtex-compiler,flashtex-pdf}` over their real JSON Lines protocols in the shell's order (no UI scripting: Accessibility is not granted and the window is never activated). Bridge launched as `%s` (no `--enable-grok`; `XAI_API_KEY` absent), so `capture_convert` is refused as `provider_disabled` — the expected refusal — and the proposal review uses the offline fixture `fixtures/capture-proposal.json`, explicitly not a provider result. Transcript: `reports/%s/capture-cycle/transcript.jsonl`." % (" ".join(os.path.basename(x) if i == 0 else x for i, x in enumerate(cycle.get("bridge_argv", []))), os.path.basename(rd)))
        L.append("")
        L.append("| check | result | detail |")
        L.append("|---|---|---|")
        for c in cycle.get("checks", []):
            L.append("| %s | %s | %s |" % (c["name"].replace("|", "\\|"), "PASS" if c["ok"] else "FAIL", c.get("detail", "").replace("|", "\\|").replace("\n", " ")[:200]))
        L.append("")
        ci = cycle.get("capture_image", {}); comp = cycle.get("compile", {}); pdf = cycle.get("pdf", {})
        L.append("- Capture image: `%s`, %s decoded bytes, sha256 `%s` (%s)." % (ci.get("mime_type"), ci.get("decoded_bytes"), ci.get("sha256"), ci.get("source")))
        L.append("- Convert refusal: `%s`." % json.dumps(cycle.get("convert_error")))
        L.append("- Bridge SIGKILL: `%s`; restarted pid %s. Ledger SIGKILL: `%s`." % (json.dumps(cycle.get("bridge_sigkill")), cycle.get("bridge_restart_pid"), json.dumps(cycle.get("ledger_sigkill"))))
        L.append("- Prepared edit (offline review): `%s`; ledger receipt `%s`; durable document after: `%s`." % (json.dumps(cycle.get("prepared_edit")), json.dumps(cycle.get("ledger_receipt")), json.dumps(cycle.get("ledger_document_after"))))
        L.append("- `capture_applied`: %s" % cycle.get("bridge_capture_applied"))
        L.append("- Compile: status `%s`, %s page(s), %s item(s), %s diagnostic(s), round trip %s ms; items excerpt: `%s`." % (comp.get("status"), comp.get("pages"), comp.get("items"), comp.get("diagnostic_count"), ms(comp.get("round_trip_ms")), (comp.get("item_text_excerpt") or "").replace("`", "'")))
        if comp.get("diagnostics"):
            L.append("- Diagnostics: `%s`" % json.dumps(comp.get("diagnostics"))[:600])
        L.append("- PDF: `%s`, %s bytes, sha256 `%s`, header `%s`, %s page object(s), writer %s ms; writer output: `%s`." % (os.path.basename(pdf.get("path") or ""), pdf.get("bytes"), pdf.get("sha256"), pdf.get("header"), pdf.get("page_objects"), ms(pdf.get("export_ms")), (pdf.get("writer_output") or "").strip().replace("\n", " / ").replace("`", "'")[:300]))
        tm = cycle.get("timings_ms", {})
        L.append("- Round trips (ms): %s." % ", ".join("%s=%s" % (k, "/".join(ms(x) for x in v)) for k, v in sorted(tm.items())))
    else:
        L.append("Not run.")
    L.append("")

    L.append("## Steps and exact commands")
    L.append("")
    L.append("| step | exit | seconds | log |")
    L.append("|---|---:|---:|---|")
    for s in steps:
        L.append("| %s | %s | %s | `%s` |" % (s.get("step"), s.get("exit"), s.get("seconds"), s.get("log")))
    L.append("")
    L.append("```")
    L.append(commands)
    L.append("```")
    L.append("")
    L.append("## Limitations")
    L.append("")
    L.append("- The typing bench inserts text programmatically (`insertText`), so the OS keyboard/event-queue hop is not measured, and `paint` is the completed CoreAnimation commit rather than display scan-out (see the bench's own methodology and limitations). One run per cell; percentiles are nearest-rank over 200 keystrokes; concurrent load on this machine (other agents' builds/tests) is recorded above but not controlled.")
    L.append("- The launch check exercises child crash and app survival; the shell has no automatic child relaunch, so 'restart' here means the app keeps running and reports the exit — reattachment is a user action.")
    L.append("- The capture cycle drives the packaged app's bundled helpers, not the app's menus/sheets (Accessibility is not granted; the window is never activated). It therefore proves the helpers and the shell's protocol sequence end-to-end, not the SwiftUI review sheet itself; the shell's own review/insertion logic is covered by `swift test` (`RealBridgeTests`, `ShellModelBridgeTests`) on the branch.")
    L.append("- No Grok/provider call is made anywhere; the proposal reviewed is a fixture. A real conversion needs `--enable-grok` plus the Mac credential adapter and is outside this runner by design.")
    L.append("- make-app.sh re-signs the bundle ad hoc, so as-shipped hashes of the bundled binaries differ from the built ones by the signature blob; the signature-masked content hashes are compared instead. The helpers are built in a shared clone pinned at the main SHA and the app in one pinned at the branch SHA, so `components.json` carries the real short SHAs.")
    L.append("")
    open(a.out, "w", encoding="utf-8").write("\n".join(L) + "\n")
    print("%s: %d gates, %d failed -> %s" % (verdict, len(gates), len(failed), a.out))
    for s, n, ok, d in failed:
        print("  FAIL [%s] %s: %s" % (s, n, d[:160]))
    return 0 if not failed else 1


if __name__ == "__main__":
    sys.exit(main())
