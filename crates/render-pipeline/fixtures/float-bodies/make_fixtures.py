#!/usr/bin/env python3
"""Writes the float-body fixtures (`NN-*.tex`): lists, display math, headings,
TikZ, side-by-side graphics and minipages inside floats, and `figure*`/`table*`
in two-column documents. Test data only."""
import os

HERE = os.path.dirname(os.path.abspath(__file__))


def pre(size="12pt", extra="", opts=""):
    return (f"\\documentclass[{size}{opts}]{{article}}\n\\usepackage[T1]{{fontenc}}\n\\usepackage{{lmodern}}\n"
            f"\\usepackage[margin=1in]{{geometry}}\n{extra}\\setlength{{\\parindent}}{{0pt}}\n\\pagestyle{{empty}}\n"
            "\\begin{document}\n")


BEFORE = "Text before the float in its own paragraph.\n\n"
AFTER = "\n\nText after the float, in a paragraph of its own.\n\\end{document}\n"
TAB = ("\\begin{tabular}{lcr}\\hline Left & Centre & Right\\\\\\hline one & two & three\\\\ "
       "four & five & six\\\\\\hline\\end{tabular}")
DEMO = "\\usepackage[demo]{graphicx}\n"
AMS = "\\usepackage{amsmath}\n"
TIKZ = "\\usepackage{tikz}\n"

# Filler for the two-column documents: words of at most four letters, so no
# word can be hyphenated (\lefthyphenmin 2 + \righthyphenmin 3).
WORDS = ("the cat and dog ran up a big hill to see the sun set on a warm day so we all sat by the old oak "
         "tree for a nap and then went back home in the dark with our hats on").split()


def filler(paragraphs, words_per=60, seed=0):
    out = []
    k = seed
    for p in range(paragraphs):
        ws = []
        for i in range(words_per):
            w = WORDS[k % len(WORDS)]
            k += 7 if i % 3 else 5
            if i % 12 == 0:
                w = w.capitalize()
            ws.append(w + ("." if i % 12 == 11 or i + 1 == words_per else ""))
        out.append(" ".join(ws))
    return "\n\n".join(out)


def two(extra="", size="10pt"):
    return pre(size=size, opts=",twocolumn", extra=extra)


FIG_STAR = ("\\begin{figure*}[t]\n\\centering\n\\includegraphics[width=5in,height=1.5in]{wide}\n"
            "\\caption{A figure across both columns.}\n\\end{figure*}\n\n")
END = "\n\\end{document}\n"

