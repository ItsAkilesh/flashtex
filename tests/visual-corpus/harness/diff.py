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

def _fdata(im):
    """Float pixel values of an 'F' image as a list (avoids the deprecated getdata)."""
    import struct as _st
    b = im.tobytes()
    return list(_st.unpack("<%df" % (len(b) // 4), b))


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
        return _fdata(im.resize((bw, bh), Image.BOX)), bw, bh
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
        return _fdata(prod.crop((0, 0, bw * n, bh * n)).resize((bw, bh), Image.BOX))
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


def add_png_text(path, fields):
    """Insert Latin-1 `tEXt` chunks (keyword -> text) before the first IDAT of an existing PNG."""
    data = open(path, "rb").read()
    if not data.startswith(b"\x89PNG\r\n\x1a\n"):
        return
    def chunk(t, b):
        c = t + b
        return struct.pack(">I", len(b)) + c + struct.pack(">I", zlib.crc32(c) & 0xFFFFFFFF)
    i, out, inserted = 8, [data[:8]], False
    while i < len(data):
        n, = struct.unpack(">I", data[i:i + 4])
        t = data[i + 4:i + 8]
        if t == b"IDAT" and not inserted:
            for k, v in fields.items():
                out.append(chunk(b"tEXt", k.encode("latin-1") + b"\x00" + v.encode("latin-1", "replace")))
            inserted = True
        out.append(data[i:i + 12 + n])
        i += 12 + n
    open(path, "wb").write(b"".join(out))


def png_text(path):
    """Read tEXt chunks back (used by the self-test)."""
    data = open(path, "rb").read()
    i, out = 8, {}
    while i < len(data):
        n, = struct.unpack(">I", data[i:i + 4])
        t = data[i + 4:i + 8]
        if t == b"tEXt":
            k, _, v = data[i + 8:i + 8 + n].partition(b"\x00")
            out[k.decode("latin-1")] = v.decode("latin-1")
        i += 12 + n
    return out


def write_png(path, w, h, rgb, max_bytes=None, footer=None, rasterize=None):
    """Write RGB bytes as PNG; if larger than max_bytes, box-downsample by 2 and retry.
    With `footer` and a `rasterize` binary, the provenance footer is burned into the
    pixels (and XMP description) by `rasterize annotate` after each encode."""
    import subprocess
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
        if footer and rasterize:
            tmp = path + ".raw.png"
            os.replace(path, tmp)
            r = subprocess.run([rasterize, "annotate", tmp, path, footer], capture_output=True)
            if r.returncode != 0 or not os.path.exists(path):
                os.replace(tmp, path)
            else:
                os.remove(tmp)
        if footer:
            add_png_text(path, {"Description": footer, "Software": "FlashTeX visual corpus harness (tests/visual-corpus/harness/diff.py)"})
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


def exact_compare(prefix_a, prefix_b):
    """Byte-exact RGBA comparison of two rasters of the same size (no normalisation)."""
    wa, ha, ra, _ = load_rgba(prefix_a)
    wb, hb, rb, _ = load_rgba(prefix_b)
    if (wa, ha) != (wb, hb):
        return {"equal": False, "size_a": [wa, ha], "size_b": [wb, hb], "reason": "size mismatch"}
    if ra == rb:
        return {"equal": True, "differing_pixels": 0, "pixels": wa * ha}
    ga, gb = to_gray(wa, ha, ra), to_gray(wb, hb, rb)
    d = absdiff(ga, gb)
    h = histogram(d)
    return {"equal": False, "differing_pixels": wa * ha - h[0], "pixels": wa * ha,
            "max_abs_diff": max(i for i, c in enumerate(h) if c), "reason": "pixel values differ"}


def ink_centroid(g, dark=128):
    """(cx, cy) of pixels darker than `dark`, in px; None if no ink."""
    if HAVE_PIL:
        im = Image.frombytes("L", (g.w, g.h), g.data).point(lambda v: 255 if v < dark else 0)
        cols = list(im.resize((g.w, 1), Image.BOX).tobytes())
        rows = list(im.resize((1, g.h), Image.BOX).tobytes())
    else:
        cols = [0] * g.w
        rows = [0] * g.h
        for y in range(g.h):
            row = g.data[y * g.w:(y + 1) * g.w]
            c = 0
            for x, v in enumerate(row):
                if v < dark:
                    cols[x] += 1
                    c += 1
            rows[y] = c
    tc, tr = sum(cols), sum(rows)
    if tc == 0 or tr == 0:
        return None
    return sum(x * c for x, c in enumerate(cols)) / tc, sum(y * c for y, c in enumerate(rows)) / tr


def shifted(g, dx, dy):
    """Copy of g translated by (dx, dy) px, white-filled."""
    if HAVE_PIL:
        im = Image.new("L", (g.w, g.h), 255)
        im.paste(Image.frombytes("L", (g.w, g.h), g.data), (dx, dy))
        return Gray(g.w, g.h, im.tobytes())
    out = bytearray(b"\xff" * (g.w * g.h))
    for y in range(g.h):
        ty = y + dy
        if 0 <= ty < g.h:
            row = g.data[y * g.w:(y + 1) * g.w]
            x0, x1 = max(0, dx), min(g.w, g.w + dx)
            out[ty * g.w + x0:ty * g.w + x1] = row[x0 - dx:x1 - dx]
    return Gray(g.w, g.h, bytes(out))


def ink_projections(g, dark=128):
    """(column ink counts, row ink counts) for pixels darker than `dark`."""
    if HAVE_PIL:
        im = Image.frombytes("L", (g.w, g.h), g.data).point(lambda v: 255 if v < dark else 0)
        cols = list(im.resize((g.w, 1), Image.BOX).tobytes())
        rows = list(im.resize((1, g.h), Image.BOX).tobytes())
        return cols, rows
    cols, rows = [0] * g.w, [0] * g.h
    for y in range(g.h):
        row = g.data[y * g.w:(y + 1) * g.w]
        c = 0
        for x, v in enumerate(row):
            if v < dark:
                cols[x] += 1
                c += 1
        rows[y] = c
    return cols, rows


def best_shift(a, b, max_shift):
    """Shift d maximising sum(a[i] * b[i + d]) over |d| <= max_shift (b moved by -d aligns to a)."""
    n = len(a)
    best, best_d = -1.0, 0
    for d in range(-max_shift, max_shift + 1):
        lo, hi = max(0, -d), min(n, n - d)
        sc = 0.0
        for i in range(lo, hi):
            if a[i]:
                sc += a[i] * b[i + d]
        if sc > best:
            best, best_d = sc, d
    return best_d


def crop(g, x0, y0, x1, y1):
    x0, y0 = max(0, x0), max(0, y0)
    x1, y1 = min(g.w, x1), min(g.h, y1)
    if x1 <= x0 or y1 <= y0:
        return None
    if HAVE_PIL:
        return Gray(x1 - x0, y1 - y0, Image.frombytes("L", (g.w, g.h), g.data).crop((x0, y0, x1, y1)).tobytes())
    return Gray(x1 - x0, y1 - y0, b"".join(g.data[y * g.w + x0:y * g.w + x1] for y in range(y0, y1)))


def pair_metrics(ref, ours):
    d = absdiff(ref, ours)
    h = histogram(d)
    total = ref.w * ref.h
    ssim, nb, low, _ = ssim_blocks(ref, ours)
    return {"diff_mean": round(sum(i * c for i, c in enumerate(h)) / total, 4),
            "differing_fraction": round((total - h[0]) / total, 6),
            "ssim_8x8_mean": round(ssim, 4), "ssim_blocks": nb}


def registration_diagnostics(ref, ours, dpi, max_shift_pt=60):
    """Global translation between the rasters by 1-D ink-projection correlation
    (column profile -> dx, row profile -> dy), plus the ink-centroid estimate; the
    rendering error is then re-measured with the translation undone. Scale is NOT
    estimated: both sides are rasterized from equal MediaBoxes at the same DPI
    (the native capture's resample factor is recorded separately). Diagnostic
    only: separates 'everything is offset' from 'shapes/positions differ'."""
    cr, co = ink_centroid(ref), ink_centroid(ours)
    if not cr or not co:
        return {"available": False}
    rc, rr = ink_projections(ref)
    oc, orr = ink_projections(ours)
    m = int(round(max_shift_pt * dpi / 72))
    dx = best_shift(rc, oc, m)
    dy = best_shift(rr, orr, m)
    raw = pair_metrics(ref, ours)
    out = {"available": True, "method": "1-D ink projection cross-correlation (dx from columns, dy from rows), search ±%d pt; scale assumed 1; "
                                        "a candidate shift counts as registration error only if undoing it lowers mean|Δ| (else rejected, shift 0)" % max_shift_pt,
           "centroid_shift_pt": [round((co[0] - cr[0]) * 72 / dpi, 2), round((co[1] - cr[1]) * 72 / dpi, 2)],
           "candidate_shift_pt": [round(dx * 72 / dpi, 2), round(dy * 72 / dpi, 2)]}
    after = pair_metrics(ref, shifted(ours, -dx, -dy)) if (dx or dy) else raw
    if (dx or dy) and after["diff_mean"] >= raw["diff_mean"]:
        # The projections correlate best at an offset that does not explain the difference
        # (typically different line breaks): not a registration error, so report none.
        out["candidate_rejected"] = {"shift_pt": out["candidate_shift_pt"], "diff_mean_if_applied": after["diff_mean"],
                                     "ssim_8x8_mean_if_applied": after["ssim_8x8_mean"]}
        dx, dy, after = 0, 0, raw
    reduction = (raw["diff_mean"] - after["diff_mean"]) / raw["diff_mean"] if raw["diff_mean"] else 0.0
    out.update({"shift_px": [dx, dy], "shift_pt": [round(dx * 72 / dpi, 2), round(dy * 72 / dpi, 2)],
                "registered_diff_mean": after["diff_mean"], "registered_differing_fraction": after["differing_fraction"],
                "registered_ssim_8x8_mean": after["ssim_8x8_mean"],
                "error_reduction_fraction": round(reduction, 4),
                # How much of the raw error the shift explains: a genuine global offset removes most of it;
                # a large shift that removes a few percent is a coincidental alignment of unrelated lines.
                "confidence": "none" if not (dx or dy) else "strong" if reduction >= 0.25 else "moderate" if reduction >= 0.05 else "weak"})
    return out


def display_boxes_from_words(words, page, dpi, left_pt=72, right_pt=540):
    """Display-math boxes on `page` from PDFKit word boxes: lines whose ink is
    indented on BOTH sides by >= 24 pt relative to the text measure; consecutive
    such lines merge; 4 pt margin. Returned in px."""
    lines = {}
    for w in words:
        if w["page"] != page:
            continue
        key = round(w["bottom"] / 0.6)
        lines.setdefault(key, []).append(w)
    boxes = []
    for key in sorted(lines):
        ws = lines[key]
        x0, x1 = min(w["x"] for w in ws), max(w["right"] for w in ws)
        y0, y1 = min(w["top"] for w in ws), max(w["bottom"] for w in ws)
        if x0 >= left_pt + 24 and x1 <= right_pt - 24:
            if boxes and y0 - boxes[-1][3] < 14:
                b = boxes[-1]
                boxes[-1] = [min(b[0], x0), b[1], max(b[2], x1), max(b[3], y1)]
            else:
                boxes.append([x0, y0, x1, y1])
    s = dpi / 72
    return [{"name": f"display-{i + 1}", "pt": [round(b[0] - 4, 1), round(b[1] - 4, 1), round(b[2] + 4, 1), round(b[3] + 4, 1)],
             "px": [int((b[0] - 4) * s), int((b[1] - 4) * s), int((b[2] + 4) * s), int((b[3] + 4) * s)]} for i, b in enumerate(boxes)]


def region_metrics(ref, ours, reg_shift, regions):
    """Raw and post-registration metrics per region (regions in px)."""
    out = []
    dx, dy = reg_shift
    reg_img = shifted(ours, -dx, -dy) if (dx or dy) else ours
    for r in regions:
        x0, y0, x1, y1 = r["px"]
        a, b, c = crop(ref, x0, y0, x1, y1), crop(ours, x0, y0, x1, y1), crop(reg_img, x0, y0, x1, y1)
        if a is None or b is None or a.w < 8 or a.h < 8:
            out.append({"name": r["name"], "px": r["px"], "skipped": "empty region"})
            continue
        raw, post = pair_metrics(a, b), pair_metrics(a, c)
        ink_a = sum(histogram(a)[:128])
        out.append({"name": r["name"], "pt": r.get("pt"), "px": r["px"], "ink_pixels_ref": ink_a,
                    "raw": raw, "registered": post})
    return out


def sha256_file(path):
    import hashlib
    return hashlib.sha256(open(path, "rb").read()).hexdigest() if os.path.exists(path) else None


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


def compare_pages(ref_prefixes, our_prefixes, out_dir, tag, threshold, dpi, max_png, root, images=True,
                  ref_words=None, footer=None, rasterize=None):
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
        page_footer = (footer + f" | page {n + 1}") if footer else None
        if images:
            ov_size, ov_scale = write_png(ov, w1, h1, overlay_rgb(ref, ours), max_png, page_footer, rasterize)
            hm_size, hm_scale = write_png(hm, w1, h1, heatmap_rgb(diff), max_png, page_footer, rasterize)
        else:
            ov = hm = None
            ov_size = hm_size = ov_scale = hm_scale = 0
        min_run = int(round(10 * dpi / 72))  # >= 10pt of continuous ink
        registration = registration_diagnostics(ref, ours, dpi)
        # Regions (pt -> px): text area inside the 1in margins, header/footer bands, display boxes from reference words.
        sc = dpi / 72
        regions = [{"name": "text-area", "pt": [72, 72, 540, 720], "px": [int(72 * sc), int(72 * sc), int(540 * sc), int(720 * sc)]},
                   {"name": "header-band", "pt": [0, 0, 612, 72], "px": [0, 0, w1, int(72 * sc)]},
                   {"name": "footer-band", "pt": [0, 720, 612, 792], "px": [0, int(720 * sc), w1, h1]}]
        if ref_words:
            regions += display_boxes_from_words(ref_words, n + 1, dpi)
        regions_out = region_metrics(ref, ours, registration.get("shift_px", [0, 0]) if registration.get("available") else [0, 0], regions)
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
            if best is None:  # FlashTeX drew a rule where the reference has none on this page
                best = {"unmatched": True, "ours_len_pt": round((r["x1"] - r["x0"]) * 72 / dpi, 2),
                        "ours_y_pt": round((r["y0"] + r["y1"]) / 2 * 72 / dpi, 2), "ours_thick_px": r["y1"] - r["y0"] + 1}
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
            "registration": registration, "regions": regions_out,
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
            "registration_shift_pt": [p.get("registration", {}).get("shift_pt") for p in ok],
            "registration_confidence": [p.get("registration", {}).get("confidence") for p in ok],
            "registered_diff_mean": round(statistics.fmean(p["registration"]["registered_diff_mean"] for p in ok if p.get("registration", {}).get("available")), 4)
                if any(p.get("registration", {}).get("available") for p in ok) else None,
            "registered_ssim_8x8_mean": round(statistics.fmean(p["registration"]["registered_ssim_8x8_mean"] for p in ok if p.get("registration", {}).get("available")), 4)
                if any(p.get("registration", {}).get("available") for p in ok) else None,
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


def build_report(entries, prov, evidence, thresholds_result, regress_result, args, gates=()):
    L = []
    L.append("# FlashTeX visual corpus: reference-render and raster-diff evidence\n")
    L.append(f"Generated {prov.get('generated_utc')} on {prov.get('machine', 'mac-m1max-a')} by `tests/visual-corpus/harness/run.sh`.\n")
    L.append("**Scope statement.** These are narrow-case measurements over a small declared corpus. "
             "They never claim general pixel perfection, LaTeX compatibility, or parity outside these fixtures, "
             "these engines, this font, this page size, this DPI and these builds. The reference engines are "
             "test oracles only; FlashTeX never invokes them and remains an original Rust implementation.\n")
    L.append("**Acceptance vs diagnostics.** The only acceptance signals in this report are the exact-equality gates below "
             "(zero pixel difference between FlashTeX's export raster and its preview rasters, and raw PDF byte identity against "
             "the pinned profile). Every tolerance, threshold, SSIM, registration shift or regression comparison further down is a "
             "diagnostic to explain *why* something differs; none of them ever counts as acceptance.\n")
    L.append("## Exact-equality gates (acceptance)\n")
    if not gates:
        L.append("No FlashTeX outputs were available to gate.\n")
    else:
        L.append("| Fixture | Compiler | export = preview-equivalent | export = native preview capture | PDF bytes = pinned | PDF SHA-256 |")
        L.append("|---|---|---|---|---|---|")
        def cell(v):
            if v == "unavailable":
                return "unavailable"
            if isinstance(v, list):
                bad = [x for x in v if not x.get("equal")]
                if not bad:
                    return f"**EQUAL** ({len(v)} page{'s' if len(v) != 1 else ''})"
                b = bad[0]
                return f"DIFFERENT: {b.get('differing_pixels', '?')}/{b.get('pixels', '?')} px" + (f", max |Δ| {b['max_abs_diff']}" if "max_abs_diff" in b else f" ({b.get('reason')})")
            return str(v)
        for g in gates:
            pdfc = {True: "**EQUAL**", False: "DIFFERENT", "unpinned": "unpinned (no reference profile entry)",
                    "baseline": "baseline pinned in this run (not a pass)"}[g["pdf_byte_identity"]]
            L.append(f"| {g['fixture']} | {g['compiler']} | {cell(g['export_vs_preview_equivalent'])} | {cell(g['export_vs_native_preview'])} | {pdfc} | `{(g['pdf_sha256'] or '')[:16]}…` |")
        L.append("")
        L.append(f"- Reference profile: `{os.path.basename(args.profile) if args.profile else 'none'}`"
                 + (" — **pinned/re-baselined in this run** (explicit `--pin-profile`)." if args.pin_profile else "."))
        L.append("- Classification of native-preview differences: the native capture comes from a screen capture at the display's "
                 "backing scale, resampled to the raster size, so a DIFFERENT result there is expected to be dominated by "
                 "resampling and text rasterization (CoreText on screen vs CoreGraphics PDF rendering); it is reported as-is, "
                 "without normalisation. Preview-equivalent vs export differences isolate the drawing path (CoreText glyph "
                 "run vs the PDF writer's text operators) from any capture effects.")
        L.append("")
    L.append("## Provenance\n")
    for k in ("suite_branch", "suite_sha", "input_main_sha", "machine", "os", "swift", "cargo", "python", "pillow"):
        if k in prov:
            L.append(f"- {k}: `{prov[k]}`")
    L.append(f"- DPI: {args.dpi} (every raster: CoreGraphics bitmap, sRGB IEC61966-2.1, 8-bit RGBA, white opaque background, "
             f"MediaBox mapped to width_pt*{args.dpi}/72 px; text antialiased, font smoothing off, subpixel positioning on)")
    L.append(f"- Overlay/heatmap PNGs emitted for engines: {args.images_for_engines}, sides: {args.images_for_sides} (metrics are computed for every engine and side; PNGs are downscaled by 2 until ≤{args.max_png_bytes} B)")
    L.append("- Every overlay/heatmap PNG carries a burned-in footer (and XMP dc:description) with the fixture SHA-256, oracle engine+version+font, "
             "compiler and flashtex-pdf SHAs, side, DPI/colour profile and run stamp; `metrics.json` repeats them per entry under `provenance`.")
    L.append("- Registration: global (dx,dy) between reference and candidate estimated by 1-D ink-projection cross-correlation (±60 pt search; "
             "scale assumed 1 because both sides are rasterized from equal MediaBoxes at the same DPI — the native capture's resample factor is "
             "recorded separately). Tables show raw error, the registration shift, and the rendering error after undoing the shift. "
             "Regions: text area (1in margins), header/footer bands, and display-math boxes derived from the reference word boxes.")
    L.append(f"- Pixel threshold for `above_threshold_fraction`: |Δluma| ≥ {args.threshold}/255; SSIM: 8×8 blocks, K1=0.01, K2=0.03")
    L.append(f"- Arithmetic backend: {'Pillow ' + prov.get('pillow', '?') + ' (accelerator; identical integer results to the stdlib path)' if HAVE_PIL else 'pure Python stdlib'}")
    L.append("")
    L.append("### Reference engines (oracle only)\n")
    eng = prov.get("engines", {})
    L.append("| Oracle | Available | Version | Body font | Preamble |")
    L.append("|---|---|---|---|---|")
    for e in ("pdflatex", "pdflatex-lm", "xelatex", "xelatex-lm", "lualatex", "lualatex-lm"):
        base = e.split("-")[0]
        row = eng.get(base, {})
        if e == "pdflatex":
            font = "URW Nimbus Roman (`times` package, T1 fontenc)"
        elif e == "pdflatex-lm":
            font = "Latin Modern Roman Type 1 (`lmodern` package, T1 fontenc) — LaTeX's default Computer Modern look"
        elif e.endswith("-lm"):
            font = "Latin Modern Roman OpenType (`fontspec`, lmroman12-*.otf from the TeX Live tree by explicit path; bold-italic uses lmroman10-bolditalic, the only LM bold-italic face)"
        else:
            font = "Times New Roman (`fontspec`, /System/Library/Fonts/Supplemental/Times New Roman.ttf)"
        pre = prov.get("preambles", {}).get(e, "").replace("\n", " ")
        avail = "yes" if row.get("available") else ("NO — PDFs reused from run " + str(row["reused_from"].get("run")) if row.get("reused_from") else "NO")
        L.append(f"| {e} | {avail} | {row.get('version', '-')} | {font if (row.get('available') or row.get('reused_from')) else '-'} | `{pre}` |")
    refs = prov.get("references") or {}
    if refs:
        L.append("\n### Reference availability this run\n")
        L.append(f"- rendered fresh by an installed engine: {len(refs.get('fresh', []))} fixture/oracle pairs")
        by_run = {}
        for k, v in (refs.get("reused") or {}).items():
            by_run.setdefault((v.get("run"), v.get("engine_version")), []).append(k)
        for (run, ver), keys in sorted(by_run.items()):
            L.append(f"- reused from run `{run}` ({ver}): {len(keys)} pairs — the PDF stored in `evidence/{run}/references/` was reused after its "
                     "recorded fixture SHA-256 and preamble matched; raster and word boxes re-derived by this run's rasterizer; the engine "
                     "version above is the one that produced that PDF, not a binary present on this machine")
        vr = refs.get("vs_recorded")
        if vr:
            sm = vr.get("summary", {})
            L.append(f"- live renders vs previously recorded references ({vr.get('recorded_run')}, {vr.get('recorded_distribution')}): "
                     f"{sm.get('pairs')} pairs, rasters pixel-identical at {args.dpi:g} DPI for {sm.get('raster_identical')} of them "
                     f"(max differing px {sm.get('max_differing_px')}); PDF bytes identical for {sm.get('pdf_identical')} "
                     f"(differences are CreationDate/ModDate, trailer /ID and compressed-stream bytes only); details in `{vr.get('file')}`")
        if refs.get("unavailable"):
            L.append(f"- **reference unavailable** (reported, not failed): {len(refs['unavailable'])} pairs")
            for u in refs["unavailable"][:24]:
                L.append(f"  - {u['key']}: {u['reason']}")
    L.append("\nThe `-lm` oracles are the intended primary apples-to-apples target once a Latin-Modern-metrics FlashTeX pipeline "
             "exists; the Times oracles match the current compiler's Times metrics. Both are reported for every fixture.")
    pk = prov.get("packages") or {}
    if pk.get("distribution"):
        L.append(f"\nTeX distribution this run: **{pk['distribution']}**, texbin → `{pk.get('texbin_realpath', '?')}`, {pk.get('tlmgr', 'tlmgr version unknown')}.")
    L.append(f"\nEngine flags: `{' '.join(prov.get('engine_flags', []))}`. Page size: US letter 612×792 pt for every producer "
             "(checked per page from the MediaBox). LaTeX package versions: see `provenance.json` → `packages`.\n")
    L.append("### FlashTeX builds under test\n")
    for c in prov.get("compilers", []):
        if c.get("build_ok") is False:
            L.append(f"- compiler `{c['label']}`: `{c['ref']}` @ `{c['sha']}` ({c.get('crate')}/{c.get('binary')}) — **DID NOT BUILD**, not compared: `{c.get('build_error')}`")
        else:
            L.append(f"- compiler `{c['label']}`: `{c['ref']}` @ `{c['sha']}` ({c.get('crate', 'crates/compiler')}) — {c.get('note', '')}")
    p = prov.get("pdf_writer", {})
    L.append(f"- PDF writer: `{p.get('ref')}` @ `{p.get('sha')}` (`flashtex-pdf --verify{' --embed-font auto' if p.get('embed') else ''}`; "
             f"body font Times-Roman standard-14, Unicode fallback subset of {p.get('embed_font', 'none')})")
    L.append("- export raster: the flashtex-pdf PDF rasterized by the same CoreGraphics rasterizer as the references")
    L.append("- preview-equivalent raster: `rasterize preview` re-implements the Mac app's `PDFExport.render` draw (CoreText "
             "`Times-Roman` at x_pt/baseline_y_pt/font_size_pt, U+2500 runs as 0.5em×0.0857em rules) straight into the bitmap. "
             "It links nothing from apps/mac and is **not** the SwiftUI preview; it is labelled preview-equivalent throughout.")
    if prov.get("native_preview"):
        npv = prov["native_preview"]
        L.append(f"- native preview capture: {npv.get('method')}. App: `{npv.get('app')}`. Display(s): {npv.get('display')}. "
                 f"Capture backing factor(s): {npv.get('backing_scale_factors')} px/pt; resample factor to the 144-DPI raster: "
                 f"{npv.get('resample_scale_min')}–{npv.get('resample_scale_max')} (>1 means the capture was UPSAMPLED, so glyph edges are "
                 f"interpolated and this comparison is coarser than the export one).")
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
        L.append("| Fixture | Engine | Compiler | Pages ref/ours | Status | mean\\|Δ\\| raw | SSIM₈ raw | registration Δ pt (dx,dy per page; `weak`/`moderate` = shift explains <25% of the error) | mean\\|Δ\\| after reg | SSIM₈ after reg | max | differing | ≥thr | words ref/ours/aligned | seq= | mean\\|dx\\| pt | mean\\|dy\\| pt | line-start agree | rules ref/ours | overlay |")
        L.append("|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|")
        for e in rows:
            if e.get("skipped"):
                L.append(f"| {e['fixture']} | {e['engine']} | {e['compiler']} | {e.get('ref_pages', '-')}/{e.get('our_pages', '-')} | "
                         f"**{e['skipped']}** | " + " | ".join(["-"] * 15) + " |")
                continue
            r, w = e.get("raster") or {}, e.get("words") or {}
            pg = e.get("pages", [])
            first = next((p for p in pg if p.get("overlay")), None)
            link = f"[p1]({first['overlay']})" if first else "-"
            L.append("| {} | {} | {} | {} | {} | {} | {} | {} | {} | {} | {} | {} | {} | {} | {} | {} | {} | {} | {} | {} |".format(
                e["fixture"], e["engine"], e["compiler"], f"{e.get('ref_pages', '-')}/{e.get('our_pages', '-')}",
                e.get("status", "-"), fmt(r.get("diff_mean")), fmt(r.get("ssim_8x8_mean")),
                "; ".join(f"({v[0]:g},{v[1]:g}){'' if c in ('none', 'strong') else ' ' + str(c)}"
                          for v, c in zip(r.get("registration_shift_pt") or [], r.get("registration_confidence") or []) if v) or "-",
                fmt(r.get("registered_diff_mean")), fmt(r.get("registered_ssim_8x8_mean")),
                fmt(r.get("diff_max")), fmt(r.get("differing_fraction"), 4),
                fmt(r.get("above_threshold_fraction"), 4),
                f"{w.get('ref_words', '-')}/{w.get('ours_words', '-')}/{w.get('aligned', '-')}",
                "yes" if w.get("sequence_equal") else ("no" if w else "-"),
                fmt(w.get("dx_abs_mean"), 2), fmt(w.get("dy_abs_mean"), 2), fmt(w.get("line_start_agreement")),
                f"{r.get('rules_ref', '-')}/{r.get('rules_ours', '-')}", link))
        L.append("")

    table("export", "Diagnostic: export comparison (flashtex-pdf PDF vs reference PDF, both rasterized identically)",
          "This is the PDF-output comparison against the oracle. Word boxes come from PDFKit on both PDFs; rules are ink rows ≥10pt long. "
          "Diagnostic only.")
    table("preview", "Diagnostic: preview-equivalent comparison (CoreText draw of compile_result vs reference PDF raster)",
          "Weaker than a capture of the real preview: it re-implements the app's draw code path rather than exercising the "
          "SwiftUI Canvas. Word-box metrics are not available for this side (no PDF), so they are omitted.")
    npv = prov.get("native_preview") or {}
    table("native", "Diagnostic: native preview capture comparison (screen capture of the running FlashTeXMac preview vs reference PDF raster)",
          "The actual SwiftUI Canvas preview, captured with `screencapture -l <window id>`, page region detected and resampled "
          f"to the reference raster size (resample factor {npv.get('resample_scale_min')}–{npv.get('resample_scale_max')}, "
          f"display backing {npv.get('backing_scale_factors')} px/pt; see provenance). The 'page N' caption corner is masked white. "
          "Word-box metrics are unavailable (a screenshot has no text layer). This is a separate, independent comparison from the "
          "export table above and from the weaker preview-equivalent table.")
    missing = [e for e in (npv.get("entries") or []) if not e.get("native_raster")]
    if missing:
        L.append("Native captures NOT available (reported, not skipped silently):\n")
        for e in missing:
            L.append(f"- {e['fixture']}/{e['compiler']}: {e.get('reason', 'no capture')}")
        L.append("")
    if npv.get("per_entry_resample_scale"):
        L.append("Per-entry resample factor (capture px → raster px): " + ", ".join(f"{k} ×{v}" for k, v in npv["per_entry_resample_scale"].items()) + "\n")

    L.append("## Per-fixture diagnostic details (export side)\n")
    for e in entries:
        if e["side"] != "export" or e.get("skipped"):
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
            reg = p.get("registration") or {}
            if reg.get("available"):
                rej = reg.get("candidate_rejected")
                L.append(f"  - registration error (diagnostic): global shift {reg['shift_pt']} pt by ink-projection correlation "
                         + (f"(correlation candidate {rej['shift_pt']} pt REJECTED: applying it gives mean|Δ| {rej['diff_mean_if_applied']}, not lower; "
                            f"centroid estimate {reg['centroid_shift_pt']} pt); " if rej else f"(centroid estimate {reg['centroid_shift_pt']} pt); ")
                         + f"confidence {reg.get('confidence')} (shift explains {round(100 * reg.get('error_reduction_fraction', 0))}% of raw mean|Δ|); rendering error after undoing it: mean|Δ| "
                         f"{reg['registered_diff_mean']}, differing {reg['registered_differing_fraction']}, SSIM₈ {reg['registered_ssim_8x8_mean']} "
                         f"(raw {p['diff_mean']}, {p['differing_fraction']}, {p['ssim_8x8_mean']})")
            if p.get("regions"):
                L.append("  - regions (raw → after registration, SSIM₈ / mean|Δ|): " + "; ".join(
                    f"{rg['name']} {rg['raw']['ssim_8x8_mean']}→{rg['registered']['ssim_8x8_mean']} / {rg['raw']['diff_mean']}→{rg['registered']['diff_mean']}"
                    + (f" [{rg['pt'][0]:g},{rg['pt'][1]:g}–{rg['pt'][2]:g},{rg['pt'][3]:g} pt]" if rg['name'].startswith('display') else "")
                    for rg in p["regions"] if "raw" in rg))
            if p["rule_matches"]:
                L.append("  - rules (FlashTeX → nearest reference ink row, pt): " + "; ".join(
                    (f"Δx {m['dx_pt']} Δy {m['dy_pt']} len {m['ours_len_pt']} vs {m['ref_len_pt']}, thickness px {m['ours_thick_px']} vs {m['ref_thick_px']}"
                     if m and not m.get("unmatched") else
                     f"FlashTeX rule len {m['ours_len_pt']} at y {m['ours_y_pt']} pt with NO reference rule on this page" if m else "unmatched")
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
        L.append("## Diagnostic thresholds (never acceptance)\n")
        L.append(f"Thresholds file: `harness/{os.path.basename(args.thresholds)}` (copied here as `thresholds.used.json`). A failure here is an acceptance signal for the narrow case only.\n")
        for e in entries:
            t = e.get("thresholds")
            if t:
                L.append(f"- {e['fixture']}/{e['engine']}/{e['compiler']}/{e['side']}: checked {t['checked']}; "
                         + ("**FAIL** " + "; ".join(t["failures"]) if t["failures"] else "pass"))
        L.append("")
    if regress_result is not None:
        L.append("## Diagnostic regression check vs previous evidence (never acceptance)\n")
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
    ap.add_argument("--max-png-bytes", type=int, default=90000, help="downscale overlay/heatmap PNGs by 2 until under this size (hard cap 300 KB in the report contract)")
    ap.add_argument("--native", default=None, help="dir with <fixture>/<compiler>/native-p1.rgba captures")
    ap.add_argument("--profile", default=None, help="pinned reference profile JSON (raw PDF SHA-256 per fixture/compiler)")
    ap.add_argument("--pin-profile", action="store_true", help="write the current PDF SHA-256s into --profile (explicit re-baseline)")
    ap.add_argument("--gate", action="store_true", help="exit 5 when any exact-equality gate fails")
    ap.add_argument("--rasterize", default=None, help="rasterize binary; enables provenance footers burned into overlay/heatmap PNGs")
    ap.add_argument("--images-for-sides", default="export,native",
                    help="comma-separated sides (export, preview, native) whose PNGs are written; metrics are computed for all")
    ap.add_argument("--images-for-engines", default="pdflatex,pdflatex-lm",
                    help="comma-separated engines whose overlay/heatmap PNGs are written (metrics are computed for all); 'all' for every engine")
    ap.add_argument("--from-metrics", default=None, help="rebuild report.md only, from this metrics.json (no comparison is recomputed)")
    args = ap.parse_args()

    os.makedirs(args.evidence, exist_ok=True)
    prov = json.load(open(args.provenance)) if args.provenance else {}
    thresholds = json.load(open(args.thresholds)) if args.thresholds else None
    if args.from_metrics:
        m = json.load(open(args.from_metrics))
        entries, gates = m["entries"], m.get("gates", [])
        tr = [e for e in entries if e.get("thresholds", {}).get("failures")] if thresholds is not None else None
        open(os.path.join(args.evidence, "report.md"), "w", encoding="utf-8").write(
            build_report(entries, prov, args.evidence, tr, m.get("regress"), args, gates))
        return 0
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
                        if not ref_pages:
                            entry["skipped"] = "reference unavailable" if not einfo.get("available") else "no reference raster"
                            entry["reason"] = einfo.get("reason") or f"engine exit {einfo.get('exit')}"
                        else:
                            entry["skipped"] = "no flashtex raster"
                        entries.append(entry)
                        continue
                    out_dir = os.path.join(args.evidence, "images", fx)
                    tag = f"{eng}-{comp}-{side}"
                    sys.stderr.write(f"== {fx}/{eng}/{comp}/{side}\n")
                    want_images = (args.images_for_engines == "all" or eng in args.images_for_engines.split(",")) \
                        and side in args.images_for_sides.split(",")
                    fx_sha = next((f["sha256"] for f in prov.get("fixtures", []) if f["name"] == fx + ".tex"), "")
                    comp_info = next((c for c in prov.get("compilers", []) if c["label"] == comp), {})
                    reused = einfo.get("reused_from") if einfo.get("reused") else None
                    eng_ver = (reused or {}).get("engine_version") or prov.get("engines", {}).get(eng.split("-")[0], {}).get("version", "?")
                    font = ("times/T1" if eng == "pdflatex" else "lmodern/T1" if eng == "pdflatex-lm"
                            else "Latin Modern OTF" if eng.endswith("-lm") else "Times New Roman TTF")
                    entry["provenance"] = {"fixture_sha256": fx_sha, "engine": eng, "engine_version": eng_ver, "font": font,
                                           "reference_pdf_sha256": einfo.get("pdf_sha256"),
                                           "reference_reused_from_run": (reused or {}).get("run"),
                                           "compiler_sha": comp_info.get("sha"), "pdf_writer_sha": (prov.get("pdf_writer") or {}).get("sha"),
                                           "dpi": args.dpi, "color_space": "sRGB IEC61966-2.1", "pixel_format": "RGBA8",
                                           "side": side, "run": prov.get("generated_utc")}
                    footer = (f"fixture {fx}.tex sha256 {fx_sha[:16]} | oracle {eng} {eng_ver} {font}"
                              + (f" (PDF reused from run {reused.get('run')})" if reused else "")
                              + f" | compiler {comp}@{(comp_info.get('sha') or '')[:12]}"
                              f" | flashtex-pdf {((prov.get('pdf_writer') or {}).get('sha') or '')[:12]} | side {side} | {args.dpi:g} DPI sRGB RGBA8 | run {prov.get('generated_utc')}")
                    pages = compare_pages(ref_pages, our_pages, out_dir, tag, args.threshold, args.dpi, args.max_png_bytes, args.evidence, want_images,
                                          ref_words=ref_words, footer=footer, rasterize=args.rasterize)
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

    # Exact-equality gates (acceptance): no tolerance, no registration, no normalisation.
    gates = []
    profile = json.load(open(args.profile)) if args.profile and os.path.exists(args.profile) else {"pdf_sha256": {}}
    new_profile = {"pdf_sha256": {}, "pinned_utc": datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ"),
                   "pdf_writer": prov.get("pdf_writer"), "compilers": prov.get("compilers"), "fixtures": {f["name"]: f["sha256"] for f in prov.get("fixtures", [])}}

    def allequal(v):
        return isinstance(v, list) and bool(v) and all(x.get("equal") for x in v)
    for fx in fixtures:
        for comp in (sorted(os.listdir(os.path.join(args.flashtex, fx))) if os.path.isdir(os.path.join(args.flashtex, fx)) else []):
            fdir = os.path.join(args.flashtex, fx, comp)
            g = {"fixture": fx, "compiler": comp}
            exp, pre = page_prefixes(fdir, "export"), page_prefixes(fdir, "preview")
            nat = page_prefixes(os.path.join(args.native, fx, comp), "native") if args.native and os.path.isdir(os.path.join(args.native, fx, comp)) else []
            g["export_vs_preview_equivalent"] = ([exact_compare(a, b) for a, b in zip(exp, pre)] if exp and pre else "unavailable")
            g["export_vs_native_preview"] = ([exact_compare(a, b) for a, b in zip(exp, nat)] if exp and nat else "unavailable")
            sha = sha256_file(os.path.join(fdir, "flashtex.pdf"))
            key = f"{fx}/{comp}"
            new_profile["pdf_sha256"][key] = sha
            pinned = profile.get("pdf_sha256", {}).get(key)
            g["pdf_sha256"] = sha
            g["pdf_pinned_sha256"] = pinned
            g["pdf_byte_identity"] = ("baseline" if args.pin_profile else "unpinned" if pinned is None else (sha == pinned))
            g["pass_export_preview_equivalent"] = allequal(g["export_vs_preview_equivalent"])
            g["pass_export_native"] = allequal(g["export_vs_native_preview"])
            g["pass_pdf_bytes"] = g["pdf_byte_identity"] is True
            gates.append(g)
    if args.pin_profile and args.profile:
        json.dump(new_profile, open(args.profile, "w"), indent=1)
    gate_failures = [g for g in gates if not (g["pass_export_preview_equivalent"] and g["pass_export_native"] and g["pass_pdf_bytes"])]

    regress_result = None
    if args.regress:
        regress_result = regress(entries, args.regress, {"ssim_8x8_mean": args.regress_tolerance,
                                                         "diff_mean": args.regress_tolerance * 100,
                                                         "above_threshold_fraction": args.regress_tolerance})
    thresholds_result = None
    if thresholds is not None:
        thresholds_result = [e for e in entries if e.get("thresholds", {}).get("failures")]
    metrics = {"generated_utc": datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ"), "dpi": args.dpi,
               "threshold": args.threshold, "pillow": HAVE_PIL, "entries": entries, "gates": gates,
               "gate_failures": [f"{g['fixture']}/{g['compiler']}" for g in gate_failures],
               "regress": regress_result, "threshold_failures": [f"{e['fixture']}/{e['engine']}/{e['compiler']}/{e['side']}" for e in (thresholds_result or [])]}
    json.dump(metrics, open(os.path.join(args.evidence, "metrics.json"), "w"), indent=1)
    open(os.path.join(args.evidence, "report.md"), "w", encoding="utf-8").write(
        build_report(entries, prov, args.evidence, thresholds_result, regress_result, args, gates))
    rc = 0
    if args.gate and gate_failures:
        sys.stderr.write(f"exact-equality gate failures: {len(gate_failures)}\n")
        rc = 5
    if thresholds_result:
        sys.stderr.write(f"threshold failures: {len(thresholds_result)}\n")
        rc = 4
    if regress_result and regress_result.get("worse"):
        sys.stderr.write(f"regressions: {len(regress_result['worse'])}\n")
        rc = 3
    return rc


if __name__ == "__main__":
    sys.exit(main())
