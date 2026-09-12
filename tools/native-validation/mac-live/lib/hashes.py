#!/usr/bin/env python3
"""Binary provenance: sha256 as shipped plus sha256 with the Mach-O code
signature removed (`codesign --remove-signature` on a temporary copy), so a
helper copied into FlashTeX.app and re-signed ad hoc by make-app.sh can be
proven byte-identical to the freshly built binary. Stdlib only.

CLI: hashes.py <file>...  -> JSON {path: {sha256, sha256_unsigned, bytes, signature}}
"""
import hashlib
import json
import os
import shutil
import subprocess
import sys
import tempfile


def sha256(path):
    h = hashlib.sha256()
    with open(path, "rb") as f:
        for chunk in iter(lambda: f.read(1 << 20), b""):
            h.update(chunk)
    return h.hexdigest()


def signature(path):
    try:
        p = subprocess.run(["codesign", "-dv", path], capture_output=True, text=True, timeout=30)
        out = p.stdout + p.stderr
        for line in out.splitlines():
            if line.startswith(("Signature=", "Authority=", "TeamIdentifier=")):
                return line.strip()
        return out.strip().splitlines()[-1] if out.strip() else "unknown"
    except Exception as e:
        return "unavailable (%s)" % e


def unsigned_sha256(path):
    d = tempfile.mkdtemp(prefix="flashtex-unsigned.")
    try:
        tmp = os.path.join(d, os.path.basename(path))
        shutil.copy2(path, tmp)
        p = subprocess.run(["codesign", "--remove-signature", tmp], capture_output=True, text=True, timeout=60)
        if p.returncode != 0:
            return None
        return sha256(tmp)
    except Exception:
        return None
    finally:
        shutil.rmtree(d, ignore_errors=True)


def describe(path):
    if not os.path.isfile(path):
        return {"path": path, "sha256": None, "sha256_unsigned": None, "bytes": None, "signature": None}
    return {"path": path, "sha256": sha256(path), "sha256_unsigned": unsigned_sha256(path),
            "bytes": os.path.getsize(path), "signature": signature(path)}


if __name__ == "__main__":
    json.dump({p: describe(p) for p in sys.argv[1:]}, sys.stdout, indent=1, sort_keys=True)
    print()
