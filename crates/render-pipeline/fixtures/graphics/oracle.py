#!/usr/bin/env python3
"""Inline-graphics oracle (TEST ONLY; never in the product path).

Runs MacTeX `pdflatex` on every `NN-*.tex` fixture with uncompressed output
(`\\pdfcompresslevel=0 \\pdfobjcompresslevel=0`, which changes no layout)
and interprets the page content streams (q/Q, cm, BT/Tf/Td/Tm/TJ, re f, Do,
form XObjects with /BBox and /Matrix) into `reference/NN-*.json`:

  glyphs: page, text (font code mapped to Unicode), origin x/y and advance
          vector in bp, y down from the page top
  images: page, the unit-square transform [a,b,c,d,e,f] (y down) and, for
          images drawn inside a clipping form (graphicx `clip`), the clip
          quadrilateral as four page points
  rules:  page and the four corners of every filled rectangle

`tests/graphics_oracle.rs` groups glyphs into words the same way on both
sides and compares positions (0.5 bp), image transforms (0.01 bp), clips and
line breaks against these committed files, so cargo never runs TeX.
"""
import json, os, re, shutil, subprocess, sys, tempfile

HERE = os.path.dirname(os.path.abspath(__file__))
PDFLATEX = shutil.which("pdflatex") or "/Library/TeX/texbin/pdflatex"

# OT1 text-font codes that are not ASCII (cmr/cmbx/cmti/cmsl family).
OT1 = {11: "ff", 12: "fi", 13: "fl", 14: "ffi", 15: "ffl", 16: "\u0131", 17: "\u0237", 25: "\u00df", 26: "\u00e6", 27: "\u0153", 28: "\u00f8", 29: "\u00c6", 30: "\u0152", 31: "\u00d8", 34: "\u201d", 92: "\u201c", 123: "\u2013", 124: "\u2014", 60: "\u00a1", 62: "\u00bf"}
CMMI = {11: "\u03b1", 12: "\u03b2", 13: "\u03b3", 14: "\u03b4", 58: ".", 59: ",", 60: "<", 62: ">", 61: "/"}
CMSY = {0: "\u2212", 1: "\u22c5", 2: "\u00d7", 3: "\u2217"}


def decode(font, code):
    base = font.upper()
    if "CMMI" in base:
        return CMMI.get(code, chr(code))
    if "CMSY" in base:
        return CMSY.get(code, chr(code))
    if "CMTT" in base:
        return chr(code)
    return OT1.get(code, chr(code))


# ---------------------------------------------------------------- PDF reading
class Ref:
    def __init__(self, n):
        self.n = n

    def __repr__(self):
        return f"{self.n} R"


class Name(str):
    pass


class Op(str):
    pass


DELIM = b"()<>[]{}/%"
WS = b" \t\r\n\f\0"


def lex(data, pos=0, end=None):
    end = len(data) if end is None else end
    while pos < end:
        c = data[pos]
        if c in WS:
            pos += 1
        elif c == 0x25:  # %
            while pos < end and data[pos] not in b"\r\n":
                pos += 1
        elif c == 0x2F:  # /name
            s = pos + 1
            pos = s
            while pos < end and data[pos] not in WS and data[pos] not in DELIM:
                pos += 1
            yield Name(data[s:pos].decode("latin-1"))
        elif c == 0x28:  # (string)
            depth, pos, out = 1, pos + 1, bytearray()
            while pos < end:
                ch = data[pos]
                if ch == 0x5C:
                    nxt = data[pos + 1]
                    if nxt in b"01234567":
                        j = pos + 1
                        while j < pos + 4 and data[j] in b"01234567":
                            j += 1
                        out.append(int(data[pos + 1:j], 8) & 255)
                        pos = j
                        continue
                    out.append({0x6E: 10, 0x72: 13, 0x74: 9, 0x62: 8, 0x66: 12}.get(nxt, nxt))
                    pos += 2
                    continue
                if ch == 0x28:
                    depth += 1
                elif ch == 0x29:
                    depth -= 1
                    if depth == 0:
                        pos += 1
                        break
                out.append(ch)
                pos += 1
            yield bytes(out)
        elif data[pos:pos + 2] == b"<<":
            pos += 2
            yield Op("<<")
        elif data[pos:pos + 2] == b">>":
            pos += 2
            yield Op(">>")
        elif c == 0x3C:  # <hex>
            e = data.index(b">", pos)
            h = re.sub(rb"\s", b"", data[pos + 1:e])
            pos = e + 1
            yield bytes.fromhex((h + b"0" * (len(h) % 2)).decode())
        elif c in b"[]":
            pos += 1
            yield Op(chr(c))
        else:
            s = pos
            while pos < end and data[pos] not in WS and data[pos] not in DELIM:
                pos += 1
            tok = data[s:pos].decode("latin-1")
            if re.fullmatch(r"[+-]?(\d+\.?\d*|\.\d+)", tok):
                yield float(tok)
            else:
                yield Op(tok)


