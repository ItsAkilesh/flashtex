#!/usr/bin/env python3
"""Regenerate crates/makeindex/tests/fixtures/mi-* from the oracle.

Oracle: MacTeX/TeX Live `makeindex` 2.18 (never run by cargo tests).
Each case directory holds its inputs, `case.json` (arguments + ctype) and
the oracle's `expected.ind`, `expected.ilg` and `expected.json`
(exit status, first stderr line).  Nothing is filtered: makeindex writes no
timing lines, and style paths are the `./name` form kpathsea reports.

Usage: python3 tools/gen_fixtures.py [--makeindex PATH] [--texlive-tests DIR]
"""
import argparse, json, os, random, shutil, subprocess, sys, tempfile

HERE = os.path.dirname(os.path.abspath(__file__))
FIX = os.path.join(HERE, "..", "tests", "fixtures")

def ie(key, page, kw=b"\\indexentry"):
    if isinstance(key, str):
        key = key.encode("utf-8")
    if isinstance(page, str):
        page = page.encode()
    return kw + b"{" + key + b"}{" + page + b"}\n"

def idx(*pairs):
    return b"".join(ie(k, p) for k, p in pairs)

CASES = []

def case(name, files, args, ctype="darwin"):
    CASES.append(dict(name=name, files=files, args=args, ctype=ctype))

def B(s):
    return s.encode("utf-8") if isinstance(s, str) else s

