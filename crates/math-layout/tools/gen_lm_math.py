#!/usr/bin/env python3
r"""Generate src/lm_math.rs from `latinmodern-math.otf` (Latin Modern Math).

Development-time data extraction, not a product path: the crate never opens
the font at runtime. It embeds

* the OpenType `MATH` table's `MathConstants` (all 56 values, font units),
* `head.unitsPerEm`, `OS/2.sxHeight` (TeX's x_height), and the advance of
  U+2003 EM SPACE / `unitsPerEm` as the quad,
* the `MathVariants` **vertical** variant lists (glyph id, advance height in
  font units) and vertical assemblies for the large operators and delimiters
  the CM adapter covers, so that `\displaystyle` variant selection through
  `DisplayOperatorMinHeight` can be checked against the TFM `next larger`
  chains (FT-020 rev 4 (c)). The font engine (FT-018) does not parse
  `MathVariants`; this table is the evidence for that request, not a
  substitute for the engine.

No third-party modules: the OTF tables needed are parsed directly (OpenType
specification, `MATH` table chapter).

Usage: python3 tools/gen_lm_math.py /path/to/latinmodern-math.otf > src/lm_math.rs
"""
import hashlib
import struct
import sys

# Field order follows the OpenType specification (MathConstants).
CONSTANT_NAMES = [
    "math_leading", "axis_height", "accent_base_height", "flattened_accent_base_height",
    "subscript_shift_down", "subscript_top_max", "subscript_baseline_drop_min",
    "superscript_shift_up", "superscript_shift_up_cramped", "superscript_bottom_min",
    "superscript_baseline_drop_max", "sub_superscript_gap_min",
    "superscript_bottom_max_with_subscript", "space_after_script", "upper_limit_gap_min",
    "upper_limit_baseline_rise_min", "lower_limit_gap_min", "lower_limit_baseline_drop_min",
    "stack_top_shift_up", "stack_top_display_style_shift_up", "stack_bottom_shift_down",
    "stack_bottom_display_style_shift_down", "stack_gap_min", "stack_display_style_gap_min",
    "stretch_stack_top_shift_up", "stretch_stack_bottom_shift_down",
    "stretch_stack_gap_above_min", "stretch_stack_gap_below_min",
    "fraction_numerator_shift_up", "fraction_numerator_display_style_shift_up",
    "fraction_denominator_shift_down", "fraction_denominator_display_style_shift_down",
    "fraction_numerator_gap_min", "fraction_num_display_style_gap_min",
    "fraction_rule_thickness", "fraction_denominator_gap_min",
    "fraction_denom_display_style_gap_min", "skewed_fraction_horizontal_gap",
    "skewed_fraction_vertical_gap", "overbar_vertical_gap", "overbar_rule_thickness",
    "overbar_extra_ascender", "underbar_vertical_gap", "underbar_rule_thickness",
    "underbar_extra_ascender", "radical_vertical_gap", "radical_display_style_vertical_gap",
    "radical_rule_thickness", "radical_extra_ascender", "radical_kern_before_degree",
    "radical_kern_after_degree", "radical_degree_bottom_raise_percent",
]

# Symbols whose vertical variants are extracted: the CM adapter's large
# operators (cmex text-size slots with a `next larger` display variant) and the
# delimiters plain.tex's \delcode covers, plus the radical sign.
VARIANT_CHARS = [
    0x2211, 0x220F, 0x222B, 0x222E, 0x22C3, 0x22C2, 0x2A01, 0x2A02, 0x2A00, 0x22C1, 0x22C0, 0x2210,
    0x28, 0x29, 0x5B, 0x5D, 0x7B, 0x7D, 0x230A, 0x230B, 0x2308, 0x2309, 0x27E8, 0x27E9,
    0x7C, 0x2016, 0x2F, 0x5C, 0x2191, 0x2193, 0x2195, 0x221A,
]


def u16(b, o):
    return struct.unpack(">H", b[o:o + 2])[0]


def i16(b, o):
    return struct.unpack(">h", b[o:o + 2])[0]


def u32(b, o):
    return struct.unpack(">I", b[o:o + 4])[0]


def tables(b):
    n = u16(b, 4)
    out = {}
    for i in range(n):
        rec = 12 + 16 * i
        tag = b[rec:rec + 4].decode("latin-1")
        off, length = u32(b, rec + 8), u32(b, rec + 12)
        out[tag] = b[off:off + length]
    return out


