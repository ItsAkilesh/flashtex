#!/usr/bin/env python3
"""
Oracle corpus generator for crates/tex-expansion.

Each case is (id, setup, expr):
  - `setup` is executed as ordinary top-level TeX commands (definitions,
    assignments) -- it must produce no visible characters of its own.
  - `expr` is a purely *expandable* fragment (macro calls, \\the, \\number,
    \\romannumeral, \\string, \\csname, conditionals used as expressions,
    etc) with no bare assignments, since it is captured via a real TeX
    `\\write`, whose argument is scanned in "expand only" mode (exactly
    like `\\edef`'s body) -- assignments embedded there would NOT execute
    and would corrupt the captured output.

For each case this script:
  1. Writes tmp/<id>.tex running `setup` then `\\immediate\\write\\out{expr}`.
  2. Runs real `tex` (plain format directly on primitives -- no plain.tex
     macro dependency) in batchmode, oracle-only per KC-101 (never in the
     product path).
  3. Saves the captured line to tests/oracle/expected/<id>.txt.
  4. Appends a manifest entry (id -> full source `setup ++ expr`, which is
     exactly the string fed to the Rust engine's `expand_str`) to
     tests/oracle/manifest.json.

Cases needing the LaTeX kernel (`\newcommand`, counters, `\@ifnextchar`,
...) set `latex=True` and are run through `pdflatex` with a minimal
`article` preamble instead of bare `tex`.

Run: python3 tests/oracle/gen/cases.py   (from crates/tex-expansion/)
"""
import json
import os
import subprocess
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
ORACLE_DIR = os.path.dirname(HERE)
TMP_DIR = os.path.join(ORACLE_DIR, "tmp")
EXPECTED_DIR = os.path.join(ORACLE_DIR, "expected")
MANIFEST_PATH = os.path.join(ORACLE_DIR, "manifest.json")

os.makedirs(TMP_DIR, exist_ok=True)
os.makedirs(EXPECTED_DIR, exist_ok=True)

