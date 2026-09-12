import json, subprocess, sys

import os
BIN = os.environ.get("FLASHTEX_COMPILER", "crates/compiler/target/debug/flashtex-compiler")

S = [
 # from tests/grok-corpus expected outputs
 ("corpus-integral-unicode", r"\[ ∫_0^π \sin^2θ\,dθ = \frac{π}{2} \]"),
 ("corpus-det-pmatrix", r"\[ \det\begin{pmatrix} 1 & 2 & 3 \\ 0 & 1 & 4 \\ 5 & 6 & 0 \end{pmatrix} = 1(0-24) - 2(0-20) + 3(0-5) = 1 \]"),
 ("corpus-prose-math", r"Problem 3. A particle moves along $y = x^3 - 3x$ for $t \geq 0$ where $y'(x) = 0$ and $y''(x)$."),
 ("corpus-align-star", "\\begin{align*}\n2x + 3 &= 11 \\\\\n2x &= 8 \\\\\nx &= 4\n\\end{align*}"),
 ("corpus-cfrac-nested", r"\[ x^{y^{z}} + \cfrac{1}{2+\cfrac{1}{3+\cfrac{1}{4}}} \qquad a_{i_{j_{k}}}^{2} \]"),
 ("corpus-quadratic", r"\[ x = \frac{-b \pm \sqrt{b^2-4ac}}{2a} \]"),
 ("corpus-problem-lim", r"\textbf{Problem 2.} Evaluate $\displaystyle\lim_{x\to 0} \frac{\sin x}{x}$. \[ \lim_{x\to 0} \frac{\sin x}{x} = 1 \]"),
 ("corpus-bigskip", "\\textbf{Problem 1.} Solve $3x - 7 = 2x + 5$.\n\n\\bigskip\nNext."),
 # common homework forms
 ("lim", r"\[ \lim_{x \to 0} \frac{1-\cos x}{x^2} = \frac{1}{2} \]"),
 ("lim-inf", r"\[ \lim_{n\to\infty} \left(1+\frac{1}{n}\right)^n = e \]"),
 ("derivative-d-dx", r"\[ \frac{d}{dx} x^2 = 2x, \quad \frac{dy}{dx} = 3x^2 \]"),
 ("partial", r"\[ \frac{\partial f}{\partial x} + \frac{\partial^2 f}{\partial y^2} = 0 \]"),
 ("vec-hat-bar", r"$\vec{v} = 3\hat{i} + \bar{x}$"),
 ("mathbf", r"$\mathbf{F} = m\mathbf{a}$"),
 ("mathrm", r"$E = mc^2 \, \mathrm{J}$, $\mathrm{d}x$"),
 ("operatorname", r"$\operatorname{rank}(A) = \operatorname{tr}(B)$"),
 ("left-right-big", r"\[ \left( \frac{a}{b} \right)^2 = \big( x \big) \Big[ y \Big] \bigg\{ z \bigg\} \]"),
 ("binary-ops", r"$a \cdot b \times c \pm d \mp e \div f$"),
 ("relations", r"$a \leq b \geq c \neq d \approx e \equiv f \sim g \propto h \le i \ge j$"),
 ("greek-lower", r"$\alpha\beta\gamma\delta\epsilon\varepsilon\zeta\eta\theta\vartheta\iota\kappa\lambda\mu\nu\xi\pi\rho\sigma\tau\upsilon\phi\varphi\chi\psi\omega$"),
 ("greek-upper", r"$\Gamma\Delta\Theta\Lambda\Xi\Pi\Sigma\Upsilon\Phi\Psi\Omega$"),
 ("functions", r"$\sin x + \cos x + \tan x + \log x + \ln x + \exp(x) + \sec x + \arctan x$"),
 ("binom", r"\[ \binom{n}{k} = \frac{n!}{k!(n-k)!} \]"),
 ("sqrt-n", r"$\sqrt[3]{8} = 2$"),
 ("dots", r"$1, 2, \dots, n$ and $a_1 + \cdots + a_n$ and $x_1, \ldots, x_n$ $\vdots \ddots$"),
 ("text-in-math", r"\[ f(x) = 0 \text{ for all } x \]"),
 ("multline", "\\begin{multline*}\na + b + c \\\\\n+ d + e = f\n\\end{multline*}"),
 ("split", "\\begin{equation*}\n\\begin{split}\na &= b + c \\\\\n&= d\n\\end{split}\n\\end{equation*}"),
 ("alignat", "\\begin{alignat*}{2}\nx &= 1 &\\quad y &= 2 \\\\\nz &= 3 & w &= 4\n\\end{alignat*}"),
 ("tag", "\\begin{equation*}\nE = mc^2 \\tag{1}\n\\end{equation*}"),
 ("boxed", r"\[ \boxed{x = 4} \]"),
 ("tabular", "\\begin{tabular}{|c|c|}\n\\hline\n$x$ & $f(x)$ \\\\\n\\hline\n1 & 2 \\\\\n\\hline\n\\end{tabular}"),
 ("sum-limits", r"\[ \sum_{i=1}^{n} i = \frac{n(n+1)}{2}, \quad \prod_{k=1}^{n} k = n! \]"),
 ("int-limits", r"\[ \int_{0}^{1} x^2 \, dx = \frac{1}{3} \]"),
 ("iint-oint", r"\[ \iint_D f \, dA = \oint_C F \cdot dr, \quad \iiint_V dV \]"),
 ("nabla-infty", r"$\nabla \cdot \mathbf{E} = \rho, \quad \infty$"),
 ("set-braces", r"$S = \{ x \mid x > 0 \}$"),
 ("set-ops", r"$A \cup B \cap C \subset D \subseteq E \supset F$, $x \in A$, $y \notin B$, $\emptyset$"),
 ("quantifiers", r"$\forall \epsilon > 0 \; \exists \delta > 0$"),
 ("arrows", r"$f: A \to B$, $x \mapsto x^2$, $p \Rightarrow q$, $p \iff q$, $a \rightarrow b \leftarrow c \Leftarrow d \leftrightarrow e$"),
 ("cases", r"\[ f(x) = \begin{cases} x & x \geq 0 \\ -x & x < 0 \end{cases} \]"),
 ("prime-deriv", r"$f'(x) = 2x$, $f''(x) = 2$"),
 ("abs-norm", r"$|x| + \|v\| + \lvert y \rvert + \lVert w \rVert$"),
 ("floor-ceil-angle", r"$\lfloor x \rfloor + \lceil y \rceil + \langle u, v \rangle$"),
 ("equation-numbered", "\\begin{equation}\nF = ma\n\\end{equation}"),
 ("align-numbered", "\\begin{align}\na &= b \\\\\nc &= d\n\\end{align}"),
 ("gather", "\\begin{gather*}\na = b \\\\\nc = d\n\\end{gather*}"),
 ("bmatrix", r"\[ A = \begin{bmatrix} 1 & 0 \\ 0 & 1 \end{bmatrix}, \quad \begin{vmatrix} a & b \\ c & d \end{vmatrix} \]"),
 ("mathcal-mathbb", r"$\mathcal{L}\{f\}$, $x \in \mathbb{R}$"),
 ("degrees-circ", r"$90^\circ$, $\angle ABC$, $\triangle ABC$, $\perp$, $\parallel$"),
 ("therefore", r"$\therefore x = 2$, $\because$"),
 ("tfrac-dfrac", r"$\dfrac{1}{2} + \tfrac{3}{4}$"),
 ("overline-underline", r"$\overline{AB}$ and $\underline{x}$ and $\overbrace{a+b}^{n}$"),
 ("pmod-mod", r"$a \equiv b \pmod{n}$, $a \bmod b$"),
 ("limsup-max", r"$\max_{x} f(x)$, $\min(a,b)$, $\sup S$, $\inf S$, $\limsup_{n} a_n$, $\gcd(a,b)$"),
 ("spacing", r"$a\,b\:c\;d\!e\quad f\qquad g$"),
 ("displaystyle-frac", r"$\displaystyle \sum_{n=1}^\infty \frac{1}{n^2} = \frac{\pi^2}{6}$"),
 ("sqrt-nested", r"\[ \sqrt{1+\sqrt{2+\sqrt{3}}} \]"),
 ("ell-hbar", r"$\ell$, $\hbar$, $\Re z$, $\Im z$, $\aleph_0$"),
 ("cdot-dots-matrix", r"\[ \begin{pmatrix} a_{11} & \cdots & a_{1n} \\ \vdots & \ddots & \vdots \\ a_{m1} & \cdots & a_{mn} \end{pmatrix} \]"),
 ("ne-neq", r"$x \ne 0$, $a \ll b$, $c \gg d$, $x \simeq y$, $A \cong B$"),
 ("frac-pm-sqrt-choose", r"$\frac{n}{k}$, ${n \choose k}$"),
 ("stackrel-overset", r"$\overset{?}{=}$, $\stackrel{def}{=}$, $\underset{x}{\arg\min}$"),
]

