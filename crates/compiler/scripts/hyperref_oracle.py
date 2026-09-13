#!/usr/bin/env python3
"""Regenerates crates/compiler/tests/hyperref_oracle/expected/*.json.

Oracle only: runs MacTeX/TeX Live pdflatex (never part of the product path)
on every tests/hyperref_oracle/cases/*.tex with uncompressed objects, then
extracts from the PDF exactly what hyperref wrote:

  pages[].annotations   /Link annotations in /Annots order: /Rect, /Border,
                        /C, /H, the action (/GoTo /D name or /URI), and the
                        text pdftotext finds inside the rectangle
  pages[].words         every word with its box (PDF points, y up)
  pages[].fill_colors   distinct non-black fill operators in content order
  destinations          the /Names /Dests tree: name -> page, view
  outlines              /Outlines in document order: title, level, dest, /Count
  info                  /Title /Author /Subject /Keywords /Creator
  catalog               /PageMode and /OpenAction

`cargo test` never runs TeX; tests/hyperref_oracle.rs reads these files.
Usage: python3 crates/compiler/scripts/hyperref_oracle.py [case-name ...]
"""
import hashlib
import html
import json
import re
import shutil
import subprocess
import sys
import tempfile
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent / "tests" / "hyperref_oracle"
CASES = ROOT / "cases"
EXPECTED = ROOT / "expected"

WS = b" \t\r\n\f\x00"
DELIM = b"()<>[]{}/%"


class Ref:
    def __init__(self, num):
        self.num = num

    def __repr__(self):
        return f"{self.num} 0 R"


class Name(str):
    pass


class Parser:
    def __init__(self, data, pos=0):
        self.d = data
        self.p = pos

    def ws(self):
        while self.p < len(self.d):
            c = self.d[self.p : self.p + 1]
            if c in (b" ", b"\t", b"\r", b"\n", b"\f", b"\x00"):
                self.p += 1
            elif c == b"%":
                while self.p < len(self.d) and self.d[self.p : self.p + 1] not in (b"\r", b"\n"):
                    self.p += 1
            else:
                return

    def value(self):
        self.ws()
        d, p = self.d, self.p
        if d.startswith(b"<<", p):
            self.p += 2
            out = {}
            while True:
                self.ws()
                if self.d.startswith(b">>", self.p):
                    self.p += 2
                    return out
                key = self.value()
                out[str(key)] = self.value()
        if d.startswith(b"<", p):
            end = d.index(b">", p)
            self.p = end + 1
            return bytes.fromhex(d[p + 1 : end].decode().replace(" ", "")).decode("latin-1")
        c = d[p : p + 1]
        if c == b"[":
            self.p += 1
            out = []
            while True:
                self.ws()
                if self.d.startswith(b"]", self.p):
                    self.p += 1
                    return out
                out.append(self.value())
        if c == b"(":
            return self.string()
        if c == b"/":
            self.p += 1
            start = self.p
            while self.p < len(d) and d[self.p] not in WS and d[self.p] not in DELIM:
                self.p += 1
            return Name(d[start : self.p].decode("latin-1"))
        start = self.p
        while self.p < len(d) and d[self.p] not in WS and d[self.p] not in DELIM:
            self.p += 1
        token = d[start : self.p].decode("latin-1")
        if re.fullmatch(r"[+-]?\d+", token):
            save = self.p
            m = re.compile(rb"\s+(\d+)\s+R(?![A-Za-z])").match(d, self.p)
            if m:
                self.p = m.end()
                return Ref(int(token))
            self.p = save
        return token  # numbers stay verbatim strings; true/false/null too

    def string(self):
        d = self.d
        self.p += 1
        depth, out = 1, bytearray()
        while True:
            c = d[self.p]
            self.p += 1
            if c == 0x5C:
                e = d[self.p]
                self.p += 1
                if 0x30 <= e <= 0x37:
                    digits = bytes([e])
                    while len(digits) < 3 and 0x30 <= d[self.p] <= 0x37:
                        digits += bytes([d[self.p]])
                        self.p += 1
                    out.append(int(digits, 8) & 0xFF)
                elif e in (0x0A, 0x0D):
                    continue
                else:
                    out.append({0x6E: 0x0A, 0x72: 0x0D, 0x74: 0x09, 0x62: 0x08, 0x66: 0x0C}.get(e, e))
            elif c == 0x28:
                depth += 1
                out.append(c)
            elif c == 0x29:
                depth -= 1
                if depth == 0:
                    return bytes(out).decode("latin-1")
                out.append(c)
            else:
                out.append(c)


def text_string(s):
    """A PDF text string (latin-1 carried bytes) as Unicode."""
    raw = s.encode("latin-1")
    if raw.startswith(b"\xfe\xff"):
        return raw[2:].decode("utf-16-be")
    return raw.decode("latin-1")