# (id, setup, expr, latex)
CASES = [
    # -- \def / \let / basic macro calling --------------------------------
    ("def_simple", r"\def\a{hello}", r"\a", "tex"),
    ("def_with_arg", r"\def\greet#1{Hello, #1!}", r"\greet{world}", "tex"),
    ("def_delimited", r"\def\a#1;{[#1]}", r"\a hello;", "tex"),
    ("def_two_delimited", r"\def\a#1,#2.{(#1)(#2)}", r"\a x,y.", "tex"),
    ("let_to_macro", r"\def\a{X}\let\b=\a", r"\b", "tex"),
    # NOTE: no `let_to_char` case here: `\let\a=b` makes `\a` a
    # non-expandable "let to character" token. Real TeX's `\write`
    # captures such tokens as the literal control sequence "\a " (since
    # `\write`'s scan only expands *expandable* tokens and copies
    # everything else verbatim) -- but our engine intentionally
    # substitutes the underlying character immediately, matching what
    # real TeX's *main control* does when such a token reaches
    # typesetting (append the character, exactly as if `b` had been
    # typed). Those are two different real-TeX subsystems with two
    # different observable behaviors for the same construct; since this
    # crate's output feeds typesetting (not `\write`), `\write` is the
    # wrong oracle for this one case. See CONTRACT.md "Known deviations".
    ("def_nested_call", r"\def\inner{IN}\def\outer{[\inner]}", r"\outer", "tex"),
    ("def_hash_hash", r"\def\a{\#}", r"\a", "tex"),
    # NOTE: no oracle case for `#{` (brace-delimited last parameter) here.
    # `#{` leaves the delimiting `{` unconsumed for whatever follows the
    # call (TeXbook p.205); under real *document* typesetting (main
    # control) that leftover `{...}` is a silent group (matching this
    # crate's design -- see CONTRACT.md "Known deviations"), but `\write`'s
    # argument scan is a `scan_toks` context that always preserves braces
    # literally, so it cannot validate this construct without producing a
    # misleading expectation. Covered instead by the Rust unit test
    # `def_brace_delim_last` in tests/expand_tests.rs.
    # -- \edef / \noexpand / \expandafter ----------------------------------
    ("edef_expands_body", r"\def\a{X}\edef\b{[\a]}", r"\b", "tex"),
    ("edef_noexpand", r"\def\a{X}\edef\b{\noexpand\a}\let\a=\relax", r"\b", "tex"),
    ("expandafter_basic", r"\def\a{X}\def\b{\a}\expandafter\def\expandafter\c\expandafter{\b}", r"\c", "tex"),
    ("expandafter_chain", r"\def\a{A}\def\b{B}\def\pick#1#2{#1}", r"\expandafter\pick\expandafter{\a}{\b}", "tex"),
    # -- \csname / \string / \number / \romannumeral ------------------------
    ("csname_builds", r"\def\foo{bar}", r"\csname foo\endcsname", "tex"),
    ("csname_undefined_relax", r"", r"\ifx\csname zzz\endcsname\relax DEF\else NOTDEF\fi", "tex"),
    ("string_of_cs", r"", r"\string\foo", "tex"),
    ("string_of_char", r"", r"\string a", "tex"),
    ("number_basic", r"", r"\number 42", "tex"),
    ("number_negative", r"", r"\number -7", "tex"),
    ("romannumeral_basic", r"", r"\romannumeral 1994", "tex"),
    ("romannumeral_zero", r"", r"\romannumeral 0", "tex"),
    # -- registers / \the / arithmetic --------------------------------------
    ("count_the", r"\count0=5 ", r"\the\count0", "tex"),
    ("count_advance", r"\count0=5 \advance\count0 by 3 ", r"\the\count0", "tex"),
    ("count_multiply", r"\count0=5 \multiply\count0 by 3 ", r"\the\count0", "tex"),
    ("count_divide", r"\count0=15 \divide\count0 by 3 ", r"\the\count0", "tex"),
    ("countdef_alias", r"\countdef\mycount=5 \mycount=7 ", r"\the\mycount", "tex"),
    ("dimen_pt", r"\dimen0=12pt ", r"\the\dimen0", "tex"),
    ("dimen_in", r"\dimen0=1in ", r"\the\dimen0", "tex"),
    ("numexpr_basic", r"\count0=\numexpr 2+3*4\relax ", r"\the\count0", "etex"),
    ("numexpr_parens", r"\count0=\numexpr (2+3)*4\relax ", r"\the\count0", "etex"),
    # -- grouping / \global / \aftergroup -----------------------------------
    ("group_restores_local", r"\def\a{outer}{\def\a{inner}}", r"\a", "tex"),
    ("group_global_survives", r"\def\a{outer}{\global\def\a{inner}}", r"\a", "tex"),
    ("begingroup_endgroup_count", r"\count0=1 \begingroup\count0=2 \endgroup", r"\the\count0", "tex"),
    # -- conditionals --------------------------------------------------------
    ("iftrue_basic", r"", r"\iftrue A\else B\fi", "tex"),
    ("iffalse_basic", r"", r"\iffalse A\else B\fi", "tex"),
    ("ifnum_lt", r"", r"\ifnum 3<5 yes\else no\fi", "tex"),
    ("ifnum_gt", r"", r"\ifnum 5<3 yes\else no\fi", "tex"),
    ("ifnum_eq", r"", r"\ifnum 5=5 yes\else no\fi", "tex"),
    ("ifdim_lt", r"", r"\ifdim 1pt<2pt yes\else no\fi", "tex"),
    ("ifodd_true", r"", r"\ifodd 3 odd\else even\fi", "tex"),
    ("ifodd_false", r"", r"\ifodd 4 odd\else even\fi", "tex"),
    ("ifx_same", r"\def\a{X}\def\b{X}", r"\ifx\a\b same\else different\fi", "tex"),
    ("ifx_diff", r"\def\a{X}\def\b{Y}", r"\ifx\a\b same\else different\fi", "tex"),
    ("ifcase_two", r"", r"\ifcase 2 zero\or one\or two\or three\fi", "tex"),
    ("ifcase_zero", r"", r"\ifcase 0 zero\or one\or two\fi", "tex"),
    ("if_char_eq", r"", r"\if ab same\else diff\fi", "tex"),
    ("if_char_same", r"", r"\if aa same\else diff\fi", "tex"),
    ("ifcat_letters", r"", r"\ifcat ab same\else diff\fi", "tex"),
    ("ifcat_diff", r"", r"\ifcat a1 same\else diff\fi", "tex"),
    ("nested_if_else", r"", r"\iftrue \iffalse T\else F\fi \else O\fi", "tex"),
    ("newif_true", r"\newif\ifmyflag \myflagtrue", r"\ifmyflag YES\else NO\fi", "tex"),
    ("newif_false", r"\newif\ifmyflag \myflagfalse", r"\ifmyflag YES\else NO\fi", "tex"),
    ("unless_iftrue", r"", r"\unless\iftrue A\else B\fi", "etex"),
    # -- LaTeX layer (rendered as document body, captured via pdftotext,
    # since these macros use \let/\def/\futurelet internally and so
    # cannot be captured through \write's edef-like restricted scan) ------
    ("newcommand_basic", r"\newcommand{\hi}{Hello}", r"\hi", "latex-render"),
    ("newcommand_arg", r"\newcommand{\greet}[1]{Hello #1}", r"\greet{World}", "latex-render"),
    ("newcommand_opt_default", r"\newcommand{\greet}[2][Hi]{#1, #2}", r"\greet{World}", "latex-render"),
    ("newcommand_opt_given", r"\newcommand{\greet}[2][Hi]{#1, #2}", r"\greet[Yo]{World}", "latex-render"),
    ("renewcommand_basic", r"\newcommand{\mytestcmd}{old}\renewcommand{\mytestcmd}{new}", r"\mytestcmd", "latex-render"),
    ("providecommand_keeps", r"\newcommand{\mytestcmd}{old}\providecommand{\mytestcmd}{new}", r"\mytestcmd", "latex-render"),
    ("newenvironment_basic", r"\newenvironment{myenv}{[BEGIN]}{[END]}", r"\begin{myenv}content\end{myenv}", "latex-render"),
    ("newcounter_setcounter", r"\newcounter{foo}\setcounter{foo}{5}", r"\arabic{foo}", "latex-render"),
    ("stepcounter_twice", r"\newcounter{foo}\stepcounter{foo}\stepcounter{foo}", r"\arabic{foo}", "latex-render"),
    ("addtocounter_basic", r"\newcounter{foo}\setcounter{foo}{3}\addtocounter{foo}{4}", r"\arabic{foo}", "latex-render"),
    ("roman_counter", r"\newcounter{foo}\setcounter{foo}{4}", r"\roman{foo}", "latex-render"),
    ("Roman_counter", r"\newcounter{foo}\setcounter{foo}{4}", r"\Roman{foo}", "latex-render"),
    ("alph_counter", r"\newcounter{foo}\setcounter{foo}{1}", r"\alph{foo}", "latex-render"),
    ("Alph_counter", r"\newcounter{foo}\setcounter{foo}{2}", r"\Alph{foo}", "latex-render"),
    (
        "ifnextchar_star",
        r"\makeatletter\def\test{\@ifnextchar*{\@teststar}{\@testnostar}}\def\@teststar*{STAR}\def\@testnostar{NOSTAR}\makeatother",
        r"\test*",
        "latex-render",
    ),
    (
        "ifnextchar_nostar",
        r"\makeatletter\def\test{\@ifnextchar*{\@teststar}{\@testnostar}}\def\@teststar*{STAR}\def\@testnostar{NOSTAR}\makeatother",
        r"\test ",
        "latex-render",
    ),
    ("ifstar_star", r"\makeatletter\def\test{\@ifstar{STARBRANCH}{NOSTARBRANCH}}\makeatother", r"\test*", "latex-render"),
    ("nameuse_namedef", r"\makeatletter\@namedef{foo}{bar}", r"\@nameuse{foo}\makeatother", "latex-render"),
    # -- additional coverage: more def/param/register/conditional variety ----
    ("def_no_args_reuse", r"\def\x{A}\def\y{\x\x\x}", r"\y", "tex"),
    ("def_param_repeated", r"\def\dup#1{#1#1}", r"\dup{ab}", "tex"),
    ("def_three_params", r"\def\three#1#2#3{#3#2#1}", r"\three{a}{b}{c}", "tex"),
    ("let_chain", r"\def\a{X}\let\b=\a\let\c=\b", r"\c", "tex"),
    ("edef_concat", r"\def\a{A}\def\b{B}\edef\c{\a\b}", r"\c", "tex"),
    ("edef_the_count", r"\count5=9 \edef\c{\the\count5}\count5=0 ", r"\c", "tex"),
    ("csname_expandafter", r"\def\foo{FOO}", r"\expandafter\csname foo\endcsname", "tex"),
    ("string_of_digit", r"", r"\string 5", "tex"),
    ("number_of_count", r"\count0=99 ", r"\number\count0", "tex"),
    ("romannumeral_small", r"", r"\romannumeral 9", "tex"),
    ("romannumeral_large", r"", r"\romannumeral 3888", "tex"),
    ("count_octal", r"\count0='17 ", r"\the\count0", "tex"),
    ("count_hex", r'\count0="FF ', r"\the\count0", "tex"),
    ("count_backtick", r"\count0=`A ", r"\the\count0", "tex"),
    ("count_negative_advance", r"\count0=5 \advance\count0 by -8 ", r"\the\count0", "tex"),
    ("dimen_cm", r"\dimen0=2.5cm ", r"\the\dimen0", "tex"),
    ("dimen_negative", r"\dimen0=-3pt ", r"\the\dimen0", "tex"),
    ("skip_the", r"\skip0=1pt plus 2pt minus 3pt ", r"\the\skip0", "tex"),
    ("group_nested_three_deep", r"\def\a{L0}{\def\a{L1}{\def\a{L2}}}", r"\a", "tex"),
    ("ifnum_ge_via_not_lt", r"", r"\ifnum 5<5 lt\else \ifnum 5>5 gt\else eq\fi\fi", "tex"),
    ("ifdim_gt", r"", r"\ifdim 3pt>1pt yes\else no\fi", "tex"),
    ("ifx_undefined_both", r"", r"\ifx\undefinedaaa\undefinedbbb same\else different\fi", "tex"),
    ("ifx_primitive_same", r"", r"\ifx\relax\relax same\else different\fi", "tex"),
    ("ifcase_negative_falls_to_else", r"", r"\ifcase -1 zero\or one\else other\fi", "tex"),
    ("ifcase_beyond_or_falls_to_else", r"", r"\ifcase 5 zero\or one\else other\fi", "tex"),
    ("nested_ifcase_in_iftrue", r"", r"\iftrue\ifcase 1 a\or b\or c\fi\fi", "tex"),
    ("newif_default_false", r"\newif\ifmyflag ", r"\ifmyflag YES\else NO\fi", "tex"),
    ("unless_iffalse", r"", r"\unless\iffalse A\else B\fi", "etex"),
    ("dimexpr_basic", r"\dimen0=\dimexpr 2pt+3pt\relax ", r"\the\dimen0", "etex"),
    ("numexpr_division", r"\count0=\numexpr 17/3\relax ", r"\the\count0", "etex"),
    ("numexpr_nested_parens", r"\count0=\numexpr (2+3)*(4-1)\relax ", r"\the\count0", "etex"),
    ("gdef_survives_extra_group", r"\makeatletter{\gdef\foo@bar{Y}}", r"\foo@bar", "tex"),
    ("catcode_tilde_as_active", r"\catcode`\~=13 \def~{TILDE}", r"~", "tex"),
    ("countdef_then_advance", r"\countdef\mycount=9 \mycount=1 \advance\mycount by 4 ", r"\the\mycount", "tex"),
    ("global_inside_nested_group", r"\count0=1 {{\global\count0=9 }}", r"\the\count0", "tex"),
    ("newcommand_two_args", r"\newcommand{\pair}[2]{(#1,#2)}", r"\pair{x}{y}", "latex-render"),
    ("newcounter_within_arabic", r"\newcounter{sec}\newcounter{sub}[sec]\setcounter{sub}{3}", r"\arabic{sub}", "latex-render"),
    ("refstepcounter_basic", r"\newcounter{foo}\refstepcounter{foo}\refstepcounter{foo}", r"\arabic{foo}", "latex-render"),
    # NOTE: no oracle case for `\fnsymbol` -- real LaTeX renders it as
    # math-mode footnote symbols (asterisk-operator, dagger, ...), which
    # are not representable as plain catcode-Other characters at all; our
    # engine deliberately substitutes ASCII approximations ("*", "**",
    # "#", ...), documented as a known limitation in CONTRACT.md. A
    # pdftotext-captured Unicode glyph could never equal that
    # approximation, so no oracle comparison is meaningful here.
    ("value_in_setcounter", r"\newcounter{foo}\setcounter{foo}{4}\newcounter{bar}\setcounter{bar}{\value{foo}}", r"\arabic{bar}", "latex-render"),
]