FIXTURES = {
    "01-itemize-in-figure": pre() + BEFORE
    + "\\begin{figure}[h]\n\\begin{itemize}\n\\item First point of the figure.\n\\item Second point, long enough "
    "to wrap onto a second line inside the float box for sure here.\n\\end{itemize}\n"
    "\\caption{A list in a figure.}\n\\end{figure}" + AFTER,
    "02-enumerate-top-table": pre() + BEFORE
    + "\\begin{table}[t]\n\\begin{enumerate}\n\\item One step.\n\\item Two steps.\n\\item Three steps.\n"
    "\\end{enumerate}\n\\caption{An enumerate at the top of a table.}\n\\end{table}" + AFTER,
    "03-description-after-text": pre(size="10pt") + BEFORE
    + "\\begin{figure}[h]\nA paragraph of text opens this figure.\n\\begin{description}\n\\item[Alpha] the first "
    "entry.\n\\item[Beta] the second entry.\n\\end{description}\n\\caption{A description after text.}\n\\end{figure}"
    + AFTER,
    "04-equation-in-figure": pre() + BEFORE
    + "\\begin{figure}[h]\n\\centering\n\\begin{equation}\nE = mc^2\n\\end{equation}\n"
    "\\caption{An equation in a figure.}\n\\end{figure}" + AFTER,
    "05-align-in-figure": pre(extra=AMS) + BEFORE
    + "\\begin{figure}[h]\n\\begin{align*}\na &= b + c\\\\\nd &= e - f\n\\end{align*}\n"
    "\\caption{Aligned equations.}\n\\end{figure}" + AFTER,
    "06-text-display-text": pre(size="10pt") + BEFORE
    + "\\begin{figure}[h]\nSome text before the display\n\\[ x^2 + y^2 = z^2 \\]\nand text after it.\n"
    "\\caption{Text around a display.}\n\\end{figure}" + AFTER,
    "07-heading-in-figure": pre() + BEFORE
    + "\\begin{figure}[h]\n\\subsection*{Inside}\nText under the heading.\n"
    "\\caption{A heading in a figure.}\n\\end{figure}" + AFTER,
    "08-tikz-in-figure": pre(extra=TIKZ) + BEFORE
    + "\\begin{figure}[h]\n\\centering\n\\begin{tikzpicture}\n\\fill (0,0) rectangle (2,1);\n"
    "\\draw (0,-0.5) -- (3,-0.5);\n\\end{tikzpicture}\n\\caption{A TikZ picture.}\n\\end{figure}" + AFTER,
    "09-graphics-hfill": pre(extra=DEMO) + BEFORE
    + "\\begin{figure}[h]\n\\includegraphics[width=2in,height=1in]{a}\\hfill\\includegraphics[width=2in,height=1in]{b}\n"
    "\\caption{Two graphics apart.}\n\\end{figure}" + AFTER,
    "10-graphics-quad": pre(extra=DEMO) + BEFORE
    + "\\begin{figure}[h]\n\\centering\n\\includegraphics[width=1.5in,height=1in]{a}\\quad\n"
    "\\includegraphics[width=1.5in,height=0.75in]{b}\n\\caption{Two graphics a quad apart.}\n\\end{figure}" + AFTER,
    "11-graphics-space": pre(extra=DEMO) + BEFORE
    + "\\begin{figure}[h]\n\\centering\n\\includegraphics[width=2in,height=1in]{a}\n"
    "\\includegraphics[width=2in,height=1in]{b}\n\\caption{Two graphics a space apart.}\n\\end{figure}" + AFTER,
    "12-minipages-captions": pre(extra=DEMO) + BEFORE
    + "\\begin{figure}[h]\n\\begin{minipage}{0.45\\linewidth}\n\\centering\n"
    "\\includegraphics[width=\\linewidth,height=1in]{a}\n\\caption{Left.}\n\\end{minipage}\\hfill\n"
    "\\begin{minipage}{0.45\\linewidth}\n\\centering\n\\includegraphics[width=\\linewidth,height=1.5in]{b}\n"
    "\\caption{Right, with a caption long enough to wrap.}\n\\end{minipage}\n\\end{figure}" + AFTER,
    "13-minipage-top-text": pre() + BEFORE
    + "\\begin{table}[h]\n\\begin{minipage}[t]{0.4\\linewidth}\nText in a minipage on the left side of the "
    "table, set as a paragraph.\n\\end{minipage}\\hfill\n\\begin{minipage}[t]{0.5\\linewidth}\n\\centering\n"
    + TAB + "\n\\end{minipage}\n\\caption{A paragraph beside a tabular.}\n\\end{table}" + AFTER,
    "14-figstar-page-two": two(DEMO) + filler(3) + "\n\n" + FIG_STAR + filler(22, seed=3) + END,
    "15-figstar-then-figure": two(DEMO) + filler(2) + "\n\n" + FIG_STAR
    + "\\begin{figure}[t]\n\\centering\n\\includegraphics[width=2in,height=1in]{narrow}\n"
    "\\caption{A column figure after the wide one.}\n\\end{figure}\n\n" + filler(22, seed=5) + END,
    "16-tablestar-then-figure": two(DEMO) + filler(2)
    + "\n\n\\begin{table*}[t]\n\\centering\n" + TAB + "\n\\caption{A table across both columns.}\n\\end{table*}\n\n"
    "\\begin{figure}[t]\n\\centering\n\\includegraphics[width=2in,height=1in]{narrow}\n"
    "\\caption{A column figure.}\n\\end{figure}\n\n" + filler(22, seed=9) + END,
    "17-two-figstars": two(DEMO) + filler(2) + "\n\n" + FIG_STAR
    + "\\begin{figure*}[t]\n\\centering\n\\includegraphics[width=6in,height=1in]{wide2}\n"
    "\\caption{A second wide figure.}\n\\end{figure*}\n\n" + filler(22, seed=11) + END,
    "18-dblfloatpage": two(DEMO) + filler(2)
    + "\n\n\\begin{figure*}[p]\n\\centering\n\\includegraphics[width=6in,height=5in]{big}\n"
    "\\caption{A wide figure on a page of its own.}\n\\end{figure*}\n\n" + filler(20, seed=13) + END,
    "19-figstar-end-flush": two(DEMO) + filler(2) + "\n\n" + FIG_STAR + filler(2, seed=15) + END,
    "20-tablestar-list-then-table": two() + filler(2)
    + "\n\n\\begin{table*}[t]\n\\caption{A wide table with a list.}\n\\begin{itemize}\n\\item Rows are kept in "
    "order.\n\\item Columns span the page.\n\\end{itemize}\n\\end{table*}\n\n"
    "\\begin{table}[t]\n\\centering\n" + TAB + "\n\\caption{A column table.}\n\\end{table}\n\n"
    + filler(22, seed=17) + END,
}

if __name__ == "__main__":
    for name, text in FIXTURES.items():
        with open(os.path.join(HERE, name + ".tex"), "w", encoding="utf-8") as f:
            f.write(text)
    print(len(FIXTURES), "fixtures")
