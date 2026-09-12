#!/usr/bin/env python3
"""Raster and structural comparison of FlashTeX renders against reference-engine
renders of the same declared fixtures, plus the Markdown evidence report.

Inputs are the directories written by render_reference.sh and render_flashtex.sh
(raw RGBA dumps from the shared CoreGraphics rasterizer, PDFKit word boxes,
compile_result rule geometry). Python 3 stdlib is sufficient; Pillow, when
importable, only accelerates the per-pixel arithmetic and PNG encoding — the
numbers are the same either way (documented in the report).

Per page pair:
  - grayscale (Rec.601 luma) absolute-difference histogram (16 bins of 16 levels),
    mean and max |diff|, fraction of pixels differing at all and above a threshold;
  - mean SSIM over non-overlapping 8x8 blocks of the grayscale images
    (K1=0.01, K2=0.03, L=255; block means/variances/covariance, no Gaussian window);
  - overlay PNG: reference ink in magenta, FlashTeX ink in cyan, overlap dark;
  - heatmap PNG: |diff| as white -> red;
  - structural: PDFKit word boxes aligned by normalised text (difflib), per-word
    dx/dy in points, line-start agreement; horizontal rule segments detected
    from pixel rows on both rasters (PDFKit exposes no rule geometry for the
    reference PDFs, so rule presence is compared via ink rows), and FlashTeX's
    own rule rectangles from compile_result / `re f` for cross-checking.

Usage:
  diff.py --reference <dir> --flashtex <dir> --evidence <dir> [--dpi 144]
          [--threshold 32] [--provenance provenance.json] [--thresholds thresholds.json]
          [--regress <previous evidence dir>] [--max-png-bytes 300000]
"""
import argparse
import difflib
import json
import math
import os
import statistics
import struct
import sys
import unicodedata
import zlib
from datetime import datetime, timezone

try:
    from PIL import Image, ImageChops, ImageMath  # optional accelerator only
    HAVE_PIL = True
except Exception:  # pragma: no cover
    HAVE_PIL = False


# --------------------------------------------------------------------------- images

class Gray:
    """8-bit grayscale page: width, height, bytes (row-major)."""

    def __init__(self, w, h, data):
        self.w, self.h, self.data = w, h, data


def load_rgba(prefix):
    meta = json.load(open(prefix + ".rgba.json"))
    raw = open(prefix + ".rgba", "rb").read()
    w, h = meta["width"], meta["height"]
    assert len(raw) == w * h * 4, f"{prefix}: {len(raw)} bytes != {w}x{h}x4"
    return w, h, raw, meta