def cmap_lookup(t):
    """Unicode → glyph id from a format-4 or format-12 subtable."""
    n = u16(t, 2)
    best = None
    for i in range(n):
        pid, eid, off = u16(t, 4 + 8 * i), u16(t, 6 + 8 * i), u32(t, 8 + 8 * i)
        fmt = u16(t, off)
        if (pid, eid) in ((3, 10), (0, 4), (0, 6)) and fmt == 12:
            best = ("12", off)
            break
        if (pid, eid) in ((3, 1), (0, 3)) and fmt == 4 and best is None:
            best = ("4", off)
    if best is None:
        raise SystemExit("no usable cmap subtable")
    m = {}
    fmt, off = best
    if fmt == "12":
        ngroups = u32(t, off + 12)
        for g in range(ngroups):
            s, e, sg = u32(t, off + 16 + 12 * g), u32(t, off + 20 + 12 * g), u32(t, off + 24 + 12 * g)
            for c in range(s, e + 1):
                m[c] = sg + (c - s)
    else:
        segx2 = u16(t, off + 6)
        seg = segx2 // 2
        ends = [u16(t, off + 14 + 2 * i) for i in range(seg)]
        starts = [u16(t, off + 16 + segx2 + 2 * i) for i in range(seg)]
        deltas = [i16(t, off + 16 + 2 * segx2 + 2 * i) for i in range(seg)]
        ro_base = off + 16 + 3 * segx2
        ros = [u16(t, ro_base + 2 * i) for i in range(seg)]
        for i in range(seg):
            for c in range(starts[i], ends[i] + 1):
                if c == 0xFFFF:
                    continue
                if ros[i] == 0:
                    g = (c + deltas[i]) & 0xFFFF
                else:
                    ga = ro_base + 2 * i + ros[i] + 2 * (c - starts[i])
                    g = u16(t, ga)
                    if g:
                        g = (g + deltas[i]) & 0xFFFF
                if g:
                    m[c] = g
    return m


def coverage(t, off):
    fmt = u16(t, off)
    out = []
    if fmt == 1:
        n = u16(t, off + 2)
        out = [u16(t, off + 4 + 2 * i) for i in range(n)]
    elif fmt == 2:
        n = u16(t, off + 2)
        for i in range(n):
            s, e, si = u16(t, off + 4 + 6 * i), u16(t, off + 6 + 6 * i), u16(t, off + 8 + 6 * i)
            out.extend(range(s, e + 1))
    return out


def hmtx_advance(tabs, gid):
    hhea = tabs["hhea"]
    n = u16(hhea, 34)
    h = tabs["hmtx"]
    i = min(gid, n - 1)
    return u16(h, 4 * i)


def math_constants(m):
    c = u16(m, 4)
    out = {"script_percent_scale_down": i16(m, c), "script_script_percent_scale_down": i16(m, c + 2),
           "delimited_sub_formula_min_height": u16(m, c + 4), "display_operator_min_height": u16(m, c + 6)}
    for i, name in enumerate(CONSTANT_NAMES):
        out[name] = i16(m, c + 8 + 4 * i)
    return out


def math_variants(m, want_gids):
    v = u16(m, 8)
    min_overlap = u16(m, v)
    vcov = u16(m, v + 2)
    vcount = u16(m, v + 6)
    cov = coverage(m, v + vcov)
    out = {}
    for i in range(vcount):
        gid = cov[i]
        if gid not in want_gids:
            continue
        co = u16(m, v + 10 + 2 * i)
        if co == 0:
            continue
        base = v + co
        asm_off = u16(m, base)
        nvar = u16(m, base + 2)
        variants = [(u16(m, base + 4 + 4 * k), u16(m, base + 6 + 4 * k)) for k in range(nvar)]
        parts = []
        if asm_off:
            a = base + asm_off
            npart = u16(m, a + 4)  # after italicsCorrection MathValueRecord
            for k in range(npart):
                p = a + 6 + 10 * k
                parts.append((u16(m, p), u16(m, p + 2), u16(m, p + 4), u16(m, p + 6), u16(m, p + 8)))
        out[gid] = (variants, parts)
    return out, min_overlap


