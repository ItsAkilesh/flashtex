#!/usr/bin/env python3
"""Write harness/oracle-profile.json: the ESTABLISHED-ENGINE pin (SHA-256 of each live oracle
PDF with engine version, distribution, fonts, preamble, flags and render environment).
Only fresh renders are pinned; reused PDFs are listed under not_pinned.

Usage: pin_oracle.py <reference work dir> <out.json> <run stamp> <fixtures dir>
"""
import hashlib, json, os, sys
work, out, stamp, fixtures = sys.argv[1:5]
engines = json.load(open(os.path.join(work, "engines.json")))
pk = engines.get("_packages", {})
entries, skipped = {}, []
for fx in sorted(os.listdir(work)):
    d = os.path.join(work, fx)
    if not os.path.isdir(d): continue
    for label in sorted(os.listdir(d)):
        ej, pdf = os.path.join(d, label, "engine.json"), os.path.join(d, label, "main.pdf")
        if not (os.path.exists(ej) and os.path.exists(pdf)): continue
        e = json.load(open(ej))
        if e.get("reused") or e.get("exit") != 0:
            skipped.append(f"{fx}/{label}: " + ("reused PDF, not a live render" if e.get("reused") else f"exit {e.get('exit')}")); continue
        base = label.split("-")[0]
        entries[f"{fx}/{label}"] = {
            "fixture_sha256": e.get("fixture_sha256"), "pdf_sha256": hashlib.sha256(open(pdf, "rb").read()).hexdigest(),
            "engine": label, "engine_version": engines.get(base, {}).get("version"), "engine_path": os.path.realpath(e.get("path") or ""),
            "flags": e.get("flags"), "env": e.get("env"), "preamble": e.get("preamble"),
            "fontspec_fonts": e.get("fontspec_fonts"), "fonts_in_log": e.get("fonts_in_log"), "pages": None}
        rj = os.path.join(d, label, "raster.json")
        if os.path.exists(rj):
            try: entries[f"{fx}/{label}"]["pages"] = json.load(open(rj)).get("pages")
            except Exception: pass
prof = {"schema_version": 1, "pinned_utc": stamp, "machine": "mac-m1max-a",
        "meaning": "SHA-256 of the ESTABLISHED ENGINE's PDF per fixture/oracle, rendered live under the pinned environment; the candidate's raw bytes are compared to these unmodified",
        "distribution": pk.get("distribution"), "texlive_root": pk.get("texlive_root"), "texbin_realpath": pk.get("texbin_realpath"), "tlmgr": pk.get("tlmgr"),
        "engines": {k: v for k, v in engines.items() if not k.startswith("_")}, "packages": pk, "system_fonts": engines.get("_system_fonts"),
        "entries": entries, "not_pinned": skipped}
json.dump(prof, open(out, "w"), indent=1, ensure_ascii=False)
print(f"oracle profile pinned: {len(entries)} entries -> {out}" + (f"; not pinned: {len(skipped)}" if skipped else ""))