# ---------------------------------------------------------------- syntax
case("basic", {"doc.idx": idx(("banana", 3), ("apple", 1), ("cherry", 2), ("apple", 5), ("Banana split", 7))}, ["doc.idx"])
case("subentries", {"doc.idx": idx(("fruit!apple", 1), ("fruit!apple!green", 2), ("fruit!apple!red", 2), ("fruit", 4), ("fruit!banana", 3), ("vegetable!carrot!orange", 9), ("vegetable", 8))}, ["doc.idx"])
case("level-jumps", {"doc.idx": idx(("a!b!c", 1), ("a", 2), ("a!b", 3), ("a!b!d", 4), ("a!e!f", 5), ("b!c", 6), ("b", 7), ("c!d!e", 8))}, ["doc"])
case("actual-at", {"doc.idx": idx(("alpha@$\\alpha$", 1), ("beta@\\textbf{beta}", 2), ("alpha@\\emph{alpha}", 3), ("alpha", 4), ("gamma!delta@$\\delta$", 5), ("gamma@$\\gamma$!eps", 6))}, ["doc.idx"])
case("see-seealso", {"doc.idx": idx(("cat|see{feline}", 1), ("dog|seealso{canine}", 2), ("dog", 3), ("feline", 4), ("cat|see{feline}", 9), ("mouse|see{rodent, small}", 2))}, ["doc.idx"])
case("ranges", {"doc.idx": idx(("topic|(", 2), ("topic", 3), ("topic|)", 7), ("other|(", 1), ("other|)", 1), ("third|(", 4), ("third|)", 5), ("fourth|(", 10), ("fourth|)", 20), ("fourth", 22))}, ["doc.idx"])
case("ranges-encap", {"doc.idx": idx(("key|(textbf", 2), ("key|textit", 3), ("key|)textbf", 6), ("k2|(textbf", 1), ("k2|)", 4), ("k3|(emph", 5), ("k3|)textbf", 9))}, ["doc.idx"])
case("ranges-norange-r", {"doc.idx": idx(("topic|(", 2), ("topic|)", 7), ("x", 1), ("x", 2), ("x", 3), ("x", 5))}, ["-r", "doc.idx"])
case("encap-mixed", {"doc.idx": idx(("term|textbf", 1), ("term", 2), ("term|textbf", 3), ("term|textit", 3), ("term", 4), ("term|hyperpage", 5), ("term|hyperpage", 6), ("term|hyperpage", 7))}, ["doc.idx"])
case("quote-escape", {"doc.idx": idx(('a"!b', 1), ('a"@b', 2), ('a"|b', 3), ('x\\"!y', 4), ('x\\\\"!y', 5), ('""quote', 6), ('q"@x@shown', 7), ('esc\\!lvl!sub', 8), ('"a', 9))}, ["doc.idx"])
case("special-chars", {"doc.idx": idx(("{nested {braces}}", 1), ("$x^2$", 2), ("\\TeX", 3), ("a_b", 4), ("100\\%", 5), ("~tilde", 6), ("#hash", 7), ("&amp", 8), ("{}", 9))}, ["doc.idx"])
case("word-order", {"doc.idx": idx(("New York", 1), ("Newark", 2), ("new", 3), ("New Jersey", 4), ("newt", 5), ("ne w", 6), ("a  b", 7), ("ab", 8))}, ["doc.idx"])
case("letter-order-l", {"doc.idx": idx(("New York", 1), ("Newark", 2), ("new", 3), ("New Jersey", 4), ("newt", 5), ("ne w", 6), ("a  b", 7), ("ab", 8))}, ["-l", "doc.idx"])
case("numbers", {"doc.idx": idx(("1", 1), ("10", 2), ("2", 3), ("007", 4), ("99999999999", 5), ("0", 6), ("12!3", 7), ("12!ab", 8), ("12", 9), ("alpha", 10))}, ["doc.idx"])
case("symbols", {"doc.idx": idx(("!", 1), ("$", 2), ("@@", 3), ("[x", 4), ("~", 5), ("1a", 6), ("-x", 7), ("9z", 8), ("a", 9), ("5", 10), ("`q", 11), ("{b}", 12))}, ["doc.idx"])
case("case", {"doc.idx": idx(("Apple", 1), ("apple", 2), ("APPLE", 3), ("aPPle", 4), ("apples", 5), ("Apple", 6), ("zebra", 7), ("Zebra", 8))}, ["doc.idx"])
case("utf8-keys", {"doc.idx": idx(("Eclair@\\'Eclair", 1), ("éclair", 2), ("Ölmühle@Ölmühle", 3), ("olive", 4), ("Zürich", 5), ("über", 6), ("ecole@école", 7), ("Élan", 8), ("zeta", 9))}, ["doc.idx"])
case("utf8-headings", {"doc.idx": idx(("éclair", 2), ("Élan", 8), ("apple", 1), ("ßtraße", 3), ("ÿes", 4), ("µ", 5), ("zeta", 6)), "h.ist": B('headings_flag 1\nheading_prefix "\\\\head{"\nheading_suffix "}\\n"\n')}, ["-s", "h.ist", "doc.idx"])
case("utf8-headings-C", {"doc.idx": idx(("éclair", 2), ("Élan", 8), ("apple", 1), ("ßtraße", 3), ("ÿes", 4), ("µ", 5), ("zeta", 6)), "h.ist": B('headings_flag 1\nheading_prefix "\\\\head{"\nheading_suffix "}\\n"\n')}, ["-s", "h.ist", "doc.idx"], ctype="C")
case("latin1-bytes", {"doc.idx": b"".join(ie(bytes([c]) + b"x", 1 + c % 7) for c in range(0x80, 0x100, 3)), "h.ist": B('headings_flag -1\n')}, ["-s", "h.ist", "doc.idx"])
case("latin1-bytes-C", {"doc.idx": b"".join(ie(bytes([c]) + b"x", 1 + c % 7) for c in range(0x80, 0x100, 3)), "h.ist": B('headings_flag -1\n')}, ["-s", "h.ist", "doc.idx"], ctype="C")
case("roman-arabic", {"doc.idx": idx(("pref", "iv"), ("pref", "v"), ("pref", "vi"), ("pref", "1"), ("pref", "2"), ("pref", "3"), ("pref", "x"), ("mix", "IV"), ("mix", "ii"), ("mix", "7"), ("mix", "xlii"), ("mix", "mcmxcix"))}, ["doc.idx"])
case("roman-arabic-style", {"doc.idx": idx(("pref", "iv"), ("pref", "v"), ("pref", "vi"), ("pref", "1"), ("pref", "2"), ("pref", "3"), ("pref", "x"), ("mix", "IV"), ("mix", "ii"), ("mix", "7"), ("mix", "V")), "p.ist": B('page_precedence "nrRaA"\n')}, ["-s", "p.ist", "doc.idx"])
case("range-collapse", {"doc.idx": idx(*[("x", p) for p in (1, 2, 3, 5, 6, 8, 10, 11, 12, 13)] + [("y", 4), ("y", 5), ("z", 7), ("z", 7), ("z", 8)])}, ["doc.idx"])
case("range-collapse-r", {"doc.idx": idx(*[("x", p) for p in (1, 2, 3, 5, 6, 8, 10, 11, 12, 13)] + [("y", 4), ("y", 5), ("z", 7), ("z", 7), ("z", 8)])}, ["-r", "doc.idx"])
case("duplicates", {"doc.idx": idx(("a", 1), ("a", 1), ("a", 1), ("b|textbf", 2), ("b|textbf", 2), ("b", 2), ("a", 2), ("a", 1), ("c|(", 3), ("c|(", 3), ("c|)", 5))}, ["doc.idx"])
case("conflict-encaps", {"doc.idx": idx(("t|textbf", 1), ("t|textit", 1), ("t", 1), ("u|see{x}", 2), ("u|seealso{y}", 2))}, ["doc.idx"])
case("composite-pages", {"doc.idx": idx(("c", "1-2"), ("c", "1-3"), ("c", "1-4"), ("c", "2-1"), ("c", "A-1"), ("c", "iv-3"), ("d", "3"), ("d", "1-1"))}, ["doc.idx"])
case("composite-dot", {"doc.idx": idx(("c", "1.2"), ("c", "1.3"), ("c", "1.4"), ("c", "2.1"), ("c", "10.1"), ("d", "1-2")), "c.ist": B('page_compositor "."\n')}, ["-s", "c.ist", "doc.idx"])
case("alpha-pages", {"doc.idx": idx(("p", "a"), ("p", "b"), ("p", "c"), ("p", "A"), ("p", "B"), ("p", "1"), ("p", "i"), ("q", "d"), ("q", "m"), ("q", "e"), ("q", "M"), ("q", "L"))}, ["doc.idx"])
case("headings-positive", {"doc.idx": idx(("apple", 1), ("!bang", 2), ("42", 3), ("Banana", 4), ("berry", 5), ("cherry", 6)), "h.ist": B('headings_flag 1\nheading_prefix "{\\\\bfseries "\nheading_suffix "\\\\hfil}\\\\nopagebreak\\n"\n')}, ["-s", "h.ist", "doc.idx"])
case("headings-negative", {"doc.idx": idx(("apple", 1), ("!bang", 2), ("42", 3), ("Banana", 4), ("berry", 5), ("cherry", 6)), "h.ist": B('headings_flag -1\nheading_prefix "\\n  \\\\item \\\\textbf{"\nheading_suffix "}"\nsymhead_negative "sym"\nnumhead_negative "num"\n')}, ["-s", "h.ist", "doc.idx"])
case("headings-custom-symnum", {"doc.idx": idx(("apple", 1), ("!bang", 2), ("42", 3), ("7", 4)), "h.ist": B('headings_flag 1\nsymhead_positive "Sym-Bols"\nnumhead_positive "Num-Bers"\n')}, ["-s", "h.ist", "doc.idx"])
case("delimiters", {"doc.idx": idx(("a", 1), ("a", 2), ("a", 3), ("a", 7), ("a!b", 4), ("a!b!c", 5), ("a!b!c", 9), ("d|textbf", 1)), "d.ist": B('delim_0 "\\\\dotfill "\ndelim_1 " :: "\ndelim_2 " ; "\ndelim_n "; "\ndelim_r "\\\\,--\\\\,"\ndelim_t "."\nencap_prefix "\\\\idx"\nencap_infix "["\nencap_suffix "]"\n')}, ["-s", "d.ist", "doc.idx"])
case("line-wrap", {"doc.idx": idx(*[("a very long index entry name that goes on and on", p) for p in range(1, 80, 2)] + [("short", p) for p in range(2, 60, 3)]), "w.ist": B('line_max 40\nindent_space "    "\nindent_length 4\n')}, ["-s", "w.ist", "doc.idx"])
case("line-wrap-default", {"doc.idx": idx(*[("wrap entry with many pages", p) for p in range(1, 120, 2)] + [("wrap!sub", p) for p in range(1, 90, 3)])}, ["doc.idx"])
case("suffixes", {"doc.idx": idx(("two", 1), ("two", 2), ("three", 4), ("three", 5), ("three", 6), ("many", 7), ("many", 8), ("many", 9), ("many", 10), ("rng|(", 1), ("rng|)", 2)), "s.ist": B('suffix_2p "f."\nsuffix_3p "ff."\nsuffix_mp "ff."\n')}, ["-s", "s.ist", "doc.idx"])
case("suffix-2p-only", {"doc.idx": idx(("two", 1), ("two", 2), ("three", 4), ("three", 5), ("three", 6), ("many", 7), ("many", 8), ("many", 9), ("many", 10)), "s.ist": B('suffix_2p "\\\\nobreak\\\\hspace{.1em}f."\n')}, ["-s", "s.ist", "doc.idx"])
case("amble-groupskip", {"doc.idx": idx(("a", 1), ("b", 2), ("c", 3), ("1", 4), ("!", 5)), "a.ist": B('preamble "\\\\begin{theindex}\\n\\\\def\\\\x{y}\\n"\npostamble "\\n\\\\end{theindex}\\n%eof\\n"\ngroup_skip "\\n\\n\\\\bigskip\\n"\n')}, ["-s", "a.ist", "doc.idx"])
case("items-custom", {"doc.idx": idx(("a", 1), ("a!b", 2), ("a!b!c", 3), ("d!e!f", 4), ("d!e", 5), ("g!h", 6), ("g!h!i", 7), ("g", 8)), "i.ist": B('item_0 "\\n\\\\I "\nitem_1 "\\n\\\\II "\nitem_2 "\\n\\\\III "\nitem_01 "\\n\\\\Ia "\nitem_x1 "\\\\Ix "\nitem_12 "\\n\\\\IIa "\nitem_x2 "\\\\IIx "\n')}, ["-s", "i.ist", "doc.idx"])
case("page-literal", {"doc.idx": idx(("a", 1), ("b", 2))}, ["-p", "17", "doc.idx"])
case("page-any", {"doc.idx": idx(("a", 1), ("b", 2)), "doc.log": B("This is pdfTeX\n[1] [2] [3\n\n] [14] (./doc.ind) [15")}, ["-p", "any", "doc.idx"])
case("page-odd", {"doc.idx": idx(("a", 1), ("b", 2)), "doc.log": B("[1] [2] [3] [14]\nOutput written")}, ["-p", "odd", "doc.idx"])
case("page-even", {"doc.idx": idx(("a", 1), ("b", 2)), "doc.log": B("[1] [2] [3] [19]\n"), "e.ist": B('setpage_prefix "\\n\\\\setcounter{page}{"\nsetpage_suffix "}\\n\\\\relax\\n"\n')}, ["-s", "e.ist", "-p", "even", "doc.idx"])
case("page-any-nolog-number", {"doc.idx": idx(("a", 1)), "doc.log": B("no page numbers here [x]\n")}, ["-p", "any", "doc.idx"])
case("keyword-args", {"doc.idx": b"\\glossaryentry<a>(b)<1>\n\\glossaryentry<c>(x)<2>\n\\glossaryentry<b<nested>>(x)<3>\n\\indexentry{d}{4}\n", "k.ist": B("keyword \"\\\\glossaryentry\"\narg_open '<'\narg_close '>'\nrange_open '['\nrange_close ']'\nencap '('\n")}, ["-s", "k.ist", "doc.idx"])
case("keyword-args-real", {"doc.idx": b"\\glossaryentry<a|b>(1)\n\\glossaryentry<a>(2)\n\\glossaryentry<c|[>(3)\n\\glossaryentry<c|]>(5)\n\\indexentry{d}{4}\n", "k.ist": B("keyword \"\\\\glossaryentry\"\narg_open '<'\narg_close '>'\narg_open '('\narg_close ')'\nrange_open '['\nrange_close ']'\n")}, ["-s", "k.ist", "doc.idx"])
case("level-actual-encap-chars", {"doc.idx": idx(("a>b", 1), ("a=A>b=B", 2), ("c#textbf", 3), ("d!e|f", 4), ("g>h>i#(", 5), ("g>h>i#)", 7), ("q\\\\>x", 8)), "c.ist": B("level '>'\nactual '='\nencap '#'\nquote '+'\nescape '\\\\'\n")}, ["-s", "c.ist", "doc.idx"])
case("compress-blanks-c", {"doc.idx": idx(("  lots   of    space  ", 1), ("lots of space", 2), ("tab\t\tsep", 3), ("a !  b", 4), ("x@  y  ", 5))}, ["-c", "doc.idx"])
case("crlf", {"doc.idx": idx(("alpha", 1), ("beta", 2), ("alpha", 2)).replace(b"\n", b"\r\n") + b"\\indexentry{lone\rcr}{3}\n"}, ["doc.idx"])
case("empty-idx", {"doc.idx": b""}, ["doc.idx"])
case("all-rejected", {"doc.idx": b"\\foo{a}{1}\n\\indexentry{}{2}\n"}, ["doc.idx"])
case("multi-file", {"a.idx": idx(("from a", 1), ("shared", 2)), "b.idx": idx(("from b", 3), ("shared", 3), ("shared", 4)), "c.idx": b"\\indexentry{bad!}{1}\n"}, ["a.idx", "b.idx", "c.idx"])
case("output-names", {"doc.idx": idx(("a", 1))}, ["-o", "out.ind", "-t", "out.ilg", "doc.idx"])
case("no-extension-input", {"book.idx": idx(("x", 1), ("y", 2))}, ["book"])