def main():
    if len(sys.argv) != 2:
        print(__doc__, file=sys.stderr)
        return 2
    path = sys.argv[1]
    b = open(path, "rb").read()
    sha = hashlib.sha256(b).hexdigest()
    tabs = tables(b)
    upem = u16(tabs["head"], 18)
    os2 = tabs["OS/2"]
    sx_height = i16(os2, 86) if len(os2) >= 88 else 0
    cap_height = i16(os2, 88) if len(os2) >= 90 else 0
    cmap = cmap_lookup(tabs["cmap"])
    quad = hmtx_advance(tabs, cmap[0x2003]) if 0x2003 in cmap else upem
    m = tabs["MATH"]
    consts = math_constants(m)
    want = {cmap[c]: c for c in VARIANT_CHARS if c in cmap}
    variants, min_overlap = math_variants(m, set(want))
    name_rec = None
    # PostScript name from `name` id 6 (platform 3) for the provenance line.
    nm = tabs["name"]
    cnt, so = u16(nm, 2), u16(nm, 4)
    for i in range(cnt):
        r = 6 + 12 * i
        if u16(nm, r + 6) == 6 and u16(nm, r) == 3:
            ln, off = u16(nm, r + 8), u16(nm, r + 10)
            name_rec = nm[so + off:so + off + ln].decode("utf-16-be")
    w = sys.stdout.write
    w("//! Latin Modern Math (`latinmodern-math.otf`) OpenType `MATH` data, generated by\n")
    w("//! `tools/gen_lm_math.py`. Do not edit by hand.\n//!\n")
    w(f"//! Provenance: `{name_rec}`, SHA-256 `{sha}`,\n")
    w(f"//! {len(b)} bytes, unitsPerEm {upem}; the file the FT-018 font engine's README\n")
    w("//! lists (TeX Live 2026 `fonts/opentype/public/lm-math`). Values are font units\n")
    w("//! (percent fields in percent). `MathVariants` vertical variants are `(glyph id,\n")
    w("//! advance height)` pairs, smallest first, and assemblies are\n")
    w("//! `(glyph id, start connector, end connector, full advance, extender flag)`.\n")
    w("#![allow(clippy::unreadable_literal)]\n\n")
    w("use crate::metrics::OpenTypeMathConstants;\n\n")
    w(f"pub const UNITS_PER_EM: u16 = {upem};\n")
    w(f"/// `OS/2.sxHeight`: TeX's x_height (σ5) equivalent.\npub const X_HEIGHT: i16 = {sx_height};\n")
    w(f"/// `OS/2.sCapHeight`.\npub const CAP_HEIGHT: i16 = {cap_height};\n")
    w(f"/// Advance of U+2003 EM SPACE: TeX's quad (σ6) equivalent.\npub const QUAD: u16 = {quad};\n")
    w(f"pub const SCRIPT_PERCENT_SCALE_DOWN: i16 = {consts['script_percent_scale_down']};\n")
    w(f"pub const SCRIPT_SCRIPT_PERCENT_SCALE_DOWN: i16 = {consts['script_script_percent_scale_down']};\n")
    w(f"pub const DISPLAY_OPERATOR_MIN_HEIGHT: u16 = {consts['display_operator_min_height']};\n")
    w(f"pub const MIN_CONNECTOR_OVERLAP: u16 = {min_overlap};\n")
    w("/// Every `MathConstants` value the specification defines, in its order.\n")
    w("pub const ALL_CONSTANTS: &[(&str, i32)] = &[\n")
    for k in ["script_percent_scale_down", "script_script_percent_scale_down",
              "delimited_sub_formula_min_height", "display_operator_min_height"] + CONSTANT_NAMES:
        w(f"    (\"{k}\", {consts[k]}),\n")
    w("];\n\n")
    w("/// The subset `MathParams::from_opentype` consumes.\n")
    w("pub const CONSTANTS: OpenTypeMathConstants = OpenTypeMathConstants {\n")
    w(f"    units_per_em: {upem},\n")
    for k in ["axis_height", "fraction_numerator_display_style_shift_up", "fraction_numerator_shift_up",
              "stack_top_shift_up", "fraction_denominator_display_style_shift_down",
              "fraction_denominator_shift_down", "superscript_shift_up", "superscript_shift_up_cramped",
              "subscript_shift_down", "superscript_baseline_drop_max", "subscript_baseline_drop_min",
              "fraction_rule_thickness", "upper_limit_gap_min", "lower_limit_gap_min",
              "upper_limit_baseline_rise_min", "lower_limit_baseline_drop_min",
              "delimited_sub_formula_min_height"]:
        w(f"    {k}: {consts[k]},\n")
    w("};\n\n")
    w("/// A vertical variant list from `MathVariants` for one symbol.\n")
    w("#[derive(Debug, Clone, Copy, PartialEq, Eq)]\npub struct VerticalVariants {\n")
    w("    pub ch: char,\n    pub base_gid: u16,\n    /// `(glyph id, advance height)`, smallest first.\n")
    w("    pub variants: &'static [(u16, u16)],\n")
    w("    /// `(glyph id, start connector, end connector, full advance, extender)`.\n")
    w("    pub assembly: &'static [(u16, u16, u16, u16, bool)],\n}\n\n")
    w("pub static VERTICAL_VARIANTS: &[VerticalVariants] = &[\n")
    for gid in sorted(variants, key=lambda g: want[g]):
        ch = want[gid]
        vs, parts = variants[gid]
        w(f"    VerticalVariants {{\n        ch: '\\u{{{ch:04X}}}',\n        base_gid: {gid},\n")
        w("        variants: &[" + ", ".join(f"({g}, {a})" for g, a in vs) + "],\n")
        w("        assembly: &[" + ", ".join(f"({g}, {s}, {e}, {f}, {'true' if fl & 1 else 'false'})"
                                          for g, s, e, f, fl in parts) + "],\n    },\n")
    w("];\n")
    return 0


if __name__ == "__main__":
    sys.exit(main())