def run_write_mode(tex_source: str, case_id: str, engine: str) -> str:
    src_path = os.path.join(TMP_DIR, f"{case_id}.tex")
    with open(src_path, "w") as f:
        f.write(tex_source)
    cmd = [engine, "-interaction=batchmode", "-output-directory", TMP_DIR, src_path]
    result = subprocess.run(cmd, cwd=TMP_DIR, stdout=subprocess.PIPE, stderr=subprocess.STDOUT, timeout=30)
    out_path = os.path.join(TMP_DIR, f"{case_id}.out")
    if not os.path.exists(out_path) or os.path.getsize(out_path) == 0:
        log_path = os.path.join(TMP_DIR, f"{case_id}.log")
        log = open(log_path, errors="replace").read() if os.path.exists(log_path) else "(no log)"
        raise RuntimeError(
            f"{case_id}: no/empty output file.\n--- stdout ---\n{result.stdout.decode('utf-8', 'replace')}\n--- log tail ---\n{log[-1500:]}"
        )
    with open(out_path, "r") as f:
        return f.read().rstrip("\n")


def build_write_source(setup: str, expr: str, engine: str, case_id: str) -> str:
    preamble = r"\catcode`\@=11" + "\n" if engine != "latex" else ""
    return (
        preamble
        + r"\newwrite\out" + "\n"
        + rf"\immediate\openout\out={case_id}.out" + "\n"
        + setup + "\n"
        + r"\immediate\write\out{" + expr + "}\n"
        + r"\immediate\closeout\out" + "\n"
        + r"\end" + "\n"
    )