def parse_objects(tokens):
    """Builds nested values from a token iterator: dicts, arrays, refs."""
    stack = [[]]
    for t in tokens:
        if t == "<<" or t == "[":
            stack.append([t])
        elif t == ">>":
            items = stack.pop()[1:]
            stack[-1].append({items[i]: items[i + 1] for i in range(0, len(items) - 1, 2)})
        elif t == "]":
            array = stack.pop()[1:]
            stack[-1].append(array)
        elif t == "R" and len(stack[-1]) >= 2 and isinstance(stack[-1][-1], float):
            stack[-1].pop()
            n = stack[-1].pop()
            stack[-1].append(Ref(int(n)))
        else:
            stack[-1].append(t)
    return stack[0]


class Pdf:
    def __init__(self, data):
        # Offsets from the classic xref table pdfTeX writes when object
        # streams are off (a regex over the file could match inside binary
        # stream data).
        self.data = data
        self.offsets = {}
        xref = int(re.findall(rb"startxref\s+(\d+)", data)[-1])
        if data[xref:xref + 4] != b"xref":
            raise ValueError("no classic xref table (object streams still on?)")
        table_end = data.index(b"trailer", xref)
        lines = data[xref + 4:table_end].split(b"\n")
        first = count = 0
        for line in lines:
            parts = line.split()
            if len(parts) == 2:
                first, count = int(parts[0]), int(parts[1])
            elif len(parts) == 3 and count:
                if parts[2] == b"n":
                    off = int(parts[0])
                    head = re.match(rb"\s*\d+\s+\d+\s+obj", data[off:off + 40])
                    self.offsets[first] = off + head.end()
                first += 1
                count -= 1
        self.root = int(re.search(rb"/Root\s+(\d+)\s+0\s+R", data[table_end:]).group(1))
        self.cache = {}

    def obj(self, n):
        if isinstance(n, Ref):
            n = n.n
        if n in self.cache:
            return self.cache[n]
        start = self.offsets[n]
        stream_at = self.data.find(b"stream", start)
        end_at = self.data.find(b"endobj", start)
        if stream_at != -1 and stream_at < end_at:
            head = parse_objects(lex(self.data, start, stream_at))[0]
            length = self.resolve(head["Length"])
            s = stream_at + len(b"stream")
            if self.data[s:s + 2] == b"\r\n":
                s += 2
            else:
                s += 1
            value = (head, self.data[s:s + int(length)])
        else:
            value = parse_objects(lex(self.data, start, end_at))[0]
        self.cache[n] = value
        return value

    def resolve(self, v):
        while isinstance(v, Ref):
            v = self.obj(v)
        return v

    def pages(self):
        cat = self.resolve(Ref(self.root))
        out = []

        def walk(node):
            node = self.resolve(node)
            if node.get("Type") == "Pages":
                for k in node["Kids"]:
                    walk(k)
            else:
                out.append(node)
        walk(cat["Pages"])
        return out


def mul(m, n):
    """m then n (PDF row-vector convention: m x n)."""
    a, b, c, d, e, f = m
    A, B, C, D, E, F = n
    return [a * A + b * C, a * B + b * D, c * A + d * C, c * B + d * D, e * A + f * C + E, e * B + f * D + F]


def apply(m, x, y):
    return (m[0] * x + m[2] * y + m[4], m[1] * x + m[3] * y + m[5])


def contents_bytes(pdf, page):
    c = pdf.resolve(page["Contents"])
    if isinstance(c, list):
        return b"\n".join(pdf.resolve(x)[1] for x in c)
    return c[1]


