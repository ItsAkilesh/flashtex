#!/usr/bin/env python3
"""Binary provenance: sha256 as shipped, sha256 after `codesign
--remove-signature` on a temporary copy, and a signature-masked content hash
(bytes before the LC_CODE_SIGNATURE blob with the two header fields a re-sign
rewrites zeroed), so a helper copied into FlashTeX.app and re-signed ad hoc by
make-app.sh can be proven to carry the freshly built object code. The
`--remove-signature` variant alone is not stable: codesign leaves different
__LINKEDIT padding behind depending on the original signature's size. Stdlib only.

CLI: hashes.py <file>...  -> JSON {path: {sha256, sha256_unsigned, sha256_content, bytes, signature}}
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


def content_sha256(path):
    """sha256 of a thin 64-bit Mach-O with its code signature masked out:
    the bytes before the LC_CODE_SIGNATURE blob, with the signature command's
    dataoff/datasize and the __LINKEDIT segment's vmsize/filesize zeroed (the
    only header fields a re-sign rewrites). Two builds of the same object code
    signed differently hash equal; anything else differs. None if not thin Mach-O."""
    import struct
    try:
        data = open(path, "rb").read()
    except OSError:
        return None
    if len(data) < 32 or data[:4] not in (b"\xcf\xfa\xed\xfe", b"\xfe\xed\xfa\xcf"):
        return None
    little = data[:4] == b"\xcf\xfa\xed\xfe"
    e = "<" if little else ">"
    ncmds, sizeofcmds = struct.unpack(e + "II", data[16:24])
    off = 32
    buf = bytearray(data)
    sig_off = None
    for _ in range(ncmds):
        cmd, cmdsize = struct.unpack(e + "II", data[off:off + 8])
        if cmd == 0x1D:  # LC_CODE_SIGNATURE: cmd, cmdsize, dataoff, datasize
            sig_off = struct.unpack(e + "I", data[off + 8:off + 12])[0]
            buf[off + 8:off + 16] = b"\0" * 8
        elif cmd == 0x19 and data[off + 8:off + 18].rstrip(b"\0") == b"__LINKEDIT":  # LC_SEGMENT_64
            buf[off + 32:off + 40] = b"\0" * 8   # vmsize
            buf[off + 48:off + 56] = b"\0" * 8   # filesize
        off += cmdsize
    if sig_off is None:
        return hashlib.sha256(bytes(buf)).hexdigest() + "-unsigned"
    return hashlib.sha256(bytes(buf[:sig_off])).hexdigest()


def describe(path):
    if not os.path.isfile(path):
        return {"path": path, "sha256": None, "sha256_unsigned": None, "sha256_content": None, "bytes": None, "signature": None}
    return {"path": path, "sha256": sha256(path), "sha256_unsigned": unsigned_sha256(path),
            "sha256_content": content_sha256(path), "bytes": os.path.getsize(path), "signature": signature(path)}


if __name__ == "__main__":
    json.dump({p: describe(p) for p in sys.argv[1:]}, sys.stdout, indent=1, sort_keys=True)
    print()
