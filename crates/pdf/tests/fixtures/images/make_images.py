#!/usr/bin/env python3
"""Writes the image fixtures for the exact route's image XObject tests.

PNGs are written by hand (zlib + filter types 0/1 alternating per row) so bit
depth, colour type, palette, tRNS and interlacing are explicit. JPEGs come
from Pillow (baseline RGB/gray, progressive RGB, Adobe CMYK). PDFs come from
pdfTeX in ini mode (TEST ONLY; never in the product path), with object
streams, so the importer's object-stream path is exercised.
"""
import os, shutil, struct, subprocess, tempfile, zlib

HERE = os.path.dirname(os.path.abspath(__file__))
CHANNELS = {0: 1, 2: 3, 3: 1, 4: 2, 6: 4}


def chunk(kind, data):
    c = struct.pack(">I", len(data)) + kind + data
    return c + struct.pack(">I", zlib.crc32(kind + data) & 0xFFFFFFFF)


def bpp(bit_depth, color_type):
    return max(1, CHANNELS[color_type] * bit_depth // 8)


def sub_filter(r, n):
    return bytes((r[i] - (r[i - n] if i >= n else 0)) & 255 for i in range(len(r)))


def pack(samples, bit_depth):
    if bit_depth == 8:
        return bytes(samples)
    if bit_depth == 16:
        return b"".join(struct.pack(">H", s) for s in samples)
    out, acc, nbits = bytearray(), 0, 0
    for s in samples:
        acc = (acc << bit_depth) | s
        nbits += bit_depth
        if nbits == 8:
            out.append(acc)
            acc, nbits = 0, 0
    if nbits:
        out.append(acc << (8 - nbits))
    return bytes(out)


def adam7(w, h, bit_depth, color_type, sample_rows):
    passes = [(0, 0, 8, 8), (4, 0, 8, 8), (0, 4, 4, 8), (2, 0, 4, 4), (0, 2, 2, 4), (1, 0, 2, 2), (0, 1, 1, 2)]
    ch = CHANNELS[color_type]
    raw = b""
    for x0, y0, dx, dy in passes:
        for y in range(y0, h, dy):
            samples = []
            for x in range(x0, w, dx):
                samples.extend(sample_rows[y][x * ch : (x + 1) * ch])
            if samples:
                raw += b"\x00" + pack(samples, bit_depth)
    return raw


def png(name, w, h, bit_depth, color_type, sample_rows, plte=None, trns=None, phys_dpi=None, interlace=False):
    if interlace:
        raw = adam7(w, h, bit_depth, color_type, sample_rows)
    else:
        raw = b""
        n = bpp(bit_depth, color_type)
        for i, samples in enumerate(sample_rows):
            r = pack(samples, bit_depth)
            raw += bytes([i % 2]) + (sub_filter(r, n) if i % 2 else r)
    out = b"\x89PNG\r\n\x1a\n"
    out += chunk(b"IHDR", struct.pack(">IIBBBBB", w, h, bit_depth, color_type, 0, 0, 1 if interlace else 0))
    out += chunk(b"gAMA", struct.pack(">I", 45455))
    if phys_dpi:
        ppm = round(phys_dpi / 0.0254)
        out += chunk(b"pHYs", struct.pack(">IIB", ppm, ppm, 1))
    if plte:
        out += chunk(b"PLTE", plte)
    if trns is not None:
        out += chunk(b"tRNS", trns)
    z = zlib.compress(raw, 9)
    # Two IDAT chunks: consumers must concatenate them.
    out += chunk(b"IDAT", z[: len(z) // 2]) + chunk(b"IDAT", z[len(z) // 2 :])
    out += chunk(b"IEND", b"")
    with open(os.path.join(HERE, name), "wb") as f:
        f.write(out)


def grid(w, h, f):
    return [[v for x in range(w) for v in f(x, y)] for y in range(h)]


def main():
    W, H = 48, 32
    rgb = grid(W, H, lambda x, y: ((x * 255) // (W - 1), (y * 255) // (H - 1), 255 - (x * 255) // (W - 1)))
    png("rgb8.png", W, H, 8, 2, rgb)
    png("rgb8-adam7.png", W, H, 8, 2, rgb, interlace=True)
    gray = grid(40, 40, lambda x, y: (((x // 5 + y // 5) % 2) * 200 + x,))
    png("gray8-144dpi.png", 40, 40, 8, 0, gray, phys_dpi=144)
    png("gray2.png", 20, 12, 2, 0, grid(20, 12, lambda x, y: ((x + y) % 4,)))
    plte = bytes(v for i in range(16) for v in ((i * 17) & 255, (255 - i * 17) & 255, (i * 64) & 255))
    pal = grid(32, 24, lambda x, y: (((x // 4) + (y // 3) * 3) % 16,))
    png("palette4.png", 32, 24, 4, 3, pal, plte=plte)
    png("palette8-trns.png", 32, 24, 8, 3, pal, plte=plte, trns=bytes([0, 128] + [255] * 14))
    png("rgba8.png", W, H, 8, 6, grid(W, H, lambda x, y: ((x * 255) // (W - 1), 80, (y * 255) // (H - 1), (x * 255) // (W - 1))))
    png("graya8.png", 24, 24, 8, 4, grid(24, 24, lambda x, y: ((x * 10) & 255, (y * 10) & 255)))
    png("rgb16.png", W, H, 16, 2, grid(W, H, lambda x, y: ((x * 65535) // (W - 1), (y * 65535) // (H - 1), 12345)))
    png("rgba16.png", W, H, 16, 6, grid(W, H, lambda x, y: ((x * 65535) // (W - 1), 30000, (y * 65535) // (H - 1), (y * 65535) // (H - 1))))

    from PIL import Image

    im = Image.new("RGB", (64, 40))
    im.putdata([((x * 4) % 256, (y * 6) % 256, 128) for y in range(40) for x in range(64)])
    im.save(os.path.join(HERE, "rgb-96dpi.jpg"), quality=90, dpi=(96, 96))
    im.save(os.path.join(HERE, "rgb-progressive.jpg"), quality=85, progressive=True)
    im.convert("L").save(os.path.join(HERE, "gray.jpg"), quality=90)
    cmyk = Image.new("CMYK", (40, 40))
    cmyk.putdata([((x * 6) % 256, (y * 6) % 256, 60, 20) for y in range(40) for x in range(40)])
    cmyk.save(os.path.join(HERE, "cmyk.jpg"), quality=90)

    pdftex = shutil.which("pdftex") or "/Library/TeX/texbin/pdftex"
    env = dict(os.environ, SOURCE_DATE_EPOCH="1767225600", FORCE_SOURCE_DATE="1")
    pages = {
        "page-cropbox.pdf": r"\pdfpagesattr{/CropBox [10 5 190 115]}",
        "page-rotate90.pdf": r"\pdfpageattr{/Rotate 90}",
    }
    body = (
        r"\pdfpagewidth=200bp \pdfpageheight=120bp \pdfhorigin=0pt \pdfvorigin=0pt"
        r"\hsize=200bp \vsize=120bp \parindent=0pt \font\x=cmr10 at 20pt"
        r"\shipout\vbox to 120bp{\hbox to 200bp{\vrule width 30bp height 120bp\hfil"
        r"\vbox to 120bp{\vss\hbox{\x FlashTeX}\vss}\hfil"
        r"\pdfliteral{q 1 0 0 RG 4 w 10 10 m 190 110 l S Q}\vrule width 10bp height 60bp}}\end"
    )
    for name, setup in pages.items():
        with tempfile.TemporaryDirectory() as tmp:
            with open(os.path.join(tmp, "p.tex"), "w") as f:
                f.write("\\catcode`\\{=1 \\catcode`\\}=2 " + r"\pdfoutput=1 \pdfminorversion=5 \pdfcompresslevel=9 \pdfobjcompresslevel=2 " + setup + body + "\n")
            subprocess.run([pdftex, "-ini", "-interaction=batchmode", "p.tex"], cwd=tmp, env=env, check=True, capture_output=True)
            shutil.copy(os.path.join(tmp, "p.pdf"), os.path.join(HERE, name))


if __name__ == "__main__":
    main()
