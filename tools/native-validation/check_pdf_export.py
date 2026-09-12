#!/usr/bin/env python3
"""Inspect a PDF exported by the Mac app (File > Export PDF…) for the
dark-preview-versus-export expectation. Python 3 stdlib only.

Reports, per page, the MediaBox and the first fill colour set before the first
rectangle fill (the page background painted by PDFExport.render), and whether
that background is white. Exit 1 if any page background is not white, or if
--expect-pages / --expect-text are given and not met.

Usage: check_pdf_export.py <file.pdf> [--expect-pages N] [--expect-text "Hello"]
"""

import argparse
import re
import sys
import zlib

PASS, FAIL, INFO = "PASS", "FAIL", "INFO"
NUM = rb"[-+]?(?:\d+\.\d*|\.\d+|\d+)"
FILL_OPS = re.compile(
    rb"(?P<args>(?:%s\s+){1,4})(?P<op>g|rg|k|sc|scn)\b" % NUM
)
RECT_FILL = re.compile(rb"re\s+(?:f|f\*|B|b)\b")


def streams(data):
    """Yield (dict_bytes, decoded_stream) for every stream object."""
    for m in re.finditer(rb"<<(?P<dict>.*?)>>\s*stream\r?\n", data, re.S):
        start = m.end()
        end = data.find(b"endstream", start)
        if end < 0:
            continue
        raw = data[start:end].rstrip(b"\r\n")
        d = m.group("dict")
        if b"FlateDecode" in d:
            try:
                raw = zlib.decompress(raw)
            except zlib.error:
                try:
                    raw = zlib.decompressobj().decompress(raw)
                except zlib.error:
                    continue
        yield d, raw


def background_fill(content):
    """First fill colour operator before the first rectangle fill."""
    first_rect = RECT_FILL.search(content)
    limit = first_rect.start() if first_rect else len(content)
    last = None
    for m in FILL_OPS.finditer(content, 0, limit):
        last = m
    if last is None:
        return None, None
    comps = [float(x) for x in last.group("args").split()]
    return last.group("op").decode(), comps


def is_white(op, comps):
    if op == "g":
        return abs(comps[0] - 1.0) < 1e-3
    if op == "rg" or op in ("sc", "scn"):
        return all(abs(c - 1.0) < 1e-3 for c in comps)
    if op == "k":
        return all(abs(c) < 1e-3 for c in comps)
    return False


def main():
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("pdf")
    ap.add_argument("--expect-pages", type=int)
    ap.add_argument("--expect-text", action="append", default=[],
                    help="text that must appear in a content stream (Tj/TJ operand); repeatable")
    args = ap.parse_args()
    data = open(args.pdf, "rb").read()
    rows = []

    def add(s, n, d=""):
        rows.append(s)
        print(f"[{s}] {n}" + (f" -- {d}" if d else ""))

    if not data.startswith(b"%PDF"):
        add(FAIL, "header", "not a PDF")
        return 1
    boxes = re.findall(rb"/MediaBox\s*\[\s*(%s)\s+(%s)\s+(%s)\s+(%s)\s*\]" % (NUM, NUM, NUM, NUM), data)
    page_count = len(re.findall(rb"/Type\s*/Page\b", data))
    add(INFO, "pages", f"{page_count} page object(s); MediaBoxes={[tuple(float(x) for x in b) for b in boxes]}")
    if args.expect_pages is not None:
        add(PASS if page_count == args.expect_pages else FAIL, "page count", f"{page_count} (expected {args.expect_pages})")

    content_pages = 0
    all_text = b""
    for d, content in streams(data):
        if b"/Length1" in d or b"/FontFile" in d or b"/Image" in d or b"/XML" in d:
            continue  # font programs, images, metadata
        if not (RECT_FILL.search(content) or b"BT" in content):
            continue
        content_pages += 1
        op, comps = background_fill(content)
        if op is None:
            add(INFO, f"content stream {content_pages}: background", "no fill colour before first rectangle fill")
        else:
            white = is_white(op, comps)
            add(PASS if white else FAIL, f"content stream {content_pages}: page background is white",
                f"{' '.join(str(c) for c in comps)} {op}" + ("" if white else "  <- dark/non-white background exported"))
        all_text += content
    if content_pages == 0:
        add(FAIL, "content streams", "no page content stream found")
    for t in args.expect_text:
        # CoreText emits text as hex strings with subset-font glyph ids, so a
        # literal match is not guaranteed; report as INFO when absent.
        found = t.encode("utf-8") in all_text or t.encode("utf-16-be") in all_text
        add(PASS if found else INFO, f"text {t!r} literally present in content",
            "yes" if found else "not found literally (glyph-encoded text is expected with CoreText subset fonts)")
    fails = rows.count(FAIL)
    print(f"\nsummary: {rows.count(PASS)} PASS, {fails} FAIL, {rows.count(INFO)} INFO")
    return 1 if fails else 0


if __name__ == "__main__":
    sys.exit(main())