# ---------------------------------------------------------------- errors
case("errors-idx", {"doc.idx": b"".join([
    b"\\indexentry{ok}{1}\n",
    b"\\indexentryx{a}{1}\n",
    b"\\indexentry{a}\n",
    b"\\indexentry{a}x{1}\n",
    b"\\indexentry{a}{1}junk\n",
    b"\\indexentry{a!b!c!d}{1}\n",
    b"\\indexentry{!a}{1}\n",
    b"\\indexentry{a!!b}{1}\n",
    b"\\indexentry{a|b|c}{1}\n",
    b"\\indexentry{a@b@c}{1}\n",
    b"\\indexentry{@b}{1}\n",
    b"\\indexentry{a}{1 2}\n",
    b"\\indexentry{a}{?}\n",
    b"\\indexentry{a}{12x}\n",
    b"\\indexentry{a}{ivq}\n",
    b"\\indexentry{a}{1-2-3-4-5-6-7-8-9-10-11}\n",
    b"\\indexentry{multi\nline}{1}\n",
    b"\\indexentry{a}{\n",
    b"\\indexentry{a}{}\n",
    b"\\indexentry{a!@x}{1}\n",
    b"\\indexentry{a!b@}{1}\n",
    b"   \\indexentry  {spaced}   {  3  }\n",
    b"\\indexentry{last}{9}"])}, ["doc.idx"])