def run_render_mode(setup: str, expr: str, case_id: str) -> str:
    """For LaTeX-layer macros whose internals (\\futurelet, nested \\def) do
    not survive \\write's edef-like restricted expansion: typeset `setup`
    then `expr` as ordinary document body content (full main-control
    execution, completely faithful), then extract text from the rendered
    PDF with `pdftotext`."""
    src_path = os.path.join(TMP_DIR, f"{case_id}.tex")
    source = (
        r"\documentclass{article}"
        + "\n\\pagestyle{empty}\n\\begin{document}\n"
        + r"\newcommand{\oraclemarker}{}"
        + "\n\\noindent ORACLESTART\\oraclemarker\\ "
        + setup
        + "\n"
        + expr
        + "\n\\ ORACLEEND\\oraclemarker\n\\end{document}\n"
    )
    with open(src_path, "w") as f:
        f.write(source)
    cmd = ["pdflatex", "-interaction=batchmode", "-output-directory", TMP_DIR, src_path]
    subprocess.run(cmd, cwd=TMP_DIR, stdout=subprocess.PIPE, stderr=subprocess.STDOUT, timeout=30)
    pdf_path = os.path.join(TMP_DIR, f"{case_id}.pdf")
    if not os.path.exists(pdf_path):
        log_path = os.path.join(TMP_DIR, f"{case_id}.log")
        log = open(log_path, errors="replace").read() if os.path.exists(log_path) else "(no log)"
        raise RuntimeError(f"{case_id}: pdflatex produced no PDF.\n--- log tail ---\n{log[-1500:]}")
    text = subprocess.run(["pdftotext", "-layout", pdf_path, "-"], cwd=TMP_DIR, stdout=subprocess.PIPE, timeout=30)
    rendered = text.stdout.decode("utf-8", "replace")
    # Extract exactly the text between our ORACLESTART/ORACLEEND markers
    # (unique tokens, so they can't collide with a case's own content).
    start = rendered.find("ORACLESTART")
    end = rendered.find("ORACLEEND", start if start >= 0 else 0)
    if start < 0 or end < 0:
        raise RuntimeError(f"{case_id}: ORACLESTART/ORACLEEND markers not found in rendered text:\n{rendered!r}")
    middle = rendered[start + len("ORACLESTART") : end]
    return middle.strip()


def main():
    manifest = {}
    failures = []
    for case_id, setup, expr, mode in CASES:
        try:
            if mode == "latex-render":
                out = run_render_mode(setup, expr, case_id)
            else:
                engine = "etex" if mode == "etex" else "tex"
                source = build_write_source(setup, expr, engine, case_id)
                out = run_write_mode(source, case_id, engine)
        except Exception as e:
            failures.append((case_id, str(e)))
            print(f"FAIL {case_id}: {e}", file=sys.stderr)
            continue
        with open(os.path.join(EXPECTED_DIR, f"{case_id}.txt"), "w") as f:
            f.write(out)
        manifest[case_id] = {"setup": setup, "expr": expr, "expected": out, "mode": mode}
        print(f"OK   {case_id}: {out!r}")
    with open(MANIFEST_PATH, "w") as f:
        json.dump(manifest, f, indent=2, sort_keys=True)
    print(f"\n{len(manifest)} cases captured, {len(failures)} failed.")
    if failures:
        sys.exit(1)


if __name__ == "__main__":
    main()
