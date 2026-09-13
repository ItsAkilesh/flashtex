#!/usr/bin/env python3
"""Writes the pseudocode fixtures `NN-*.tex` (run `oracle.py` afterwards).

Each exercises one construct of algorithm.sty, algorithmic.sty or
algpseudocode.sty against pdflatex; see `tests/algorithms_oracle.rs`.
"""
import os

HERE = os.path.dirname(os.path.abspath(__file__))


def doc(packages, body, cls="article"):
    return f"\\documentclass{{{cls}}}\n{packages}\n\\begin{{document}}\n{body}\n\\end{{document}}\n"


ALGORITHMIC = "\\usepackage{algorithm}\n\\usepackage{algorithmic}"
ALGPSEUDO = "\\usepackage{algorithm}\n\\usepackage{algpseudocode}"

FIXTURES = {
    "01-algorithmic-basic": doc(ALGORITHMIC, r"""Text before the algorithm.

\begin{algorithm}[h]
\caption{Euclid's algorithm}\label{alg:euclid}
\begin{algorithmic}[1]
\STATE $x \gets a$
\IF{$a > b$}
\STATE swap the values
\ELSE
\RETURN $b$
\ENDIF
\end{algorithmic}
\end{algorithm}

After the algorithm."""),
    "02-algorithmic-loops": doc(ALGORITHMIC, r"""Loops follow.

\begin{algorithm}[h]
\caption{Loops}
\begin{algorithmic}[1]
\FOR{$i = 1$ \TO $n$}
\STATE $s \gets s + i$
\ENDFOR
\FORALL{$v \in V$}
\STATE visit $v$
\ENDFOR
\WHILE{$x > 0$}
\STATE $x \gets x - 1$
\ENDWHILE
\REPEAT
\STATE $y \gets y / 2$
\UNTIL{$y < 1$}
\LOOP
\STATE wait
\ENDLOOP
\end{algorithmic}
\end{algorithm}

Done."""),
    "03-algorithmic-require-ensure": doc(ALGORITHMIC, r"""Pre- and postconditions, unnumbered.

\begin{algorithm}[h]
\caption{Conditions}
\begin{algorithmic}
\REQUIRE $n \geq 0$
\ENSURE $y = x^n$
\STATE $y \gets 1$
\PRINT $y$
\end{algorithmic}
\end{algorithm}

End."""),
    "04-algorithmic-nested": doc(ALGORITHMIC, r"""Three levels.

\begin{algorithm}[h]
\caption{Nesting}
\begin{algorithmic}[1]
\WHILE{\TRUE}
\FOR{$i \gets 1$ \TO $n$}
\IF{$a_i > m$}
\STATE $m \gets a_i$
\ELSIF{$a_i = m$}
\STATE count it
\ENDIF
\ENDFOR
\ENDWHILE
\end{algorithmic}
\end{algorithm}

End."""),
    "05-algorithmic-comments": doc(ALGORITHMIC, r"""Comments.

\begin{algorithm}[h]
\caption{Comments}
\begin{algorithmic}[1]
\STATE $i \gets 0$ \COMMENT{start at zero}
\IF[check the bound]{$i < n$}
\STATE $i \gets i + 1$
\ENDIF
\end{algorithmic}
\end{algorithm}

End."""),
    "06-algorithmic-every-second": doc(ALGORITHMIC, r"""Every second line is numbered.

\begin{algorithm}[h]
\caption{Frequency}
\begin{algorithmic}[2]
\STATE one
\STATE two
\STATE three
\STATE four
\STATE five
\STATE six
\STATE seven
\STATE eight
\STATE nine
\STATE ten
\STATE eleven
\STATE twelve
\end{algorithmic}
\end{algorithm}

End."""),
    "07-algorithmic-noend": doc("\\usepackage{algorithm}\n\\usepackage[noend]{algorithmic}", r"""No end lines.

\begin{algorithm}[h]
\caption{Without end}
\begin{algorithmic}[1]
\FOR{$i = 1$ \TO $n$}
\IF{$i$ is odd}
\STATE print $i$
\ENDIF
\ENDFOR
\STATE stop
\end{algorithmic}
\end{algorithm}

End."""),
    "08-algorithmic-wrap": doc(ALGORITHMIC, r"""Long statements.

\begin{algorithm}[h]
\caption{Wrapping}
\begin{algorithmic}[1]
\FOR{each vertex}
\IF{the vertex has not been visited yet}
\STATE mark the vertex as visited and push every one of its neighbours onto the stack so that they are explored later in depth-first order
\ENDIF
\ENDFOR
\end{algorithmic}
\end{algorithm}

End."""),
    "09-algpseudocode-basic": doc(ALGPSEUDO, r"""Text before the algorithm.

\begin{algorithm}[h]
\caption{Euclid's algorithm}\label{euclid}
\begin{algorithmic}[1]
\State $r\gets a\bmod b$
\If{$a > b$}
\State swap
\Else
\State nothing
\EndIf
\State \Return $b$
\end{algorithmic}
\end{algorithm}

After the algorithm."""),
    "10-algpseudocode-loops": doc(ALGPSEUDO, r"""Loops follow.

\begin{algorithm}[h]
\caption{Loops}
\begin{algorithmic}[1]
\For{$i = 1, \dots, n$}
\State $s \gets s + i$
\EndFor
\ForAll{$v \in V$}
\State visit $v$
\EndFor
\While{$x > 0$}
\State $x \gets x - 1$
\EndWhile
\Repeat
\State $y \gets y / 2$
\Until{$y < 1$}
\Loop
\State wait
\EndLoop
\end{algorithmic}
\end{algorithm}

Done."""),
    "11-algpseudocode-nested": doc(ALGPSEUDO, r"""Three levels.

\begin{algorithm}[h]
\caption{Nesting}
\begin{algorithmic}[1]
\While{$x \neq 0$}
\For{$i \gets 1, n$}
\If{$a_i > m$}
\State $m \gets a_i$
\ElsIf{$a_i = m$}
\State count it
\EndIf
\EndFor
\EndWhile
\end{algorithmic}
\end{algorithm}

End."""),
    "12-algpseudocode-procedures": doc(ALGPSEUDO, r"""Procedures and functions.

\begin{algorithm}[h]
\caption{Procedures}
\begin{algorithmic}[1]
\Require $a, b \ge 0$
\Ensure the greatest common divisor
\Procedure{Euclid}{$a,b$}
\State $r \gets$ \Call{Mod}{$a, b$}
\State \Return $r$
\EndProcedure
\Function{Mod}{$a,b$}
\State \Return $a \bmod b$
\EndFunction
\Procedure{Main}{}
\State \Call{Euclid}{$6, 4$}
\EndProcedure
\end{algorithmic}
\end{algorithm}

End."""),
    "13-algpseudocode-comments": doc(ALGPSEUDO, r"""Comments.

\begin{algorithm}[h]
\caption{Comments}
\begin{algorithmic}[1]
\State $i \gets 0$ \Comment{start at zero}
\While{$i < n$}\Comment{We have the answer if $i$ is $n$}
\State $i \gets i + 1$
\EndWhile
\end{algorithmic}
\end{algorithm}

End."""),
    "14-algpseudocode-noend": doc("\\usepackage{algorithm}\n\\usepackage[noend]{algpseudocode}", r"""No end lines.

\begin{algorithm}[h]
\caption{Without end}
\begin{algorithmic}[1]
\For{$i = 1, n$}
\If{$i$ is odd}
\State print $i$
\EndIf
\EndFor
\State stop
\end{algorithmic}
\end{algorithm}

End."""),
    "15-algpseudocode-wrap": doc(ALGPSEUDO, r"""Long statements.

\begin{algorithm}[h]
\caption{Wrapping}
\begin{algorithmic}[1]
\For{each vertex}
\If{the vertex has not been visited yet}
\State mark the vertex as visited and push every one of its neighbours onto the stack so that they are explored later in depth-first order
\EndIf
\EndFor
\end{algorithmic}
\end{algorithm}

End."""),
    "16-algpseudocode-statex": doc(ALGPSEUDO, r"""Unnumbered lines.

\begin{algorithm}[h]
\caption{Statex}
\begin{algorithmic}[1]
\State first
\Statex a line without a number
\State second
\Statex
\State third
\end{algorithmic}
\end{algorithm}

End."""),
    "17-algorithm-ref": doc(ALGPSEUDO, r"""We first show Algorithm~\ref{alg:a} and then Algorithm~\ref{alg:b}.

\begin{algorithm}[h]
\caption{First}\label{alg:a}
\begin{algorithmic}
\State one
\end{algorithmic}
\end{algorithm}

Between the two algorithms.

\begin{algorithm}[h]
\caption{Second}\label{alg:b}
\begin{algorithmic}
\State two
\end{algorithmic}
\end{algorithm}

Both were referenced: \ref{alg:a} and \ref{alg:b}."""),
    "18-algorithm-top": doc(ALGPSEUDO, r"""The float goes to the top of the page. Lorem ipsum dolor sit amet,
consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et
dolore magna aliqua.

\begin{algorithm}[t]
\caption{At the top}
\begin{algorithmic}[1]
\State $x \gets 1$
\State $y \gets 2$
\end{algorithmic}
\end{algorithm}

Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris nisi ut
aliquip ex ea commodo consequat."""),
    "19-algorithm-renames": doc("\\usepackage{algorithm}\n\\usepackage{algpseudocode}\n\\renewcommand{\\algorithmicrequire}{\\textbf{Input:}}\n\\algrenewcommand\\algorithmicensure{\\textbf{Output:}}", r"""Renamed keywords.

\begin{algorithm}[h]
\caption{Renames}
\begin{algorithmic}[1]
\Require a graph $G$
\Ensure a spanning tree
\State build it
\end{algorithmic}
\end{algorithm}

End."""),
    "20-algorithmic-bare": doc("\\usepackage{algorithmic}", r"""An algorithmic environment in the text, without a float.
\begin{algorithmic}[1]
\STATE $x \gets 0$
\WHILE{$x < 3$}
\STATE $x \gets x + 1$
\ENDWHILE
\end{algorithmic}
And the paragraph continues after it.

A new paragraph."""),
    "21-algpseudocode-bare": doc("\\usepackage{algpseudocode}", r"""Pseudocode in the text.

\begin{algorithmic}[1]
\Procedure{Count}{$n$}
\For{$i \gets 1, n$}
\State $c \gets c + 1$
\EndFor
\EndProcedure
\end{algorithmic}

A new paragraph."""),
    "22-algorithm-plain-style": doc("\\usepackage[plain]{algorithm}\n\\usepackage{algorithmic}", r"""Plain float style: the caption goes below.

\begin{algorithm}[h]
\begin{algorithmic}[1]
\STATE $x \gets 1$
\STATE $y \gets 2$
\end{algorithmic}
\caption{Plain style}
\end{algorithm}

End."""),
}


def main():
    for name, text in FIXTURES.items():
        with open(os.path.join(HERE, name + ".tex"), "w") as f:
            f.write(text)
    print(f"wrote {len(FIXTURES)} fixtures")


if __name__ == "__main__":
    main()