case("errors-eof", {"doc.idx": b"\\indexentry{a}{1}\n\\indexentry{b}"}, ["doc.idx"])
case("errors-eof-arg2", {"doc.idx": b"\\indexentry{a}{1}\n\\indexentry{b}{2"}, ["doc.idx"])
case("errors-sty", {"doc.idx": idx(("a", 1), ("b", 2)), "bad.ist": B('% comment line\nfoobar "x"\npreamble x\nquote \'ab\'\nlevel \'\'\nactual \'\\@\'\nline_max -5\nindent_length -1\npage_precedence "rrn"\ndelim_0 %comment\n"x"\nheadings_flag\n1\npostamble')}, ["-s", "bad.ist", "doc.idx"])
case("errors-sty-quote-escape", {"doc.idx": idx(('a"!b', 1)), "q.ist": B("quote '\\\\'\n")}, ["-s", "q.ist", "doc.idx"])
case("errors-sty-precedence", {"doc.idx": idx(("a", "1"), ("a", "i")), "p.ist": B('page_precedence "nxr"\n')}, ["-s", "p.ist", "doc.idx"])
case("errors-sty-unclosed", {"doc.idx": idx(("a", 1)), "u.ist": B('preamble "never closed\n\nstill')}, ["-s", "u.ist", "doc.idx"])
case("range-errors", {"doc.idx": idx(("open|(", 1), ("open", 3), ("close|)", 2), ("types|(", "iv"), ("types|)", "3"), ("incons|(textbf", 1), ("incons|textit", 2), ("incons|)emph", 4), ("nested|(", 1), ("nested|(", 2), ("nested|)", 3), ("nested|)", 4), ("cross|(", "1-2"), ("cross|)", "2-1"), ("tail|(", 8))}, ["doc.idx"])
case("german-g", {"doc.idx": idx(('G+"ote', 1), ("Goethe", 2), ("Gosse", 3), ('Stra+"se', 4), ("Strasse", 5), ('+"Apfel', 6), ("Aepfel", 7), ("1990", 8), ("!x", 9), ("zebra", 10), ("Zebra", 11), ("ab", 12), ("Ab", 13)), "g.ist": B("quote '+'\n")}, ["-g", "-s", "g.ist", "doc.idx"])
case("german-g-fatal", {"doc.idx": idx(("a", 1))}, ["-g", "doc.idx"])
case("german-numbers-groups", {"doc.idx": idx(("alpha", 1), ("2", 2), ("!sym", 3), ("beta", 4), ("10", 5)), "g.ist": B("quote '+'\nheadings_flag 1\n")}, ["-g", "-s", "g.ist", "doc.idx"])
case("roman-heuristic", {"doc.idx": idx(("r", "i"), ("r", "c"), ("r", "d"), ("r", "m"), ("r", "ii"), ("r", "x"), ("r", "l"), ("s", "C"), ("s", "I"), ("s", "D"), ("s", "XL"), ("s", "L"))}, ["doc.idx"])
case("nested-arg-braces", {"doc.idx": idx(("{a!b}!c", 1), ("x{\"}y", 2), ('\\"{o}', 3), ("\\verb|x|", 4), ("{|}", 5), ("a|see{b|c}", 6))}, ["doc.idx"])
case("only-subentries", {"doc.idx": idx(("top!one", 1), ("top!two", 2), ("top!two!deep", 3), ("other!x", 4))}, ["doc.idx"])
case("encap-range-suffix", {"doc.idx": idx(("r|(", 1), ("r", 2), ("r|)", 3), ("s|(textbf", 1), ("s|)textbf", 2), ("t|(", 4), ("t|)", 5)), "s.ist": B('suffix_2p "f."\nsuffix_3p "ff."\n')}, ["-s", "s.ist", "doc.idx"])