class Pdf:
    def __init__(self, data):
        self.d = data
        self.offsets = {}
        for m in re.finditer(rb"(?:^|[\r\n])(\d+) 0 obj", data):
            self.offsets[int(m.group(1))] = m.end()
        trailer = data.rindex(b"trailer")
        self.trailer = Parser(data, trailer + len(b"trailer")).value()
        self.cache = {}

    def obj(self, num):
        if num not in self.cache:
            parser = Parser(self.d, self.offsets[num])
            value = parser.value()
            parser.ws()
            if isinstance(value, dict) and self.d.startswith(b"stream", parser.p):
                start = parser.p + len(b"stream")
                if self.d[start : start + 2] == b"\r\n":
                    start += 2
                else:
                    start += 1
                length = self.resolve(value["Length"])
                value = (value, self.d[start : start + int(length)])
            self.cache[num] = value
        return self.cache[num]

    def resolve(self, v):
        while isinstance(v, Ref):
            v = self.obj(v.num)
        return v


def page_refs(pdf, node_ref):
    node = pdf.resolve(node_ref)
    if node.get("Type") == "Page":
        return [node_ref]
    out = []
    for kid in node["Kids"]:
        out.extend(page_refs(pdf, kid))
    return out


def plain(v):
    if isinstance(v, Ref):
        return f"{v.num} 0 R"
    if isinstance(v, list):
        return [plain(x) for x in v]
    if isinstance(v, dict):
        return {k: plain(x) for k, x in v.items()}
    return v


def dest_view(pdf, page_index, dest):
    dest = pdf.resolve(dest)
    if isinstance(dest, dict):
        dest = pdf.resolve(dest["D"])
    page = page_index[dest[0].num]
    return {"page": page, "view": [None if x == "null" else str(x) for x in dest[1:]]}


def name_tree(pdf, node, out):
    node = pdf.resolve(node)
    names = node.get("Names", [])
    for i in range(0, len(names), 2):
        out[text_string(names[i])] = names[i + 1]
    for kid in node.get("Kids", []):
        name_tree(pdf, kid, out)


def outline_items(pdf, first, level, out):
    item_ref = first
    while item_ref is not None:
        item = pdf.resolve(item_ref)
        action = pdf.resolve(item.get("A")) if "A" in item else None
        dest = None
        if isinstance(action, dict) and "D" in action:
            dest = text_string(action["D"]) if isinstance(action["D"], str) else plain(action["D"])
        out.append(
            {
                "title": text_string(pdf.resolve(item["Title"])),
                "level": level,
                "destination": dest,
                "count": int(item["Count"]) if "Count" in item else None,
            }
        )
        if "First" in item:
            outline_items(pdf, item["First"], level + 1, out)
        item_ref = item.get("Next")


FILL = re.compile(rb"((?:[-+]?[\d.]+\s+){0,3}[-+]?[\d.]+)\s+(rg|k|g)(?![A-Za-z])")


def fill_colors(content):
    seen = []
    for m in FILL.finditer(content):
        op = f"{m.group(1).decode()} {m.group(2).decode()}"
        numbers = m.group(1).split()
        expected = {"g": 1, "rg": 3, "k": 4}[m.group(2).decode()]
        if len(numbers) < expected:
            continue
        op = " ".join(n.decode() for n in numbers[-expected:]) + " " + m.group(2).decode()
        if op == "0 g":
            continue
        if op not in seen:
            seen.append(op)
    return seen


WORD = re.compile(r'<word xMin="([\d.]+)" yMin="([\d.]+)" xMax="([\d.]+)" yMax="([\d.]+)">(.*?)</word>')


def words(pdf_path, page_number, height):
    out = subprocess.run(
        ["pdftotext", "-bbox", "-f", str(page_number), "-l", str(page_number), str(pdf_path), "-"],
        check=True,
        capture_output=True,
        text=True,
    ).stdout
    result = []
    for m in WORD.finditer(out):
        x0, y0, x1, y1 = (float(m.group(i)) for i in range(1, 5))
        result.append(
            {"text": html.unescape(m.group(5)), "box": [round(x0, 3), round(height - y1, 3), round(x1, 3), round(height - y0, 3)]}
        )
    return result


def text_in(pdf_path, page_number, height, rect):
    x0, y0, x1, y1 = rect
    out = subprocess.run(
        [
            "pdftotext", "-f", str(page_number), "-l", str(page_number),
            "-x", str(int(x0)), "-y", str(int(height - y1)),
            "-W", str(int(x1 - x0 + 1.999)), "-H", str(int(y1 - y0 + 1.999)),
            str(pdf_path), "-",
        ],
        check=True,
        capture_output=True,
        text=True,
    ).stdout
    return " ".join(out.replace("\f", " ").split())