def interpret(pdf, data, resources, ctm, page_no, height, out, clips):
    res = pdf.resolve(resources) or {}
    fonts = pdf.resolve(res.get("Font", {})) or {}
    xobjects = pdf.resolve(res.get("XObject", {})) or {}
    stack = []
    operands = []
    tm = tlm = [1, 0, 0, 1, 0, 0]
    font = None
    size = tc = tw = 0.0
    th = 1.0
    rise = 0.0
    tl = 0.0
    items = parse_objects(lex(data))

    def to_page(x, y):
        px, py = apply(ctm, x, y)
        return (px, height - py)

    def show(s):
        nonlocal tm
        f = pdf.resolve(fonts[font])
        base = str(f.get("BaseFont", ""))
        first = int(f.get("FirstChar", 0))
        widths = pdf.resolve(f.get("Widths", []))
        for code in s:
            w = widths[code - first] if 0 <= code - first < len(widths) else 0
            trm = mul(tm, ctm)
            ox, oy = apply(trm, 0, rise)
            adv = (w / 1000.0 * size + tc + (tw if code == 32 else 0)) * th
            ax, ay = apply(trm, adv, rise)
            out["glyphs"].append({
                "page": page_no, "text": decode(base, code),
                "x": round(ox, 4), "y": round(height - oy, 4),
                "dx": round(ax - ox, 4), "dy": round(-(ay - oy), 4),
            })
            tm = mul([1, 0, 0, 1, adv, 0], tm)

    for t in items:
        if not isinstance(t, Op):
            operands.append(t)
            continue
        o = operands
        if t == "q":
            stack.append(ctm)
        elif t == "Q":
            ctm = stack.pop()
        elif t == "cm":
            ctm = mul([float(v) for v in o[-6:]], ctm)
        elif t == "BT":
            tm = tlm = [1, 0, 0, 1, 0, 0]
        elif t == "Tf":
            font, size = o[-2], float(o[-1])
        elif t == "Tc":
            tc = float(o[-1])
        elif t == "Tw":
            tw = float(o[-1])
        elif t == "Tz":
            th = float(o[-1]) / 100
        elif t == "TL":
            tl = float(o[-1])
        elif t == "Ts":
            rise = float(o[-1])
        elif t in ("Td", "TD"):
            tlm = mul([1, 0, 0, 1, float(o[-2]), float(o[-1])], tlm)
            tm = tlm
            if t == "TD":
                tl = -float(o[-1])
        elif t == "Tm":
            tlm = tm = [float(v) for v in o[-6:]]
        elif t == "T*":
            tlm = mul([1, 0, 0, 1, 0, -tl], tlm)
            tm = tlm
        elif t == "Tj":
            show(o[-1])
        elif t == "TJ":
            for part in o[-1]:
                if isinstance(part, bytes):
                    show(part)
                else:
                    tm = mul([1, 0, 0, 1, -float(part) / 1000.0 * size * th, 0], tm)
        elif t == "w":
            line_width = float(o[-1])
        elif t == "m":
            path = [(float(o[-2]), float(o[-1]))]
        elif t == "l":
            path.append((float(o[-2]), float(o[-1])))
        elif t == "re":
            x, y, w, h = (float(v) for v in o[-4:])
            re_rect = [to_page(x, y), to_page(x + w, y), to_page(x + w, y + h), to_page(x, y + h)]
            out["_pending_rect"] = re_rect
        elif t in ("f", "F", "f*"):
            r = out.pop("_pending_rect", None)
            if r:
                out["rules"].append({"page": page_no, "corners": [[round(a, 4), round(b, 4)] for a, b in r]})
            path = []
        elif t == "S":
            # pdfTeX >= 1.40.21 strokes a rule as its centre line with the
            # rule's thickness as the line width: record the rectangle.
            out.pop("_pending_rect", None)
            if len(path) == 2:
                (x0, y0), (x1, y1) = path
                hw = line_width / 2
                if x0 == x1:
                    xa, xb, ya, yb = x0 - hw, x0 + hw, min(y0, y1), max(y0, y1)
                elif y0 == y1:
                    xa, xb, ya, yb = min(x0, x1), max(x0, x1), y0 - hw, y0 + hw
                else:
                    xa = None
                if xa is not None:
                    r = [to_page(xa, ya), to_page(xb, ya), to_page(xb, yb), to_page(xa, yb)]
                    out["rules"].append({"page": page_no, "corners": [[round(a, 4), round(b, 4)] for a, b in r]})
            path = []
        elif t in ("n", "s", "B", "b"):
            out.pop("_pending_rect", None)
            path = []
        elif t == "Do":
            xo = xobjects[o[-1]]
            head, body = pdf.resolve(xo)
            if head.get("Subtype") == "Image":
                m = ctm
                out["images"].append({
                    "page": page_no,
                    "transform": [round(v, 4) for v in [m[0], -m[1], m[2], -m[3], m[4], height - m[5]]],
                    "clips": [c for c in clips],
                })
            elif head.get("Subtype") == "Form":
                fm = [float(v) for v in head.get("Matrix", [1, 0, 0, 1, 0, 0])]
                inner = mul(fm, ctm)
                bb = [float(v) for v in head["BBox"]]
                corners = [apply(inner, bb[0], bb[1]), apply(inner, bb[2], bb[1]), apply(inner, bb[2], bb[3]), apply(inner, bb[0], bb[3])]
                corners = [[round(a, 4), round(height - b, 4)] for a, b in corners]
                if head.get("Group") is not None or "PTEX.FileName" in head or "PTEX.InfoDict" in head:
                    # An included PDF page: record it as an image whose unit
                    # square is its BBox.
                    unit = mul([bb[2] - bb[0], 0, 0, bb[3] - bb[1], bb[0], bb[1]], inner)
                    out["images"].append({
                        "page": page_no,
                        "transform": [round(v, 4) for v in [unit[0], -unit[1], unit[2], -unit[3], unit[4], height - unit[5]]],
                        "clips": [c for c in clips],
                    })
                else:
                    interpret(pdf, body, head.get("Resources", resources), inner, page_no, height, out, clips + [corners])
        operands = []


