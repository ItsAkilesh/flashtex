#!/usr/bin/env python3
"""Writes the inline-graphics fixtures (TEST ONLY): `NN-*.tex` and the small
deterministic images under `images/` they include.

Images (written byte-identically on every run, no external tools):
  images/checker.png      80x40 px, pHYs 72 dpi  -> 80bp x 40bp, four colour quadrants
  images/tall.png         20x60 px, pHYs 144 dpi -> 10bp x 30bp
  images/sub/dot.png      12x12 px, no pHYs      -> 12bp x 12bp
  images/frame.pdf        MediaBox 0 0 100 50 (uncompressed), an outlined box and a cross
"""
import os, struct, zlib

HERE = os.path.dirname(os.path.abspath(__file__))


def png(path, w, h, dpi, pixel):
    def chunk(kind, data):
        return struct.pack(">I", len(data)) + kind + data + struct.pack(">I", zlib.crc32(kind + data) & 0xFFFFFFFF)
    raw = b"".join(b"\0" + b"".join(bytes(pixel(x, y)) for x in range(w)) for y in range(h))
    out = b"\x89PNG\r\n\x1a\n" + chunk(b"IHDR", struct.pack(">IIBBBBB", w, h, 8, 2, 0, 0, 0))
    if dpi:
        ppm = round(dpi / 0.0254)
        out += chunk(b"pHYs", struct.pack(">IIB", ppm, ppm, 1))
    out += chunk(b"IDAT", zlib.compress(raw, 9)) + chunk(b"IEND", b"")
    os.makedirs(os.path.dirname(path), exist_ok=True)
    open(path, "wb").write(out)


def pdf(path):
    content = b"0.8 0.2 0.2 RG 2 w 5 5 90 40 re S 0 0 1 rg 0 0 m 100 50 l 0 50 m 100 0 l S 40 20 20 10 re f\n"
    objs = [
        b"<< /Type /Catalog /Pages 2 0 R >>",
        b"<< /Type /Pages /Kids [3 0 R] /Count 1 >>",
        b"<< /Type /Page /Parent 2 0 R /MediaBox [0 0 100 50] /Resources << >> /Contents 4 0 R >>",
        b"<< /Length %d >>\nstream\n" % len(content) + content + b"endstream",
    ]
    out = b"%PDF-1.4\n"
    offsets = []
    for i, o in enumerate(objs, 1):
        offsets.append(len(out))
        out += b"%d 0 obj\n" % i + o + b"\nendobj\n"
    xref = len(out)
    out += b"xref\n0 %d\n0000000000 65535 f \n" % (len(objs) + 1)
    out += b"".join(b"%010d 00000 n \n" % off for off in offsets)
    out += b"trailer\n<< /Size %d /Root 1 0 R >>\nstartxref\n%d\n%%%%EOF\n" % (len(objs) + 1, xref)
    open(path, "wb").write(out)


PRE = "\\documentclass{article}\n\\usepackage%s{graphicx}\n\\pagestyle{empty}\n%s\\begin{document}\n"
POST = "\n\\end{document}\n"