# ---------------------------------------------------------------- stress
def rnd_case(seed, n, with_ranges):
    r = random.Random(seed)
    words = ["alpha", "Alpha", "beta", "gamma", "delta", "1", "22", "!x", "$y", "zeta", "Zeta", "eta", "a b", "ab"]
    encs = ["", "", "", "|textbf", "|textit", "|see{x}"]
    out = []
    for _ in range(n):
        k = r.choice(words)
        if r.random() < 0.4:
            k += "!" + r.choice(words)
            if r.random() < 0.3:
                k += "!" + r.choice(words)
        if r.random() < 0.2:
            k += "@" + r.choice(["X", "Y", "\\emph{w}"])
        e = r.choice(encs)
        if with_ranges and r.random() < 0.1:
            e = r.choice(["|(", "|)"])
        p = str(r.randint(1, 40)) if r.random() < 0.85 else r.choice(["i", "ii", "iv", "x", "3-1", "3-2"])
        out.append(ie(k + e, p))
    return b"".join(out)

for s in range(1, 6):
    case(f"random-{s}", {"doc.idx": rnd_case(s, 60 * s, s % 2 == 0)}, ["doc.idx"])
case("random-letter-l", {"doc.idx": rnd_case(77, 300, True)}, ["-l", "doc.idx"])
case("random-r", {"doc.idx": rnd_case(78, 300, True)}, ["-r", "doc.idx"])
case("large-dots", {"doc.idx": rnd_case(99, 2600, True)}, ["doc.idx"])