def run(name):
    src = os.path.join(HERE, name + ".tex")
    tex = open(src).read()
    probed = "\\pdfcompresslevel=0 \\pdfobjcompresslevel=0\n" + tex
    with tempfile.TemporaryDirectory() as tmp:
        os.symlink(os.path.join(HERE, "images"), os.path.join(tmp, "images"))
        open(os.path.join(tmp, "doc.tex"), "w").write(probed)
        r = subprocess.run([PDFLATEX, "-interaction=batchmode", "-halt-on-error", "doc.tex"], cwd=tmp, capture_output=True)
        if r.returncode != 0:
            sys.exit(f"{name}: pdflatex failed\n" + open(os.path.join(tmp, "doc.log")).read()[-3000:])
        data = open(os.path.join(tmp, "doc.pdf"), "rb").read()
        log = open(os.path.join(tmp, "doc.log")).read()
    pdf = Pdf(data)
    out = {"fixture": name + ".tex", "glyphs": [], "images": [], "rules": []}
    pages = pdf.pages()
    for i, page in enumerate(pages, 1):
        mb = [float(v) for v in pdf.resolve(page.get("MediaBox", [0, 0, 612, 792]))]
        interpret(pdf, contents_bytes(pdf, page), page.get("Resources", {}), [1, 0, 0, 1, 0, 0], i, mb[3], out, [])
    out.pop("_pending_rect", None)
    out["pages"] = len(pages)
    out["engine"] = subprocess.run([PDFLATEX, "--version"], capture_output=True, text=True).stdout.split("\n")[0]
    out["overfull"] = len(re.findall(r"Overfull \\hbox", log))
    os.makedirs(os.path.join(HERE, "reference"), exist_ok=True)
    json.dump(out, open(os.path.join(HERE, "reference", name + ".json"), "w"), indent=1)
    return out


if __name__ == "__main__":
    names = sorted(n[:-4] for n in os.listdir(HERE) if re.match(r"\d\d-.*\.tex$", n))
    if len(sys.argv) > 1:
        names = [n for n in names if any(n.startswith(a) for a in sys.argv[1:])]
    for n in names:
        o = run(n)
        print(n, "pages", o["pages"], "glyphs", len(o["glyphs"]), "images", len(o["images"]), "rules", len(o["rules"]))