PRE = "\\documentclass{article}\n\\usepackage{amsmath}\n\\usepackage{amssymb}\n\\begin{document}\n"
POST = "\n\\end{document}\n"

reqs = []
for i, (name, body) in enumerate(S):
    t = PRE + body + POST
    reqs.append(json.dumps({"protocol_version": 1, "id": name, "type": "compile",
        "payload": {"project_id": "cov", "revision": 1, "entry_path": "m.tex",
                    "documents": [{"path": "m.tex", "text": t}]}}))
out = subprocess.run([BIN], input="\n".join(reqs) + "\n", capture_output=True, text=True).stdout
rows = []
ok = 0
for (name, body), line in zip(S, out.splitlines()):
    p = json.loads(line)["payload"]
    diags = p.get("diagnostics", [])
    errs = [d["message"] for d in diags if d["severity"] == "error"]
    warns = [d["message"] for d in diags if d["severity"] != "error" and "recognised but not implemented" not in d["message"] and "U+2500" not in d["message"]]
    if not errs:
        ok += 1
    rows.append((name, body, p.get("status"), errs, warns))

md = sys.argv[1] if len(sys.argv) > 1 else None
lines = [f"0-error snippets: {ok}/{len(S)}", "", "| # | snippet | status | errors | diagnostics (first 3) |", "|---|---|---|---|---|"]
for i, (name, body, st, errs, warns) in enumerate(rows, 1):
    d = "; ".join((errs + warns)[:4]).replace("|", "\\|").replace("\n", " ")
    lines.append(f"| {i} | `{name}` | {st} | {len(errs)} | {d} |")
report = "\n".join(lines)
print(report)
