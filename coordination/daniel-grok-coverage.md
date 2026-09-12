# daniel-grok-coverage: handwritten-math compile coverage

Offline corpus of 63 short snippets (9 expected outputs from tests/grok-corpus/cases plus common homework forms), each wrapped in article+amsmath+amssymb and compiled through the flashtex-compiler JSON Lines CLI. A snippet counts as rendering when it compiles with 0 error diagnostics (warnings listed; package-recognition and U+2500 rule-fallback warnings omitted). Reproduce: build crates/compiler, then `python3 coordination/daniel-grok-coverage.py`.

Summary: before 16/63, after batch 1 38/63, after batch 2 46/63.

## After batch 2

0-error snippets: 46/63

| # | snippet | status | errors | diagnostics (first 3) |
|---|---|---|---|---|
| 1 | `corpus-integral-unicode` | recovered | 0 |  |
| 2 | `corpus-det-pmatrix` | recovered | 0 |  |
| 3 | `corpus-prose-math` | recovered | 0 |  |
| 4 | `corpus-align-star` | recovered | 0 |  |
| 5 | `corpus-cfrac-nested` | recovered | 0 |  |
| 6 | `corpus-quadratic` | recovered | 0 |  |
| 7 | `corpus-problem-lim` | recovered | 0 |  |
| 8 | `corpus-bigskip` | recovered | 1 | \bigskip is not supported by this compiler version; unrestricted TeX math mode is not implemented |
| 9 | `lim` | recovered | 0 |  |
| 10 | `lim-inf` | recovered | 0 |  |
| 11 | `derivative-d-dx` | recovered | 0 |  |
| 12 | `partial` | recovered | 0 |  |
| 13 | `vec-hat-bar` | recovered | 3 | \vec is not supported in math mode; \hat is not supported in math mode; \bar is not supported in math mode |
| 14 | `mathbf` | recovered | 0 |  |
| 15 | `mathrm` | recovered | 0 |  |
| 16 | `operatorname` | recovered | 0 |  |
| 17 | `left-right-big` | recovered | 0 |  |
| 18 | `binary-ops` | recovered | 1 | \mp is not supported in math mode |
| 19 | `relations` | recovered | 0 |  |
| 20 | `greek-lower` | recovered | 0 |  |
| 21 | `greek-upper` | recovered | 0 |  |
| 22 | `functions` | recovered | 0 |  |
| 23 | `binom` | recovered | 0 |  |
| 24 | `sqrt-n` | recovered | 0 |  |
| 25 | `dots` | recovered | 2 | \vdots is not supported in math mode; \ddots is not supported in math mode |
| 26 | `text-in-math` | recovered | 0 |  |
| 27 | `multline` | recovered | 0 |  |
| 28 | `split` | recovered | 0 |  |
| 29 | `alignat` | recovered | 0 |  |
| 30 | `tag` | recovered | 0 |  |
| 31 | `boxed` | recovered | 0 |  |
| 32 | `tabular` | recovered | 3 | \hline is not supported by this compiler version; unrestricted TeX math mode is not implemented; \hline is not supported by this compiler version; unrestricted TeX math mode is not implemented; \hline is not supported by this compiler version; unrestricted TeX math mode is not implemented; environment 'tabular' is not implemented; its body is typeset as plain text |
| 33 | `sum-limits` | recovered | 0 |  |
| 34 | `int-limits` | recovered | 0 |  |
| 35 | `iint-oint` | recovered | 1 | \oint is not supported in math mode |
| 36 | `nabla-infty` | recovered | 0 |  |
| 37 | `set-braces` | recovered | 0 |  |
| 38 | `set-ops` | recovered | 0 |  |
| 39 | `quantifiers` | recovered | 0 |  |
| 40 | `arrows` | recovered | 1 | \mapsto is not supported in math mode |
| 41 | `cases` | recovered | 0 |  |
| 42 | `prime-deriv` | recovered | 0 |  |
| 43 | `abs-norm` | recovered | 0 |  |
| 44 | `floor-ceil-angle` | recovered | 4 | \lfloor is not supported in math mode; \rfloor is not supported in math mode; \lceil is not supported in math mode; \rceil is not supported in math mode |
| 45 | `equation-numbered` | recovered | 0 |  |
| 46 | `align-numbered` | recovered | 0 |  |
| 47 | `gather` | recovered | 0 |  |
| 48 | `bmatrix` | recovered | 0 |  |
| 49 | `mathcal-mathbb` | recovered | 2 | \mathcal is not supported in math mode; \mathbb is not supported in math mode |
| 50 | `degrees-circ` | recovered | 3 | \circ is not supported in math mode; \triangle is not supported in math mode; \parallel is not supported in math mode |
| 51 | `therefore` | recovered | 1 | \because is not supported in math mode |
| 52 | `tfrac-dfrac` | recovered | 0 |  |
| 53 | `overline-underline` | recovered | 1 | \overbrace is not supported in math mode |
| 54 | `pmod-mod` | recovered | 0 |  |
| 55 | `limsup-max` | recovered | 0 |  |
| 56 | `spacing` | recovered | 0 |  |
| 57 | `displaystyle-frac` | recovered | 0 |  |
| 58 | `sqrt-nested` | recovered | 0 |  |
| 59 | `ell-hbar` | recovered | 2 | \ell is not supported in math mode; \hbar is not supported in math mode |
| 60 | `cdot-dots-matrix` | recovered | 3 | \vdots is not supported in math mode; \ddots is not supported in math mode; \vdots is not supported in math mode |
| 61 | `ne-neq` | recovered | 3 | \ll is not supported in math mode; \gg is not supported in math mode; \simeq is not supported in math mode |
| 62 | `frac-pm-sqrt-choose` | recovered | 1 | \choose is not supported in math mode |
| 63 | `stackrel-overset` | recovered | 3 | \overset is not supported in math mode; \stackrel is not supported in math mode; \underset is not supported in math mode |

