#!/usr/bin/env python3
"""Stage the pinned rooted Latin Modern metrics into a FlashTeX.app bundle.

GH36: the render pipeline (`flashtex-render`) loads its required 12 pt metrics
only from a `texmf` root shaped `<root>/fonts/tfm/public/lm/*.tfm` plus
`<root>/doc/fonts/lm/GUST-FONT-LICENSE.TXT`, and the 10 pt face needs
`ec-lmr10.tfm` from the same directory. This helper is the packaging-time
gate for those assets. It never downloads anything and never looks at a host
TeX installation: every file comes from the explicit source root (by default
the vendored `apps/mac/Fonts/texmf`) and is refused unless its byte length and
SHA-256 match the Commander's pinned manifest
(`crates/rendering-core/docs/handoffs/native-assets/manifest.json`, itself
SHA-pinned inside `crates/rendering-core/tools/verify_bundle_resources.py`).

Usage:
  bundle-texmf.py check <source-texmf-root>
      Verify the manifest's texmf entries under the source root. Exit 0 when
      every entry is verified; 1 when any is missing/mismatched; 2 on setup
      failure (verifier or manifest missing/mismatched). JSON report on stdout.
  bundle-texmf.py stage <source-texmf-root> <Contents/Resources> <report.json>
      Re-verify the source entries, copy them to `<Resources>/texmf/...`, then
      run the pinned verifier on the whole Resources directory (fonts, metrics,
      license) and write its report to <report.json>. Exit codes as above.
      On success prints one compact JSON object (the `resources` entry for
      components.json: manifest hash, per-resource sha256/bytes) to stdout.

Symlinks and non-regular files are refused by the verifier's descriptor-relative
no-follow walk. Nothing here prints font bytes or reads outside the two roots.
"""

import json
import os
import runpy
import shutil
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
REPO_ROOT = HERE.parents[2]
VERIFIER = REPO_ROOT / "crates/rendering-core/tools/verify_bundle_resources.py"
TEXMF_PREFIX = "texmf/"


def load_verifier():
    if not VERIFIER.is_file():
        raise ValueError(f"pinned bundle resource verifier missing: {VERIFIER}")
    return runpy.run_path(str(VERIFIER))


def texmf_entries(verifier):
    manifest = verifier["pinned_manifest"]()
    entries = verifier["validate_entries"](manifest)
    return [e for e in entries if e["path"].startswith(TEXMF_PREFIX)]


def check_source(verifier, source_root):
    """Verify the texmf entries relative to `source_root` (no `texmf/` prefix)."""
    entries = texmf_entries(verifier)
    if not entries:
        raise ValueError("pinned manifest has no texmf entries")
    root_fd = os.open(source_root, os.O_RDONLY | os.O_DIRECTORY | os.O_NOFOLLOW)
    try:
        results = []
        for entry in entries:
            relative = dict(entry, path=entry["path"][len(TEXMF_PREFIX):])
            result = verifier["check_resource"](root_fd, relative)
            result["bundle_path"] = entry["path"]
            results.append(result)
    finally:
        os.close(root_fd)
    return results


def report(status, source_root, results, extra=None):
    out = {
        "format": "flashtex-bundle-texmf-stage-v1",
        "source_root": str(source_root),
        "status": status,
        "entries": results,
        "host_tex_consulted": False,
        "downloaded": False,
    }
    if extra:
        out.update(extra)
    return out


def cmd_check(source_root):
    verifier = load_verifier()
    results = check_source(verifier, source_root)
    ok = all(r["status"] == "verified" for r in results)
    print(json.dumps(report("verified" if ok else "refused", source_root, results), indent=2))
    return 0 if ok else 1


def cmd_stage(source_root, resources, report_path):
    verifier = load_verifier()
    results = check_source(verifier, source_root)
    if not all(r["status"] == "verified" for r in results):
        print(json.dumps(report("refused", source_root, results), indent=2))
        return 1
    source_root = Path(source_root)
    resources = Path(resources)
    for r in results:
        destination = resources / r["bundle_path"]
        destination.parent.mkdir(parents=True, exist_ok=True)
        # The source was just verified through a no-follow descriptor walk;
        # copy the bytes (not a symlink) and re-verify the copy below.
        shutil.copyfile(source_root / r["path"], destination)
        os.chmod(destination, 0o644)
    full = verifier["verify"](str(resources))
    Path(report_path).write_text(json.dumps(full, indent=2) + "\n")
    if full["status"] != "verified":
        print(json.dumps(report("refused", source_root, results, {"bundle_report": full}), indent=2))
        return 1
    entry = {
        "manifest_sha256": full["manifest_sha256"],
        "status": full["status"],
        "report": "resource-coverage.json",
        "scope": full["scope"],
        "resources": {
            r["path"]: {"sha256": r["actual_sha256"], "bytes": r["actual_bytes"]}
            for r in full["resources"]
        },
    }
    print(json.dumps(entry, sort_keys=True, separators=(",", ":")))
    return 0


def main(argv):
    try:
        if len(argv) == 3 and argv[1] == "check":
            return cmd_check(argv[2])
        if len(argv) == 5 and argv[1] == "stage":
            return cmd_stage(argv[2], argv[3], argv[4])
    except (ValueError, OSError) as error:
        print(json.dumps({"status": "setup_refused", "reason": str(error)}, indent=2))
        return 2
    sys.stderr.write(__doc__)
    return 2


if __name__ == "__main__":
    sys.exit(main(sys.argv))