def extract(pdf_path):
    data = pdf_path.read_bytes()
    pdf = Pdf(data)
    catalog = pdf.resolve(pdf.trailer["Root"])
    refs = page_refs(pdf, catalog["Pages"])
    page_index = {r.num: i + 1 for i, r in enumerate(refs)}
    pages = []
    for number, ref in enumerate(refs, start=1):
        page = pdf.resolve(ref)
        height = float(page["MediaBox"][3])
        contents = pdf.resolve(page["Contents"])
        streams = contents if isinstance(contents, list) else [page["Contents"]]
        content = b"".join(pdf.resolve(s)[1] for s in streams)
        annotations = []
        for annot_ref in pdf.resolve(page.get("Annots", [])):
            annot = pdf.resolve(annot_ref)
            if annot.get("Subtype") != "Link":
                continue
            action = pdf.resolve(annot.get("A", {}))
            entry = {
                "rect": [str(x) for x in annot["Rect"]],
                "border": [str(x) for x in annot.get("Border", [])],
                "color": [str(x) for x in annot.get("C", [])],
                "highlight": annot.get("H"),
                "action": action.get("S"),
            }
            if action.get("S") == "GoTo":
                entry["destination"] = text_string(action["D"]) if isinstance(action["D"], str) else plain(action["D"])
            elif action.get("S") == "URI":
                entry["uri"] = action["URI"]
            rect = [float(x) for x in annot["Rect"]]
            entry["text"] = text_in(pdf_path, number, height, rect)
            annotations.append(entry)
        pages.append(
            {
                "number": number,
                "size": [str(x) for x in page["MediaBox"][2:]],
                "annotations": annotations,
                "fill_colors": fill_colors(content),
                "words": words(pdf_path, number, height),
            }
        )
    destinations = {}
    if "Names" in catalog:
        names = pdf.resolve(catalog["Names"])
        if "Dests" in names:
            tree = {}
            name_tree(pdf, names["Dests"], tree)
            destinations = {k: dest_view(pdf, page_index, v) for k, v in sorted(tree.items())}
    outlines = []
    if "Outlines" in catalog:
        root = pdf.resolve(catalog["Outlines"])
        if "First" in root:
            outline_items(pdf, root["First"], 1, outlines)
    info = {}
    if "Info" in pdf.trailer:
        raw = pdf.resolve(pdf.trailer["Info"])
        for key in ("Title", "Author", "Subject", "Keywords", "Creator"):
            if key in raw:
                info[key] = text_string(pdf.resolve(raw[key]))
    open_action = catalog.get("OpenAction")
    if open_action is not None:
        action = pdf.resolve(open_action)
        if isinstance(action, dict) and "D" in action:
            d = pdf.resolve(action["D"])
            open_action = [page_index[d[0].num]] + [str(x) for x in d[1:]]
        else:
            open_action = plain(action)
    return {
        "pages": pages,
        "destinations": destinations,
        "outlines": outlines,
        "info": info,
        "catalog": {"page_mode": catalog.get("PageMode"), "open_action": open_action},
    }


def banner():
    first = subprocess.run(["pdflatex", "--version"], capture_output=True, text=True).stdout.splitlines()
    return first[0] if first else "unknown"


def hyperref_version():
    path = subprocess.run(["kpsewhich", "hyperref.sty"], capture_output=True, text=True).stdout.strip()
    for line in Path(path).read_text(encoding="latin-1").splitlines():
        if "ProvidesPackage{hyperref}" in line or line.startswith("  [") and "hyperref" in line.lower():
            return line.strip()
    return path


def run(case):
    source = case.read_text(encoding="utf-8")
    with tempfile.TemporaryDirectory() as tmp:
        tmp = Path(tmp)
        shutil.copy(case, tmp / case.name)
        job = case.stem
        command = [
            "pdflatex", "-interaction=nonstopmode", "-halt-on-error", f"-jobname={job}",
            r"\pdfcompresslevel=0\pdfobjcompresslevel=0\input{" + case.name + "}",
        ]
        for _ in range(3):
            done = subprocess.run(command, cwd=tmp, capture_output=True, text=True)
            if done.returncode != 0:
                sys.exit(f"{case.name}: pdflatex failed\n{done.stdout[-2000:]}")
        log = (tmp / f"{job}.log").read_text(encoding="latin-1")
        result = extract(tmp / f"{job}.pdf")
    result = {
        "case": case.name,
        "source_sha256": hashlib.sha256(source.encode("utf-8")).hexdigest(),
        "oracle": banner(),
        "hyperref": hyperref_version(),
        "warnings": sorted(set(re.findall(r"(?:Package hyperref|LaTeX) Warning: ([^\n]*)", log))),
        **result,
    }
    EXPECTED.mkdir(parents=True, exist_ok=True)
    (EXPECTED / f"{case.stem}.json").write_text(json.dumps(result, indent=1, ensure_ascii=False) + "\n", encoding="utf-8")
    print(f"{case.name}: {sum(len(p['annotations']) for p in result['pages'])} links, "
          f"{len(result['destinations'])} destinations, {len(result['outlines'])} outline items")


def main():
    wanted = set(sys.argv[1:])
    cases = sorted(CASES.glob("*.tex"))
    for case in cases:
        if not wanted or case.stem in wanted:
            run(case)


if __name__ == "__main__":
    main()
