import sys
M = (1 << 64) - 1


def gen(target):
    head = "\\documentclass{article}\n\\newcommand{\\proj}{FlashTeX}\n\\begin{document}\n"
    out = [head]
    ln = len(head)
    n = 0
    seed = 0x9E3779B97F4A7C15
    while ln < target:
        seed = (seed * 6364136223846793005 + 1442695040888963407) & M
        p = (seed >> 33) % 4
        if p == 0:
            s = "\\section{Section %d}\n" % n
        elif p == 1:
            s = "Paragraph %d of \\proj{} with inline $x^{%d} + \\alpha$ maths.\n\n" % (n, n)
        elif p == 2:
            s = ("Paragraph %d discusses results in some detail, with enough words to wrap "
                 "across a line and exercise the paragraph breaker properly.\n\n" % n)
        else:
            s = "Displayed: $$\\frac{a_{%d}}{b}$$\n\n" % n
        out.append(s)
        ln += len(s)
        n += 1
    out.append("\\end{document}\n")
    return "".join(out)


sys.stdout.write(gen(500000))
