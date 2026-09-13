#!/usr/bin/env python3
"""Writes the float-table fixtures (`NN-*.tex`). Test data only."""
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
BOOK = ("\\begin{tabular}{lrrr}\n    \\toprule\n    Corpus & Keys & Moved & Rate \\\\\n    \\midrule\n"
        "    Homework & 12480 & 41 & 0.33 \\\\\n    Lecture notes & 30112 & 208 & 0.69 \\\\\n    \\bottomrule\n"
        "  \\end{tabular}")
BOOKTABS = "\\usepackage{booktabs}\n"

FIXTURES = {
    "01-caption-above": pre() + BEFORE + "\\begin{table}[h]\n  \\centering\n  \\caption{A caption above the table.}\n  "
    + TAB + "\n\\end{table}" + AFTER,
    "02-caption-below": pre() + BEFORE + "\\begin{table}[h]\n  \\centering\n  " + TAB
    + "\n  \\caption{A caption below the table.}\n\\end{table}" + AFTER,
    "03-booktabs-top": pre(extra=BOOKTABS) + BEFORE
    + "\\begin{table}[t]\n  \\centering\n  \\caption{Page-break stability per keystroke.}\n  \\label{tab:s}\n  "
    + BOOK + "\n\\end{table}" + AFTER,
    "04-figure-tabular": pre() + BEFORE + "\\begin{figure}[h]\n\\centering\n" + TAB
    + "\n\\caption{A tabular inside a figure.}\n\\end{figure}" + AFTER,
    "05-float-page": pre(extra=BOOKTABS) + BEFORE
    + "\\begin{table}[p]\n  \\centering\n  \\caption{First table on the float page.}\n  " + BOOK
    + "\n\\end{table}\n\n\\begin{table}[p]\n  \\centering\n  " + TAB
    + "\n  \\caption{Second table on the float page.}\n\\end{table}" + AFTER,
    "06-figure-paragraph": pre() + BEFORE
    + "\\begin{figure}[h]\nThis figure holds a paragraph of ordinary text instead of a picture, long enough that it "
    "has to wrap onto a second line inside the float box.\n\\caption{A paragraph in a figure.}\n\\end{figure}" + AFTER,
    "07-small-table": pre(extra=BOOKTABS) + BEFORE + "\\begin{table}[h]\n  \\small\n  \\centering\n  \\caption{A small table.}\n  "
    + BOOK + "\n\\end{table}" + AFTER,
    "08-twocolumn-table": pre(size="10pt", opts=",twocolumn", extra=BOOKTABS)
    + "Text before the float.\n\n"
    "\\begin{table}[t]\n  \\centering\n  \\caption{A table in one column.}\n  \\label{tab:two}\n  "
    + BOOK.replace("Lecture notes", "Notes")
    + "\n\\end{table}\n\nText after the float, in a paragraph of its own, also long enough to wrap in the column.\n"
    "\\end{document}\n",
    "09-center-env": pre() + BEFORE + "\\begin{table}[h]\n\\begin{center}\n" + TAB
    + "\n\\end{center}\n\\caption{A center environment above the caption.}\n\\end{table}" + AFTER,
    "10-footnotesize-bottom": pre() + BEFORE
    + "\\begin{table}[b]\n\\footnotesize\n\\centering\n\\begin{tabular}{|l|r|}\\hline Name & Value\\\\\\hline "
    "alpha & 1\\\\ beta & 22\\\\\\hline\\end{tabular}\n\\caption{A footnotesize table at the bottom.}\n\\end{table}" + AFTER,
    "11-vspace": pre() + BEFORE + "\\begin{table}[h]\n\\centering\n\\caption{Caption, then a vspace, then the table.}\n"
    "\\vspace{6pt}\n" + TAB + "\n\\end{table}" + AFTER,
    "12-graphic-and-tabular": pre(extra="\\usepackage[demo]{graphicx}\n") + BEFORE
    + "\\begin{figure}[h]\n\\centering\n\\includegraphics[width=2in,height=1in]{x}\n\n" + TAB
    + "\n\\caption{A graphic above a tabular.}\n\\end{figure}" + AFTER,
    "13-table-star": pre() + BEFORE + "\\begin{table*}[t]\n\\centering\n\\caption{A starred table in one column.}\n"
    + TAB + "\n\\end{table*}" + AFTER,
    "14-long-caption-10pt": pre(size="10pt") + BEFORE + "\\begin{table}[h]\n\\centering\n" + TAB
    + "\n\\caption{A long caption that does not fit on one line of the text width, so it is set as an ordinary "
    "paragraph instead of a centred box.}\n\\end{table}" + AFTER,
}

if __name__ == "__main__":
    for name, text in FIXTURES.items():
        with open(os.path.join(HERE, name + ".tex"), "w", encoding="utf-8") as f:
            f.write(text)
    print(len(FIXTURES), "fixtures")