def to_gray(w, h, raw):
    if HAVE_PIL:
        return Gray(w, h, Image.frombytes("RGBA", (w, h), raw).convert("L").tobytes())
    r, g, b = raw[0::4], raw[1::4], raw[2::4]
    return Gray(w, h, bytes((299 * x + 587 * y + 114 * z + 500) // 1000 for x, y, z in zip(r, g, b)))


def absdiff(a, b):
    if HAVE_PIL:
        ia = Image.frombytes("L", (a.w, a.h), a.data)
        ib = Image.frombytes("L", (b.w, b.h), b.data)
        return Gray(a.w, a.h, ImageChops.difference(ia, ib).tobytes())
    return Gray(a.w, a.h, bytes(abs(x - y) for x, y in zip(a.data, b.data)))


def histogram(g):
    if HAVE_PIL:
        return Image.frombytes("L", (g.w, g.h), g.data).histogram()
    h = [0] * 256
    for v in g.data:
        h[v] += 1
    return h


def block_means(g, n=8):
    """Mean of each n x n block (truncating partial edge blocks) as a list of floats."""
    bw, bh = g.w // n, g.h // n
    if HAVE_PIL:
        im = Image.frombytes("L", (g.w, g.h), g.data).crop((0, 0, bw * n, bh * n)).convert("F")
        return list(im.resize((bw, bh), Image.BOX).getdata()), bw, bh
    out = []
    d = g.data
    for by in range(bh):
        rows = [d[(by * n + k) * g.w:(by * n + k + 1) * g.w] for k in range(n)]
        for bx in range(bw):
            s = 0
            for row in rows:
                s += sum(row[bx * n:(bx + 1) * n])
            out.append(s / (n * n))
    return out, bw, bh


def block_means_product(a, b, n=8):
    """Mean of a*b over each n x n block (a, b grayscale of equal size)."""
    bw, bh = a.w // n, a.h // n
    if HAVE_PIL:
        ia = Image.frombytes("L", (a.w, a.h), a.data).convert("F")
        ib = Image.frombytes("L", (b.w, b.h), b.data).convert("F")
        prod = ImageMath.lambda_eval(lambda args: args["x"] * args["y"], x=ia, y=ib)
        return list(prod.crop((0, 0, bw * n, bh * n)).resize((bw, bh), Image.BOX).getdata())
    out = []
    for by in range(bh):
        ra = [a.data[(by * n + k) * a.w:(by * n + k + 1) * a.w] for k in range(n)]
        rb = [b.data[(by * n + k) * b.w:(by * n + k + 1) * b.w] for k in range(n)]
        for bx in range(bw):
            s = 0
            for xa, xb in zip(ra, rb):
                s += sum(p * q for p, q in zip(xa[bx * n:(bx + 1) * n], xb[bx * n:(bx + 1) * n]))
            out.append(s / (n * n))
    return out


def ssim_blocks(a, b, n=8):
    K1, K2, L = 0.01, 0.03, 255.0
    C1, C2 = (K1 * L) ** 2, (K2 * L) ** 2
    ma, bw, bh = block_means(a, n)
    mb, _, _ = block_means(b, n)
    maa = block_means_product(a, a, n)
    mbb = block_means_product(b, b, n)
    mab = block_means_product(a, b, n)
    vals = []
    worst = []
    for i in range(bw * bh):
        va, vb, cov = maa[i] - ma[i] ** 2, mbb[i] - mb[i] ** 2, mab[i] - ma[i] * mb[i]
        s = ((2 * ma[i] * mb[i] + C1) * (2 * cov + C2)) / ((ma[i] ** 2 + mb[i] ** 2 + C1) * (va + vb + C2))
        vals.append(s)
        if s < 0.5:
            worst.append((s, (i % bw) * n, (i // bw) * n))
    worst.sort()
    return statistics.fmean(vals) if vals else 1.0, len(vals), sum(1 for v in vals if v < 0.9), worst[:10]


def write_png(path, w, h, rgb, max_bytes=None):
    """Write RGB bytes as PNG; if larger than max_bytes, box-downsample by 2 and retry."""
    def encode(w, h, rgb):
        if HAVE_PIL:
            im = Image.frombytes("RGB", (w, h), rgb)
            im.save(path, optimize=True)
        else:
            raw = b"".join(b"\x00" + rgb[y * w * 3:(y + 1) * w * 3] for y in range(h))
            def chunk(t, d):
                c = t + d
                return struct.pack(">I", len(d)) + c + struct.pack(">I", zlib.crc32(c) & 0xffffffff)
            png = b"\x89PNG\r\n\x1a\n" + chunk(b"IHDR", struct.pack(">IIBBBBB", w, h, 8, 2, 0, 0, 0))
            png += chunk(b"IDAT", zlib.compress(raw, 9)) + chunk(b"IEND", b"")
            open(path, "wb").write(png)
        return os.path.getsize(path)
    size = encode(w, h, rgb)
    scale = 1
    while max_bytes and size > max_bytes and w > 200:
        if HAVE_PIL:
            im = Image.frombytes("RGB", (w, h), rgb).resize((w // 2, h // 2), Image.BOX)
            w, h, rgb = w // 2, h // 2, im.tobytes()
        else:
            nw, nh = w // 2, h // 2
            out = bytearray(nw * nh * 3)
            for y in range(nh):
                r0 = rgb[(2 * y) * w * 3:(2 * y + 1) * w * 3]
                r1 = rgb[(2 * y + 1) * w * 3:(2 * y + 2) * w * 3]
                for x in range(nw):
                    for c in range(3):
                        i = (2 * x) * 3 + c
                        out[(y * nw + x) * 3 + c] = (r0[i] + r0[i + 3] + r1[i] + r1[i + 3]) // 4
            w, h, rgb = nw, nh, bytes(out)
        scale *= 2
        size = encode(w, h, rgb)
    return size, scale


def overlay_rgb(ref, ours):
    """Reference ink magenta, FlashTeX ink cyan, overlap dark, paper white."""
    if HAVE_PIL:
        ir = Image.frombytes("L", (ref.w, ref.h), ref.data)
        io = Image.frombytes("L", (ours.w, ours.h), ours.data)
        # ref ink only -> (255, 0, 255) magenta; ours only -> (0, 255, 255) cyan; both -> black.
        return Image.merge("RGB", (io, ir, ImageChops.lighter(ir, io))).tobytes()
    out = bytearray(ref.w * ref.h * 3)
    for i, (r, o) in enumerate(zip(ref.data, ours.data)):
        out[3 * i], out[3 * i + 1], out[3 * i + 2] = o, r, max(r, o)
    return bytes(out)


def heatmap_rgb(diff):
    if HAVE_PIL:
        d = Image.frombytes("L", (diff.w, diff.h), diff.data)
        inv = d.point(lambda v: 255 - v)
        full = Image.new("L", (diff.w, diff.h), 255)
        return Image.merge("RGB", (full, inv, inv)).tobytes()
    out = bytearray(diff.w * diff.h * 3)
    for i, v in enumerate(diff.data):
        out[3 * i], out[3 * i + 1], out[3 * i + 2] = 255, 255 - v, 255 - v
    return bytes(out)


def ink_rows(g, min_run_px, dark=128):
    """Horizontal dark runs: [(y, x0, x1)] with run length >= min_run_px."""
    segs = []
    if HAVE_PIL:
        im = Image.frombytes("L", (g.w, g.h), g.data).point(lambda v: 255 if v < dark else 0)
        data = im.tobytes()
    else:
        data = bytes(255 if v < dark else 0 for v in g.data)
    for y in range(g.h):
        row = data[y * g.w:(y + 1) * g.w]
        x = 0
        while True:
            x0 = row.find(b"\xff", x)
            if x0 < 0:
                break
            x1 = x0
            while x1 < g.w and row[x1] == 255:
                x1 += 1
            if x1 - x0 >= min_run_px:
                segs.append((y, x0, x1))
            x = x1
    # Merge vertically adjacent rows with overlapping x-range into one rule.
    rules = []
    for y, x0, x1 in segs:
        for r in rules:
            if r["y1"] == y - 1 and x0 < r["x1"] and x1 > r["x0"]:
                r["y1"] = y
                r["x0"], r["x1"] = min(r["x0"], x0), max(r["x1"], x1)
                break
        else:
            rules.append({"y0": y, "y1": y, "x0": x0, "x1": x1})
    return rules


# --------------------------------------------------------------------------- words

def normalise_word(w):
    """Same folding as tools/native-validation/oracle_compare.py so ligatures,
    OT1 accent splitting and dotless i compare equal across producers."""
    w = unicodedata.normalize("NFKC", w)
    w = unicodedata.normalize("NFD", w)
    out = []
    for ch in w:
        if unicodedata.combining(ch):
            continue
        if ch in "¨´`ˆ˜¸˚˝ˇ˘˙":
            continue
        if ch == "ı":
            ch = "i"
        out.append(ch)
    return "".join(out)


def words_of(path):
    if not os.path.exists(path):
        return None
    doc = json.load(open(path))
    out = []
    for p in doc["pages"]:
        for w in p["words"]:
            out.append({"page": p["number"], "text": w["text"], "norm": normalise_word(w["text"]),
                        "x": w["x_pt"], "bottom": w["bottom_pt"], "top": w["top_pt"], "right": w["right_pt"]})
    by_page = {}
    for i, w in enumerate(out):
        by_page.setdefault(w["page"], []).append(i)
    for idxs in by_page.values():
        for i in idxs:
            wi = out[i]
            wi["line_start"] = not any(abs(out[j]["bottom"] - wi["bottom"]) <= 0.6 and out[j]["x"] < wi["x"] - 0.01
                                       for j in idxs if j != i)
    return out


def compare_words(ref, ours):
    a, b = [w["norm"] for w in ref], [w["norm"] for w in ours]
    sm = difflib.SequenceMatcher(a=a, b=b, autojunk=False)
    pairs = [(i + k, j + k) for i, j, n in sm.get_matching_blocks() for k in range(n)]
    same = []
    for i, j in pairs:
        o, u = ref[i], ours[j]
        if o["page"] == u["page"]:
            same.append({"word": o["text"], "page": o["page"], "dx": round(u["x"] - o["x"], 2),
                         "dy": round(u["bottom"] - o["bottom"], 2),
                         "ref_ls": o["line_start"], "ours_ls": u["line_start"]})
    res = {"ref_words": len(a), "ours_words": len(b), "aligned": len(pairs), "aligned_same_page": len(same),
           "sequence_equal": a == b, "similarity": round(sm.ratio(), 4),
           "ref_lines": sum(1 for w in ref if w["line_start"]), "ours_lines": sum(1 for w in ours if w["line_start"])}
    if same:
        res["dx_abs_mean"] = round(statistics.fmean(abs(d["dx"]) for d in same), 2)
        res["dx_abs_max"] = round(max(abs(d["dx"]) for d in same), 2)
        res["dy_abs_mean"] = round(statistics.fmean(abs(d["dy"]) for d in same), 2)
        res["dy_abs_max"] = round(max(abs(d["dy"]) for d in same), 2)
        res["line_start_agreement"] = round(sum(1 for d in same if d["ref_ls"] == d["ours_ls"]) / len(same), 4)
        res["largest"] = sorted(same, key=lambda d: d["dx"] ** 2 + d["dy"] ** 2, reverse=True)[:5]
    res["differences"] = [{"op": t, "ref": a[i1:i2][:8], "ours": b[j1:j2][:8]}
                          for t, i1, i2, j1, j2 in sm.get_opcodes() if t != "equal"][:8]
    return res


# --------------------------------------------------------------------------- per pair

def page_prefixes(d, stem):
    out = []
    n = 1
    while os.path.exists(os.path.join(d, f"{stem}-p{n}.rgba")):
        out.append(os.path.join(d, f"{stem}-p{n}"))
        n += 1
    return out


def compare_pages(ref_prefixes, our_prefixes, out_dir, tag, threshold, dpi, max_png, root, images=True):
    os.makedirs(out_dir, exist_ok=True)
    pages = []
    for n in range(max(len(ref_prefixes), len(our_prefixes))):
        if n >= len(ref_prefixes) or n >= len(our_prefixes):
            pages.append({"page": n + 1, "missing": "flashtex" if n >= len(our_prefixes) else "reference"})
            continue
        w1, h1, raw1, m1 = load_rgba(ref_prefixes[n])
        w2, h2, raw2, m2 = load_rgba(our_prefixes[n])
        if (w1, h1) != (w2, h2):
            pages.append({"page": n + 1, "size_mismatch": [[w1, h1], [w2, h2]]})
            continue
        ref, ours = to_gray(w1, h1, raw1), to_gray(w2, h2, raw2)
        diff = absdiff(ref, ours)
        hist = histogram(diff)
        total = w1 * h1
        bins = [sum(hist[i * 16:(i + 1) * 16]) for i in range(16)]
        mean = sum(i * c for i, c in enumerate(hist)) / total
        mx = max(i for i, c in enumerate(hist) if c) if any(hist) else 0
        differing = total - hist[0]
        above = sum(hist[threshold:])
        ssim, nblocks, low_blocks, worst = ssim_blocks(ref, ours)
        ink_ref = sum(histogram(ref)[:128])
        ink_ours = sum(histogram(ours)[:128])
        ov = os.path.join(out_dir, f"{tag}-p{n + 1}-overlay.png")
        hm = os.path.join(out_dir, f"{tag}-p{n + 1}-heatmap.png")
        if images:
            ov_size, ov_scale = write_png(ov, w1, h1, overlay_rgb(ref, ours), max_png)
            hm_size, hm_scale = write_png(hm, w1, h1, heatmap_rgb(diff), max_png)
        else:
            ov = hm = None
            ov_size = hm_size = ov_scale = hm_scale = 0
        min_run = int(round(10 * dpi / 72))  # >= 10pt of continuous ink
        rr, ro = ink_rows(ref, min_run), ink_rows(ours, min_run)
        # Match each FlashTeX rule to the nearest reference rule (centre distance in pt).
        matched = []
        for r in ro:
            best = None
            for q in rr:
                dyc = ((r["y0"] + r["y1"]) - (q["y0"] + q["y1"])) / 2 * 72 / dpi
                dxc = ((r["x0"] + r["x1"]) - (q["x0"] + q["x1"])) / 2 * 72 / dpi
                dist = math.hypot(dxc, dyc)
                if best is None or dist < best["dist_pt"]:
                    best = {"dist_pt": round(dist, 2), "dx_pt": round(dxc, 2), "dy_pt": round(dyc, 2),
                            "ref_len_pt": round((q["x1"] - q["x0"]) * 72 / dpi, 2), "ours_len_pt": round((r["x1"] - r["x0"]) * 72 / dpi, 2),
                            "ref_thick_px": q["y1"] - q["y0"] + 1, "ours_thick_px": r["y1"] - r["y0"] + 1}
            matched.append(best)
        pages.append({
            "page": n + 1, "width_px": w1, "height_px": h1,
            "diff_mean": round(mean, 4), "diff_max": mx,
            "differing_fraction": round(differing / total, 6), "above_threshold_fraction": round(above / total, 6),
            "threshold": threshold, "histogram_16": bins,
            "ssim_8x8_mean": round(ssim, 4), "ssim_blocks": nblocks, "ssim_blocks_below_0.9": low_blocks,
            "ssim_worst_blocks_px": [[round(s, 3), x, y] for s, x, y in worst[:5]],
            "ink_pixels_ref": ink_ref, "ink_pixels_ours": ink_ours,
            "ink_ratio": round(ink_ours / ink_ref, 4) if ink_ref else None,
            "overlay": os.path.relpath(ov, root) if ov else None, "overlay_bytes": ov_size, "overlay_downscale": ov_scale,
            "heatmap": os.path.relpath(hm, root) if hm else None, "heatmap_bytes": hm_size, "heatmap_downscale": hm_scale,
            "rules_ref": len(rr), "rules_ours": len(ro), "rule_matches": matched[:10],
            "rules_ref_px": rr[:10], "rules_ours_px": ro[:10],
        })
    return pages


def summarise(pages):
    ok = [p for p in pages if "diff_mean" in p]
    if not ok:
        return {}
    return {"pages_compared": len(ok), "pages_missing": len(pages) - len(ok),
            "diff_mean": round(statistics.fmean(p["diff_mean"] for p in ok), 4),
            "diff_max": max(p["diff_max"] for p in ok),
            "differing_fraction": round(statistics.fmean(p["differing_fraction"] for p in ok), 6),
            "above_threshold_fraction": round(statistics.fmean(p["above_threshold_fraction"] for p in ok), 6),
            "ssim_8x8_mean": round(statistics.fmean(p["ssim_8x8_mean"] for p in ok), 4),
            "rules_ref": sum(p["rules_ref"] for p in ok), "rules_ours": sum(p["rules_ours"] for p in ok)}


# --------------------------------------------------------------------------- thresholds / regression

# Direction of "better" per metric.
HIGHER_BETTER = {"ssim_8x8_mean", "line_start_agreement", "similarity"}
LOWER_BETTER = {"diff_mean", "differing_fraction", "above_threshold_fraction", "dx_abs_mean", "dy_abs_mean"}


def check_thresholds(entry, thresholds):
    """thresholds: {"default": {...}, "<fixture>": {...}} where each value map is
    metric -> limit (min for HIGHER_BETTER, max for LOWER_BETTER). Applies to the
    export comparison against pdflatex with the newest compiler only unless the
    map has "scope": [side, engine, compiler] entries."""
    if not thresholds:
        return None
    t = dict(thresholds.get("default", {}))
    t.update(thresholds.get(entry["fixture"], {}))
    scope = t.pop("scope", None)
    if scope and [entry["side"], entry["engine"], entry["compiler"]] not in scope:
        return None
    fails = []
    src = dict(entry.get("raster", {}))
    src.update({k: v for k, v in entry.get("words", {}).items() if k in HIGHER_BETTER | LOWER_BETTER})
    for metric, limit in t.items():
        if metric not in src:
            continue
        v = src[metric]
        if metric in HIGHER_BETTER and v < limit:
            fails.append(f"{metric}={v} < {limit}")
        elif metric in LOWER_BETTER and v > limit:
            fails.append(f"{metric}={v} > {limit}")
    return {"checked": sorted(k for k in t if k in src), "failures": fails}


def regress(entries, previous_dir, tolerance):
    prev_path = os.path.join(previous_dir, "metrics.json")
    if not os.path.exists(prev_path):
        return {"error": f"no metrics.json in {previous_dir}"}
    prev = {(e["fixture"], e["engine"], e["compiler"], e["side"]): e for e in json.load(open(prev_path))["entries"]}
    worse = []
    compared = 0
    for e in entries:
        p = prev.get((e["fixture"], e["engine"], e["compiler"], e["side"]))
        if not p or not e.get("raster") or not p.get("raster"):
            continue
        compared += 1
        for metric in ("ssim_8x8_mean", "diff_mean", "above_threshold_fraction"):
            a, b = e["raster"].get(metric), p["raster"].get(metric)
            if a is None or b is None:
                continue
            if metric in HIGHER_BETTER and a < b - tolerance.get(metric, 0):
                worse.append(f"{e['fixture']}/{e['engine']}/{e['compiler']}/{e['side']}: {metric} {b} -> {a}")
            if metric in LOWER_BETTER and a > b + tolerance.get(metric, 0):
                worse.append(f"{e['fixture']}/{e['engine']}/{e['compiler']}/{e['side']}: {metric} {b} -> {a}")
    return {"previous": previous_dir, "compared": compared, "worse": worse}


# --------------------------------------------------------------------------- report

def fmt(v, nd=4):
    if v is None:
        return "-"
    if isinstance(v, float):
        return f"{v:.{nd}f}"
    return str(v)


def build_report(entries, prov, evidence, thresholds_result, regress_result, args):
    L = []
    L.append("# FlashTeX visual corpus: reference-render and raster-diff evidence\n")
    L.append(f"Generated {prov.get('generated_utc')} on {prov.get('machine', 'mac-m1max-a')} by `tests/visual-corpus/harness/run.sh`.\n")
    L.append("**Scope statement.** These are narrow-case measurements over a small declared corpus. "
             "They never claim general pixel perfection, LaTeX compatibility, or parity outside these fixtures, "
             "these engines, this font, this page size, this DPI and these builds. The reference engines are "
             "test oracles only; FlashTeX never invokes them and remains an original Rust implementation.\n")
    L.append("## Provenance\n")
    for k in ("suite_branch", "suite_sha", "input_main_sha", "machine", "os", "swift", "cargo", "python", "pillow"):
        if k in prov:
            L.append(f"- {k}: `{prov[k]}`")
    L.append(f"- DPI: {args.dpi} (every raster: CoreGraphics bitmap, sRGB IEC61966-2.1, 8-bit RGBA, white opaque background, "
             f"MediaBox mapped to width_pt*{args.dpi}/72 px; text antialiased, font smoothing off, subpixel positioning on)")
    L.append(f"- Overlay/heatmap PNGs emitted for engines: {args.images_for_engines} (metrics are computed for every engine)")
    L.append(f"- Pixel threshold for `above_threshold_fraction`: |Δluma| ≥ {args.threshold}/255; SSIM: 8×8 blocks, K1=0.01, K2=0.03")
    L.append(f"- Arithmetic backend: {'Pillow ' + prov.get('pillow', '?') + ' (accelerator; identical integer results to the stdlib path)' if HAVE_PIL else 'pure Python stdlib'}")
    L.append("")
    L.append("### Reference engines (oracle only)\n")
    eng = prov.get("engines", {})
    L.append("| Engine | Available | Version | Body font | Preamble |")
    L.append("|---|---|---|---|---|")
    for e in ("pdflatex", "xelatex", "lualatex"):
        row = eng.get(e, {})
        font = ("URW Nimbus Roman (`times` package, T1 fontenc)" if e == "pdflatex"
                else "Times New Roman (`fontspec`, /System/Library/Fonts/Supplemental/Times New Roman.ttf)")
        pre = prov.get("preambles", {}).get(e, "").replace("\n", " ")
        L.append(f"| {e} | {'yes' if row.get('available') else 'NO'} | {row.get('version', '-')} | {font if row.get('available') else '-'} | `{pre}` |")
    L.append(f"\nEngine flags: `{' '.join(prov.get('engine_flags', []))}`. Page size: US letter 612×792 pt for every producer "
             "(checked per page from the MediaBox). LaTeX package versions: see `provenance.json` → `packages`.\n")
    L.append("### FlashTeX builds under test\n")
    for c in prov.get("compilers", []):
        L.append(f"- compiler `{c['label']}`: `{c['ref']}` @ `{c['sha']}` — {c.get('note', '')}")
    p = prov.get("pdf_writer", {})
    L.append(f"- PDF writer: `{p.get('ref')}` @ `{p.get('sha')}` (`flashtex-pdf --verify{' --embed-font auto' if p.get('embed') else ''}`; "
             f"body font Times-Roman standard-14, Unicode fallback subset of {p.get('embed_font', 'none')})")
    L.append("- export raster: the flashtex-pdf PDF rasterized by the same CoreGraphics rasterizer as the references")
    L.append("- preview-equivalent raster: `rasterize preview` re-implements the Mac app's `PDFExport.render` draw (CoreText "
             "`Times-Roman` at x_pt/baseline_y_pt/font_size_pt, U+2500 runs as 0.5em×0.0857em rules) straight into the bitmap. "
             "It links nothing from apps/mac and is **not** the SwiftUI preview; it is labelled preview-equivalent throughout.")
    if prov.get("native_preview"):
        L.append(f"- native preview capture: {prov['native_preview']}")
    else:
        L.append("- native preview capture: not part of this run (see limitations).")
    L.append("")
    L.append("### Fixtures\n")
    L.append("| Fixture | SHA-256 | Purpose |")
    L.append("|---|---|---|")
    for f in prov.get("fixtures", []):
        L.append(f"| `{f['name']}` | `{f['sha256'][:16]}…` | {f.get('purpose', '')} |")
    L.append("\nFull SHAs and `.meta.json` contents are in `provenance.json`. Only the body after `\\begin{document}` is "
             "shared by every producer; the harness substitutes the engine preamble and strips it for FlashTeX.\n")

    def table(side, title, note):
        rows = [e for e in entries if e["side"] == side]
        if not rows:
            return
        L.append(f"## {title}\n")
        L.append(note + "\n")
        L.append("| Fixture | Engine | Compiler | Pages ref/ours | Status | mean\\|Δ\\| | max | differing | ≥thr | SSIM₈ | words ref/ours/aligned | seq= | mean\\|dx\\| pt | mean\\|dy\\| pt | line-start agree | rules ref/ours | overlay |")
        L.append("|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|")
        for e in rows:
            r, w = e.get("raster") or {}, e.get("words") or {}
            pg = e.get("pages", [])
            first = next((p for p in pg if p.get("overlay")), None)
            link = f"[p1]({first['overlay']})" if first else "-"
            L.append("| {} | {} | {} | {} | {} | {} | {} | {} | {} | {} | {} | {} | {} | {} | {} | {} | {} |".format(
                e["fixture"], e["engine"], e["compiler"], f"{e.get('ref_pages', '-')}/{e.get('our_pages', '-')}",
                e.get("status", "-"), fmt(r.get("diff_mean")), fmt(r.get("diff_max")), fmt(r.get("differing_fraction"), 4),
                fmt(r.get("above_threshold_fraction"), 4), fmt(r.get("ssim_8x8_mean")),
                f"{w.get('ref_words', '-')}/{w.get('ours_words', '-')}/{w.get('aligned', '-')}",
                "yes" if w.get("sequence_equal") else ("no" if w else "-"),
                fmt(w.get("dx_abs_mean"), 2), fmt(w.get("dy_abs_mean"), 2), fmt(w.get("line_start_agreement")),
                f"{r.get('rules_ref', '-')}/{r.get('rules_ours', '-')}", link))
        L.append("")

    table("export", "Export comparison (flashtex-pdf PDF vs reference PDF, both rasterized identically)",
          "This is the PDF-output comparison. Word boxes come from PDFKit on both PDFs; rules are ink rows ≥10pt long.")
    table("preview", "Preview-equivalent comparison (CoreText draw of compile_result vs reference PDF raster)",
          "Weaker than a capture of the real preview: it re-implements the app's draw code path rather than exercising the "
          "SwiftUI Canvas. Word-box metrics are not available for this side (no PDF), so they are omitted.")
    table("native", "Native preview capture comparison (screen capture of the running FlashTeXMac preview vs reference PDF raster)",
          "The actual SwiftUI preview, captured with `screencapture -l <window id>` on the Retina display, page region "
          "detected and resampled to the reference raster size; the resampling scale is recorded in `provenance.json`. "
          "Word-box metrics are unavailable (a screenshot has no text layer).")

    L.append("## Per-fixture details\n")
    for e in entries:
        if e["side"] != "export":
            continue
        L.append(f"### {e['fixture']} — {e['engine']} vs compiler `{e['compiler']}` (export)\n")
        b = e.get("build", {})
        if b.get("diagnostics"):
            L.append(f"- FlashTeX diagnostics ({b.get('diagnostic_count')}): " + "; ".join(
                f"{d['severity']}: {d['message']}" for d in b["diagnostics"][:4]) + (" …" if b.get("diagnostic_count", 0) > 4 else ""))
        if b.get("pdf_re_f_count") is not None:
            L.append(f"- FlashTeX rule rectangles: {b.get('rule_items', 0)} U+2500 items in compile_result, "
                     f"{b['pdf_re_f_count']} `re f` rectangles in the PDF" +
                     (f" (first: {b['pdf_re_f_rects'][0]} pt, PDF bottom-left origin)" if b.get("pdf_re_f_rects") else ""))
        for p in e.get("pages", []):
            if "diff_mean" not in p:
                L.append(f"- page {p['page']}: {p}")
                continue
            L.append(f"- page {p['page']}: |Δ| histogram (16 bins, pixel counts) `{p['histogram_16']}`; ink px ref/ours "
                     f"{p['ink_pixels_ref']}/{p['ink_pixels_ours']} (ratio {p['ink_ratio']}); SSIM blocks <0.9: "
                     f"{p['ssim_blocks_below_0.9']}/{p['ssim_blocks']}"
                     + (f"; [overlay]({p['overlay']}) ({p['overlay_bytes']} B, ÷{p['overlay_downscale']}), "
                        f"[heatmap]({p['heatmap']}) ({p['heatmap_bytes']} B, ÷{p['heatmap_downscale']})" if p.get("overlay") else "; images not emitted for this engine"))
            if p["rule_matches"]:
                L.append("  - rules (FlashTeX → nearest reference ink row, pt): " + "; ".join(
                    f"Δx {m['dx_pt']} Δy {m['dy_pt']} len {m['ours_len_pt']} vs {m['ref_len_pt']}, thickness px {m['ours_thick_px']} vs {m['ref_thick_px']}"
                    for m in p["rule_matches"][:3]))
        w = e.get("words") or {}
        if w.get("largest"):
            L.append("- largest word displacements (pt): " + "; ".join(
                f"`{d['word']}` dx {d['dx']} dy {d['dy']}" for d in w["largest"][:3]))
        if w.get("differences"):
            L.append("- word-sequence differences: " + "; ".join(
                f"{d['op']} ref {d['ref']} ours {d['ours']}" for d in w["differences"][:3]))
        L.append("")

    if thresholds_result is not None:
        L.append("## Per-fixture thresholds\n")
        L.append(f"Thresholds file: `harness/{os.path.basename(args.thresholds)}` (copied here as `thresholds.used.json`). A failure here is an acceptance signal for the narrow case only.\n")
        for e in entries:
            t = e.get("thresholds")
            if t:
                L.append(f"- {e['fixture']}/{e['engine']}/{e['compiler']}/{e['side']}: checked {t['checked']}; "
                         + ("**FAIL** " + "; ".join(t["failures"]) if t["failures"] else "pass"))
        L.append("")
    if regress_result is not None:
        L.append("## Regression check\n")
        if "error" in regress_result:
            L.append(f"- {regress_result['error']}")
        else:
            L.append(f"- previous evidence: `{regress_result['previous']}`; comparable entries: {regress_result['compared']}")
            L.append("- worse: " + ("none" if not regress_result["worse"] else ""))
            for wline in regress_result["worse"]:
                L.append(f"  - {wline}")
        L.append("")
    L.append("## Limitations and honesty notes\n")
    for note in prov.get("limitations", []):
        L.append(f"- {note}")
    L.append("- PDFKit exposes text selections only; it cannot report rule/line geometry from the reference PDFs, "
             "so rule presence and position are compared from ink rows of the rasters (≥10pt contiguous dark run), "
             "and FlashTeX's own rectangle geometry (`re f`) is listed only for cross-checking.")
    L.append("- The pdflatex reference uses the URW Nimbus Roman clone (`times` package), xelatex/lualatex use the "
             "macOS Times New Roman TrueType, and FlashTeX's PDF uses the standard-14 `Times-Roman` name resolved by the "
             "rasterizer's CoreGraphics PDF engine (plus an embedded Times New Roman subset for non-WinAnsi glyphs). "
             "Glyph outlines therefore differ slightly even where positions agree; the metrics include that font-substitution noise.")
    L.append("- FlashTeX has no justification, hyphenation, kerning or ligatures; line breaks and word x positions diverge "
             "progressively along a line. Page-level SSIM over text is dominated by that, not by glyph rendering.")
    L.append("- Missing pages (page-count mismatch) are reported, not silently skipped.")
    return "\n".join(L) + "\n"


# --------------------------------------------------------------------------- main

def main():
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--reference", required=True)
    ap.add_argument("--flashtex", required=True)
    ap.add_argument("--evidence", required=True)
    ap.add_argument("--dpi", type=float, default=144)
    ap.add_argument("--threshold", type=int, default=32)
    ap.add_argument("--provenance", default=None)
    ap.add_argument("--thresholds", default=None, help="JSON of per-fixture metric limits")
    ap.add_argument("--regress", default=None, help="previous evidence dir; exit 3 if any metric worsened")
    ap.add_argument("--regress-tolerance", type=float, default=0.002)
    ap.add_argument("--max-png-bytes", type=int, default=300000)
    ap.add_argument("--native", default=None, help="dir with <fixture>/<compiler>/native-p1.rgba captures")
    ap.add_argument("--images-for-engines", default="pdflatex",
                    help="comma-separated engines whose overlay/heatmap PNGs are written (metrics are computed for all); 'all' for every engine")
    args = ap.parse_args()

    os.makedirs(args.evidence, exist_ok=True)
    prov = json.load(open(args.provenance)) if args.provenance else {}
    thresholds = json.load(open(args.thresholds)) if args.thresholds else None
    fixtures = sorted(d for d in os.listdir(args.reference) if os.path.isdir(os.path.join(args.reference, d)))
    entries = []
    for fx in fixtures:
        engines = sorted(d for d in os.listdir(os.path.join(args.reference, fx)))
        compilers = sorted(d for d in os.listdir(os.path.join(args.flashtex, fx))) if os.path.isdir(os.path.join(args.flashtex, fx)) else []
        for eng in engines:
            rdir = os.path.join(args.reference, fx, eng)
            einfo = json.load(open(os.path.join(rdir, "engine.json"))) if os.path.exists(os.path.join(rdir, "engine.json")) else {}
            ref_pages = page_prefixes(rdir, "page")
            ref_words = words_of(os.path.join(rdir, "words.json"))
            for comp in compilers:
                fdir = os.path.join(args.flashtex, fx, comp)
                build = json.load(open(os.path.join(fdir, "build.json"))) if os.path.exists(os.path.join(fdir, "build.json")) else {}
                sides = [("export", page_prefixes(fdir, "export")), ("preview", page_prefixes(fdir, "preview"))]
                if args.native and os.path.isdir(os.path.join(args.native, fx, comp)):
                    sides.append(("native", page_prefixes(os.path.join(args.native, fx, comp), "native")))
                for side, our_pages in sides:
                    entry = {"fixture": fx, "engine": eng, "compiler": comp, "side": side,
                             "engine_available": bool(einfo.get("available")), "engine_exit": einfo.get("exit"),
                             "status": build.get("status"), "ref_pages": len(ref_pages), "our_pages": len(our_pages),
                             "build": {k: build.get(k) for k in ("diagnostics", "diagnostic_count", "rule_items", "pdf_re_f_count", "pdf_re_f_rects", "pdf_exit", "compile_exit")}}
                    if not ref_pages or not our_pages:
                        entry["skipped"] = "no reference raster" if not ref_pages else "no flashtex raster"
                        entries.append(entry)
                        continue
                    out_dir = os.path.join(args.evidence, "images", fx)
                    tag = f"{eng}-{comp}-{side}"
                    sys.stderr.write(f"== {fx}/{eng}/{comp}/{side}\n")
                    want_images = args.images_for_engines == "all" or eng in args.images_for_engines.split(",")
                    pages = compare_pages(ref_pages, our_pages, out_dir, tag, args.threshold, args.dpi, args.max_png_bytes, args.evidence, want_images)
                    entry["pages"] = pages
                    entry["raster"] = summarise(pages)
                    if side == "export" and ref_words is not None:
                        our_words = words_of(os.path.join(fdir, "words.json"))
                        if our_words is not None:
                            entry["words"] = compare_words(ref_words, our_words)
                    t = check_thresholds(entry, thresholds)
                    if t is not None:
                        entry["thresholds"] = t
                    entries.append(entry)

    regress_result = None
    if args.regress:
        regress_result = regress(entries, args.regress, {"ssim_8x8_mean": args.regress_tolerance,
                                                         "diff_mean": args.regress_tolerance * 100,
                                                         "above_threshold_fraction": args.regress_tolerance})
    thresholds_result = None
    if thresholds is not None:
        thresholds_result = [e for e in entries if e.get("thresholds", {}).get("failures")]
    metrics = {"generated_utc": datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ"), "dpi": args.dpi,
               "threshold": args.threshold, "pillow": HAVE_PIL, "entries": entries,
               "regress": regress_result, "threshold_failures": [f"{e['fixture']}/{e['engine']}/{e['compiler']}/{e['side']}" for e in (thresholds_result or [])]}
    json.dump(metrics, open(os.path.join(args.evidence, "metrics.json"), "w"), indent=1)
    open(os.path.join(args.evidence, "report.md"), "w", encoding="utf-8").write(
        build_report(entries, prov, args.evidence, thresholds_result, regress_result, args))
    rc = 0
    if thresholds_result:
        sys.stderr.write(f"threshold failures: {len(thresholds_result)}\n")
        rc = 4
    if regress_result and regress_result.get("worse"):
        sys.stderr.write(f"regressions: {len(regress_result['worse'])}\n")
        rc = 3
    return rc


if __name__ == "__main__":
    sys.exit(main())
