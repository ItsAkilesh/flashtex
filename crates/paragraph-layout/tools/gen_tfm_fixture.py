#!/usr/bin/env python3
"""Generate `tests/fixtures/lm_tfm.rs` from Latin Modern OT1 TFM files.

Reads `rm-lmr10.tfm`, `rm-lmbx10.tfm`, `rm-lmri10.tfm`, `rm-lmbx12.tfm`
(Latin Modern 2.004, GUST Font License) with a self-contained TFM parser and
writes a Rust source fixture: per font the design size, `\fontdimen1..7`,
the widths, heights, depths and italic corrections of the OT1 code points that map to ASCII/em dash, the KRN pairs
and the f-ligatures. Values are in 1/1000 em of the design size.

Usage: gen_tfm_fixture.py <dir-with-tfms> [out.rs]

No TeX program is involved: this is a binary-format reader only.
"""
import struct
import sys
import os


def fix(w):
    if w >= 1 << 31:
        w -= 1 << 32
    return w / (1 << 20)


def parse(path):
    b = open(path, "rb").read()
    lf, lh, bc, ec, nw, nh, nd, ni, nl, nk, ne, np_ = struct.unpack(">12H", b[:24])
    words = [struct.unpack(">I", b[24 + 4 * i : 28 + 4 * i])[0] for i in range(lf - 6)]
    o = 0
    header = words[o : o + lh]
    o += lh
    ci = words[o : o + ec - bc + 1]
    o += ec - bc + 1
    W = [fix(x) for x in words[o : o + nw]]
    o += nw
    H = [fix(x) for x in words[o : o + nh]]
    o += nh
    D = [fix(x) for x in words[o : o + nd]]
    o += nd
    I = [fix(x) for x in words[o : o + ni]]
    o += ni
    LK = words[o : o + nl]
    o += nl
    K = [fix(x) for x in words[o : o + nk]]
    o += nk
    o += ne
    P = [fix(x) for x in words[o : o + np_]]
    design = fix(header[1])
    chars = {}
    for c in range(bc, ec + 1):
        w = ci[c - bc]
        wi = w >> 24
        hi = (w >> 20) & 15
        di = (w >> 16) & 15
        ii = (w >> 10) & 63
        tag = (w >> 8) & 3
        rem = w & 255
        if wi == 0:
            continue
        e = {"w": W[wi], "h": H[hi], "d": D[di], "i": I[ii], "kern": {}, "lig": {}}
        if tag == 1:
            j = rem
            first = LK[j]
            if (first >> 24) > 128:
                j = 256 * ((first >> 8) & 255) + (first & 255)
            while True:
                ins = LK[j]
                skip = ins >> 24
                nxt = (ins >> 16) & 255
                op = (ins >> 8) & 255
                r = ins & 255
                if op >= 128:
                    e["kern"][nxt] = K[256 * (op - 128) + r]
                elif op == 0:
                    e["lig"][nxt] = r
                if skip >= 128:
                    break
                j += skip + 1
        chars[c] = e
    return design, P, chars


# OT1 code -> Unicode for the code points this crate's samples use.
def ot1_char(code):
    if code == 124:
        return "—"  # emdash (the `---` ligature target)
    if code == 123:
        return "–"
    if code == 39:
        return "’"
    if code == 96:
        return "‘"
    if 32 <= code <= 126 and code not in (34, 60, 62, 92, 95, 125, 126):
        return chr(code)
    return None


LIG_CODES = {11: "ﬀ", 12: "ﬁ", 13: "ﬂ", 14: "ﬃ", 15: "ﬄ"}


def rust_char(ch):
    return "'\\u{%04X}'" % ord(ch) if ord(ch) > 126 or ch in "'\\" else "'%s'" % ch


def emit(name, design, P, chars, out):
    ident = name.replace("-", "_").upper()
    out.append("/// `%s.tfm`: design size %gpt; `\\fontdimen1..7` in 1/1000 em." % (name, design))
    out.append("pub const %s_PARAMS: [f64; 7] = [%s];" % (ident, ", ".join("%.3f" % (p * 1000) for p in P[:7])))
    widths = []
    kerns = []
    ligs = []
    for code, e in sorted(chars.items()):
        ch = ot1_char(code)
        if ch is None:
            if code in LIG_CODES:
                widths.append((LIG_CODES[code], e["w"], e["h"], e["d"], e["i"]))
            continue
        widths.append((ch, e["w"], e["h"], e["d"], e["i"]))
        for nxt, k in sorted(e["kern"].items()):
            nc = ot1_char(nxt)
            if nc is not None:
                kerns.append((ch, nc, k))
        for nxt, r in sorted(e["lig"].items()):
            nc = ot1_char(nxt)
            if nc is not None and r in LIG_CODES:
                ligs.append((ch, nc, LIG_CODES[r]))
    out.append("/// (char, width, height, depth, italic correction) in 1/1000 em.")
    out.append("pub const %s_CHARS: &[(char, f64, f64, f64, f64)] = &[" % ident)
    for ch, w, h, d, i in widths:
        out.append(
            "    (%s, %.3f, %.3f, %.3f, %.3f),"
            % (rust_char(ch), w * 1000, h * 1000, d * 1000, i * 1000)
        )
    out.append("];")
    out.append("pub const %s_KERNS: &[(char, char, f64)] = &[" % ident)
    for a, b, k in kerns:
        out.append("    (%s, %s, %.3f)," % (rust_char(a), rust_char(b), k * 1000))
    out.append("];")
    out.append("pub const %s_LIGS: &[(char, char, char)] = &[" % ident)
    for a, b, r in ligs:
        out.append("    (%s, %s, %s)," % (rust_char(a), rust_char(b), rust_char(r)))
    out.append("];")


def main():
    d = sys.argv[1]
    dest = sys.argv[2] if len(sys.argv) > 2 else os.path.join(
        os.path.dirname(os.path.abspath(__file__)), "..", "tests", "fixtures", "lm_tfm.rs"
    )
    out = [
        "//! Generated by `tools/gen_tfm_fixture.py` from the Latin Modern 2.004",
        "//! OT1 TFM files (`rm-lmr10`, `rm-lmbx10`, `rm-lmri10`, `rm-lmbx12`; GUST",
        "//! Font License, B. Jackowski and J. M. Nowacki). Widths and kerns in",
        "//! 1/1000 em of the design size; OT1 codes mapped to Unicode.",
        "#![allow(dead_code)]",
        "",
    ]
    for name in ["rm-lmr10", "rm-lmbx10", "rm-lmri10", "rm-lmbx12"]:
        design, P, chars = parse(os.path.join(d, name + ".tfm"))
        emit(name, design, P, chars, out)
        out.append("")
    with open(dest, "w") as f:
        f.write("\n".join(out))
    print("wrote", dest)


if __name__ == "__main__":
    main()