FIXTURES = {
    "01-inline-heights": (
        "", "",
        "Text with a small \\includegraphics[height=4pt]{images/checker} image, a taller\n"
        "\\includegraphics[height=12pt]{images/checker} one and a very tall\n"
        "\\includegraphics[height=36pt]{images/checker} one in the same paragraph of\n"
        "running text that continues for a while so that it wraps onto more lines and\n"
        "the tall image pushes the baselines apart as the interline glue requires.\n\n"
        "A second paragraph follows the first one.",
    ),
    "02-inline-breaks": (
        "", "",
        "Images take part in line breaking as boxes:\n" + " ".join(
            f"word{i} \\includegraphics[width=0.9in]{{images/checker}}" for i in range(9)
        ) + "\nand the paragraph ends here.",
    ),
    "03-inline-natural": (
        "", "",
        "Natural sizes: \\includegraphics{images/checker} then \\includegraphics{images/tall}\n"
        "then \\includegraphics{images/frame} and \\includegraphics{images/sub/dot} end.",
    ),
    "04-inline-keys": (
        "", "",
        "Keys: \\includegraphics[width=2cm]{images/checker} a \\includegraphics[totalheight=1cm]{images/tall}\n"
        "b \\includegraphics[scale=0.5]{images/frame} c \\includegraphics[width=2cm,height=2cm,keepaspectratio]{images/checker}\n"
        "d \\includegraphics[width=1cm,height=2cm]{images/frame} e.",
    ),
    "05-scalebox": (
        "", "",
        "A \\scalebox{2}{Big} word, \\scalebox{0.5}{small text} and \\scalebox{1.5}[0.75]{wide}\n"
        "sit in a paragraph that goes on long enough to wrap onto a second line of text\n"
        "so that the scaled boxes change the line heights.",
    ),
    "06-reflect-negative": (
        "", "",
        "Start \\reflectbox{Mirror} then \\scalebox{-1}[1]{left} then \\scalebox{1}[-1]{down}\n"
        "then \\scalebox{-0.5}{both} end.",
    ),
    "07-resizebox": (
        "", "",
        "Sizes \\resizebox{2cm}{!}{Resized text} and \\resizebox{!}{20pt}{High} and\n"
        "\\resizebox{3cm}{8pt}{Both ways} end.",
    ),
    "08-resizebox-star": (
        "", "",
        "Total \\resizebox*{!}{2\\baselineskip}{gyp} and \\resizebox{!}{2\\baselineskip}{gyp}\n"
        "and \\resizebox{\\width}{2\\height}{tall} end.",
    ),
    "09-rotate-quarter": (
        "", "",
        "Turns \\rotatebox{90}{Up} and \\rotatebox{-90}{Down} and \\rotatebox{180}{Turned}\n"
        "and \\rotatebox{270}{Back} end.",
    ),
    "10-rotate-angles": (
        "", "",
        "Angles \\rotatebox{30}{thirty} and \\rotatebox{45}{fortyfive} and\n"
        "\\rotatebox{-60}{minus} and \\rotatebox{135}{obtuse} end.",
    ),
    "11-rotate-origin-lr": (
        "", "",
        "Origins \\rotatebox[origin=l]{40}{Left} and \\rotatebox[origin=r]{40}{Right} and\n"
        "\\rotatebox[origin=c]{40}{Center} end.",
    ),
    "12-rotate-origin-tb": (
        "", "",
        "Origins \\rotatebox[origin=t]{-35}{Top} and \\rotatebox[origin=b]{-35}{Bottom} and\n"
        "\\rotatebox[origin=B]{-35}{Base} and \\rotatebox[origin=lt]{60}{Corner} and\n"
        "\\rotatebox[origin=rB]{60}{Other} end.",
    ),
    "13-rotate-image": (
        "", "",
        "Rotated \\rotatebox{30}{\\includegraphics[width=1cm]{images/checker}} and\n"
        "\\rotatebox[origin=c]{90}{\\includegraphics[width=1cm]{images/checker}} end.",
    ),
    "14-scale-image": (
        "", "",
        "Scaled \\scalebox{0.5}{\\includegraphics{images/checker}} and\n"
        "\\resizebox{1in}{!}{\\includegraphics{images/frame}} and\n"
        "\\reflectbox{\\includegraphics[height=1cm]{images/tall}} end.",
    ),
    "15-graphicx-angle": (
        "", "",
        "Keys \\includegraphics[angle=45,width=1cm]{images/checker} and\n"
        "\\includegraphics[width=1cm,angle=90]{images/checker} and\n"
        "\\includegraphics[origin=c,angle=30,height=8mm]{images/checker} and\n"
        "\\includegraphics[angle=-30,scale=0.3]{images/frame} end.",
    ),
    "16-trim-clip": (
        "", "",
        "Trimmed \\includegraphics[trim=10 5 20 10,clip]{images/checker} and\n"
        "\\includegraphics[trim=10 5 20 10,clip,width=2cm]{images/checker} and\n"
        "\\includegraphics[trim=1cm 0 0 0.5cm,clip]{images/frame} end.",
    ),
    "17-trim-noclip": (
        "", "",
        "Unclipped \\includegraphics[trim=10 5 20 10]{images/checker} and\n"
        "\\includegraphics[trim=10 5 20 10,width=2cm]{images/frame} end.",
    ),
    "18-viewport": (
        "", "",
        "Viewport \\includegraphics[viewport=20 10 70 40,clip]{images/frame} and\n"
        "\\includegraphics[viewport=20 10 70 40]{images/frame} and\n"
        "\\includegraphics*[20,10][70,40]{images/frame} end.",
    ),
    "19-graphicspath": (
        "", "\\graphicspath{{images/}{images/sub/}}\n",
        "Search path \\includegraphics{checker} and \\includegraphics[height=1em]{dot} end.",
    ),
    "20-draft-key": (
        "", "",
        "Draft \\includegraphics[draft,width=3cm,height=1cm]{images/checker} and\n"
        "\\includegraphics[draft]{images/frame} end.",
    ),
    "21-draft-option": (
        "[draft]", "",
        "Global draft \\includegraphics[width=2cm]{images/checker} and\n"
        "\\includegraphics{images/tall} end.",
    ),
    "22-nested": (
        "", "",
        "Nested \\scalebox{1.5}{\\rotatebox{30}{nested \\includegraphics[height=1em]{images/checker}}}\n"
        "and \\rotatebox{20}{\\scalebox{2}[1]{wide}} end.",
    ),
    "23-transform-math": (
        "", "",
        "Math \\rotatebox{20}{$x^2+y$} and \\scalebox{2}{$a+b$} end.",
    ),
    "24-transform-breaks": (
        "", "",
        "Transformed boxes break lines like words: " + " ".join(
            f"\\scalebox{{1.2}}{{word{i}}}" if i % 3 == 0 else (f"\\rotatebox{{90}}{{w{i}}}" if i % 3 == 1 else f"plain{i}")
            for i in range(30)
        ) + " end.",
    ),
}


def main():
    img = os.path.join(HERE, "images")
    png(os.path.join(img, "checker.png"), 80, 40, 72, lambda x, y: (200, 40, 40) if (x < 40) == (y < 20) else (40, 40, 200))
    png(os.path.join(img, "tall.png"), 20, 60, 144, lambda x, y: (30, 150, 60) if y < 30 else (230, 200, 30))
    png(os.path.join(img, "sub", "dot.png"), 12, 12, None, lambda x, y: (0, 0, 0) if (x - 6) ** 2 + (y - 6) ** 2 < 25 else (255, 255, 255))
    pdf(os.path.join(img, "frame.pdf"))
    for name, (opts, preamble, body) in FIXTURES.items():
        open(os.path.join(HERE, name + ".tex"), "w").write(PRE % (opts, preamble) + body + POST)
    print(len(FIXTURES), "fixtures")


if __name__ == "__main__":
    main()