# ------------------------------------------------ TeX Live regression data
TL = [
    ("tl-sample", ["sample.idx"], []), ("tl-foo", ["foo.idx"], []),
    ("tl-foo-head1", ["foo.idx"], ["head1.ist"]), ("tl-foo-head2", ["foo.idx"], ["head2.ist"]),
    ("tl-tort", ["tort.idx"], []), ("tl-tortW", ["tortW.idx"], []), ("tl-tort-head1", ["tort.idx"], ["head1.ist"]),
    ("tl-nested3", ["nested3.idx"], ["nested3.ist"]), ("tl-toodeep", ["toodeep.idx"], []),
    ("tl-nested-range", ["nested-range.idx"], []), ("tl-nested-range-bb", ["nested-range-bb.idx"], []),
] + [(f"tl-range{i}", ["range.idx"], [f"range{i}.ist"]) for i in (1, 2, 3, 4)] \
  + [(f"tl-pprecA-{i}", ["pprecA.idx"], [f"pprec{i}.ist"]) for i in (0, 1, 2)] \
  + [(f"tl-pprecB-{i}", ["pprecB.idx"], [f"pprec{i}.ist"]) for i in (0, 3, 4)] \
  + [(f"tl-romalp{x}-{i}", [f"romalp{x}.idx"], [f"pprec{i}.ist"]) for x, l in (("A", (5, 6)), ("B", (5, 6, 7)), ("C", (5,)), ("D", (5, 6, 7))) for i in l]