## Before (origin/main + amsmath lane, f05b4cef)

0-error snippets: 16/63

| # | snippet | status | errors | diagnostics (first 3) |
|---|---|---|---|---|
| 1 | `corpus-integral-unicode` | recovered | 1 | \sin is not supported in math mode; packages amsmath are recognised but not implemented; packages amssymb are recognised but not implemented |
| 2 | `corpus-det-pmatrix` | recovered | 1 | \det is not supported in math mode; packages amsmath are recognised but not implemented; packages amssymb are recognised but not implemented |
| 3 | `corpus-prose-math` | recovered | 0 | packages amsmath are recognised but not implemented; packages amssymb are recognised but not implemented |
| 4 | `corpus-align-star` | recovered | 0 | packages amsmath are recognised but not implemented; packages amssymb are recognised but not implemented |
| 5 | `corpus-cfrac-nested` | recovered | 3 | \cfrac is not supported in math mode; \cfrac is not supported in math mode; \cfrac is not supported in math mode |
| 6 | `corpus-quadratic` | recovered | 0 | packages amsmath are recognised but not implemented; packages amssymb are recognised but not implemented; '─' (U+2500) will not survive PDF export: fraction rules are drawn with a box-drawing character as a stand-in; runtime-v1 has no rule item type yet, so they cannot be exported faithfully |
| 7 | `corpus-problem-lim` | recovered | 7 | \displaystyle is not supported in math mode; \lim is not supported in math mode; \to is not supported in math mode |
| 8 | `corpus-bigskip` | recovered | 1 | \bigskip is not supported by this compiler version; unrestricted TeX math mode is not implemented; packages amsmath are recognised but not implemented; packages amssymb are recognised but not implemented |
| 9 | `lim` | recovered | 3 | \lim is not supported in math mode; \to is not supported in math mode; \cos is not supported in math mode |
| 10 | `lim-inf` | recovered | 4 | \lim is not supported in math mode; \to is not supported in math mode; \left is not supported in math mode |
| 11 | `derivative-d-dx` | recovered | 0 | packages amsmath are recognised but not implemented; packages amssymb are recognised but not implemented; '─' (U+2500) will not survive PDF export: fraction rules are drawn with a box-drawing character as a stand-in; runtime-v1 has no rule item type yet, so they cannot be exported faithfully |
| 12 | `partial` | recovered | 4 | \partial is not supported in math mode; \partial is not supported in math mode; \partial is not supported in math mode |
| 13 | `vec-hat-bar` | recovered | 3 | \vec is not supported in math mode; \hat is not supported in math mode; \bar is not supported in math mode |
| 14 | `mathbf` | recovered | 2 | \mathbf is not supported in math mode; \mathbf is not supported in math mode; packages amsmath are recognised but not implemented |
| 15 | `mathrm` | recovered | 2 | \mathrm is not supported in math mode; \mathrm is not supported in math mode; packages amsmath are recognised but not implemented |
| 16 | `operatorname` | recovered | 2 | \operatorname is not supported in math mode; \operatorname is not supported in math mode; packages amsmath are recognised but not implemented |
| 17 | `left-right-big` | recovered | 8 | \left is not supported in math mode; \right is not supported in math mode; \big is not supported in math mode |
| 18 | `binary-ops` | recovered | 1 | \mp is not supported in math mode; packages amsmath are recognised but not implemented; packages amssymb are recognised but not implemented |
| 19 | `relations` | recovered | 5 | \equiv is not supported in math mode; \sim is not supported in math mode; \propto is not supported in math mode |
| 20 | `greek-lower` | recovered | 15 | \epsilon is not supported in math mode; \varepsilon is not supported in math mode; \zeta is not supported in math mode |
| 21 | `greek-upper` | recovered | 11 | \Gamma is not supported in math mode; \Delta is not supported in math mode; \Theta is not supported in math mode |
| 22 | `functions` | recovered | 8 | \sin is not supported in math mode; \cos is not supported in math mode; \tan is not supported in math mode |
| 23 | `binom` | recovered | 1 | \binom is not supported in math mode; packages amsmath are recognised but not implemented; packages amssymb are recognised but not implemented |
| 24 | `sqrt-n` | recovered | 1 | \sqrt requires a braced math argument; packages amsmath are recognised but not implemented; packages amssymb are recognised but not implemented |
| 25 | `dots` | recovered | 5 | \dots is not supported in math mode; \cdots is not supported in math mode; \ldots is not supported in math mode |
| 26 | `text-in-math` | recovered | 0 | packages amsmath are recognised but not implemented; packages amssymb are recognised but not implemented |
| 27 | `multline` | recovered | 0 | packages amsmath are recognised but not implemented; packages amssymb are recognised but not implemented; environment 'multline*' is not implemented; its body is typeset as plain text |
| 28 | `split` | recovered | 2 | \begin{split} is not supported in math mode; \end is not supported in math mode; packages amsmath are recognised but not implemented |
| 29 | `alignat` | recovered | 1 | \quad is not supported by this compiler version; unrestricted TeX math mode is not implemented; packages amsmath are recognised but not implemented; packages amssymb are recognised but not implemented |
| 30 | `tag` | recovered | 1 | \tag is not supported in math mode; packages amsmath are recognised but not implemented; packages amssymb are recognised but not implemented |
| 31 | `boxed` | recovered | 1 | \boxed is not supported in math mode; packages amsmath are recognised but not implemented; packages amssymb are recognised but not implemented |
| 32 | `tabular` | recovered | 3 | \hline is not supported by this compiler version; unrestricted TeX math mode is not implemented; \hline is not supported by this compiler version; unrestricted TeX math mode is not implemented; \hline is not supported by this compiler version; unrestricted TeX math mode is not implemented |
| 33 | `sum-limits` | recovered | 1 | \prod is not supported in math mode; packages amsmath are recognised but not implemented; packages amssymb are recognised but not implemented |
| 34 | `int-limits` | recovered | 0 | packages amsmath are recognised but not implemented; packages amssymb are recognised but not implemented; '─' (U+2500) will not survive PDF export: fraction rules are drawn with a box-drawing character as a stand-in; runtime-v1 has no rule item type yet, so they cannot be exported faithfully |
| 35 | `iint-oint` | recovered | 3 | \iint is not supported in math mode; \oint is not supported in math mode; \iiint is not supported in math mode |
| 36 | `nabla-infty` | recovered | 3 | \nabla is not supported in math mode; \mathbf is not supported in math mode; \rho is not supported in math mode |
| 37 | `set-braces` | recovered | 0 | packages amsmath are recognised but not implemented; packages amssymb are recognised but not implemented |
| 38 | `set-ops` | recovered | 7 | \cup is not supported in math mode; \cap is not supported in math mode; \subset is not supported in math mode |
| 39 | `quantifiers` | recovered | 1 | \epsilon is not supported in math mode; packages amsmath are recognised but not implemented; packages amssymb are recognised but not implemented |
| 40 | `arrows` | recovered | 7 | \to is not supported in math mode; \mapsto is not supported in math mode; \iff is not supported in math mode |
| 41 | `cases` | recovered | 0 | packages amsmath are recognised but not implemented; packages amssymb are recognised but not implemented |
| 42 | `prime-deriv` | recovered | 0 | packages amsmath are recognised but not implemented; packages amssymb are recognised but not implemented |
| 43 | `abs-norm` | recovered | 4 | \lvert is not supported in math mode; \rvert is not supported in math mode; \lVert is not supported in math mode |
| 44 | `floor-ceil-angle` | recovered | 6 | \lfloor is not supported in math mode; \rfloor is not supported in math mode; \lceil is not supported in math mode |
| 45 | `equation-numbered` | recovered | 0 | packages amsmath are recognised but not implemented; packages amssymb are recognised but not implemented |
| 46 | `align-numbered` | recovered | 0 | packages amsmath are recognised but not implemented; packages amssymb are recognised but not implemented |
| 47 | `gather` | recovered | 0 | packages amsmath are recognised but not implemented; packages amssymb are recognised but not implemented |
| 48 | `bmatrix` | recovered | 0 | packages amsmath are recognised but not implemented; packages amssymb are recognised but not implemented |
| 49 | `mathcal-mathbb` | recovered | 2 | \mathcal is not supported in math mode; \mathbb is not supported in math mode; packages amsmath are recognised but not implemented |
| 50 | `degrees-circ` | recovered | 5 | \circ is not supported in math mode; \angle is not supported in math mode; \triangle is not supported in math mode |
| 51 | `therefore` | recovered | 2 | \therefore is not supported in math mode; \because is not supported in math mode; packages amsmath are recognised but not implemented |
| 52 | `tfrac-dfrac` | recovered | 2 | \dfrac is not supported in math mode; \tfrac is not supported in math mode; packages amsmath are recognised but not implemented |
| 53 | `overline-underline` | recovered | 3 | \overline is not supported in math mode; \underline is not supported in math mode; \overbrace is not supported in math mode |
| 54 | `pmod-mod` | recovered | 3 | \equiv is not supported in math mode; \pmod is not supported in math mode; \bmod is not supported in math mode |
| 55 | `limsup-max` | recovered | 6 | \max is not supported in math mode; \min is not supported in math mode; \sup is not supported in math mode |
| 56 | `spacing` | recovered | 0 | packages amsmath are recognised but not implemented; packages amssymb are recognised but not implemented |
| 57 | `displaystyle-frac` | recovered | 1 | \displaystyle is not supported in math mode; packages amsmath are recognised but not implemented; packages amssymb are recognised but not implemented |
| 58 | `sqrt-nested` | recovered | 0 | packages amsmath are recognised but not implemented; packages amssymb are recognised but not implemented |
| 59 | `ell-hbar` | recovered | 5 | \ell is not supported in math mode; \hbar is not supported in math mode; \Re is not supported in math mode |
| 60 | `cdot-dots-matrix` | recovered | 5 | \cdots is not supported in math mode; \vdots is not supported in math mode; \ddots is not supported in math mode |
| 61 | `ne-neq` | recovered | 5 | \ne is not supported in math mode; \ll is not supported in math mode; \gg is not supported in math mode |
| 62 | `frac-pm-sqrt-choose` | recovered | 1 | \choose is not supported in math mode; packages amsmath are recognised but not implemented; packages amssymb are recognised but not implemented |
| 63 | `stackrel-overset` | recovered | 5 | \overset is not supported in math mode; \stackrel is not supported in math mode; \underset is not supported in math mode |