def add_texlive(tdir):
    for name, idxs, ists in TL:
        files = {f: open(os.path.join(tdir, f), "rb").read() for f in idxs + ists}
        args = (["-s", ists[0]] if ists else []) + idxs
        case(name, files, args)

def run_case(c, mk):
    d = os.path.join(FIX, "mi-" + c["name"])
    if os.path.isdir(d):
        shutil.rmtree(d)
    os.makedirs(d)
    for f, data in c["files"].items():
        open(os.path.join(d, f), "wb").write(data)
    json.dump({"args": c["args"], "ctype": c["ctype"]}, open(os.path.join(d, "case.json"), "w"), indent=1)
    env = dict(os.environ)
    env["LC_ALL"] = "C" if c["ctype"] == "C" else "en_US.UTF-8"
    with tempfile.TemporaryDirectory() as t:
        for f, data in c["files"].items():
            open(os.path.join(t, f), "wb").write(data)
        p = subprocess.run([mk, "-q"] + c["args"], cwd=t, env=env, capture_output=True)
        before = set(c["files"])
        produced = sorted(set(os.listdir(t)) - before)
        ind = [f for f in produced if f.endswith(".ind")]
        ilg = [f for f in produced if f.endswith(".ilg")]
        if ind:
            shutil.copy(os.path.join(t, ind[0]), os.path.join(d, "expected.ind"))
        if ilg:
            shutil.copy(os.path.join(t, ilg[0]), os.path.join(d, "expected.ilg"))
        err = p.stderr.decode("latin-1").split("\n")[0] if p.returncode else ""
        json.dump({"exit": p.returncode, "stderr": err, "ind": ind[0] if ind else None, "ilg": ilg[0] if ilg else None},
                  open(os.path.join(d, "expected.json"), "w"), indent=1)
    return p.returncode

def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--makeindex", default=shutil.which("makeindex") or "/Library/TeX/texbin/makeindex")
    ap.add_argument("--texlive-tests", default=None, help="texk/makeindexk/tests directory")
    a = ap.parse_args()
    if a.texlive_tests:
        add_texlive(a.texlive_tests)
    os.makedirs(FIX, exist_ok=True)
    for c in CASES:
        rc = run_case(c, a.makeindex)
        print(f"{c['name']}: exit {rc}")
    print(len(CASES), "cases")

if __name__ == "__main__":
    main()
