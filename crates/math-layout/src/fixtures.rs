//! Reference expressions shared by the golden tests, the `dump` example and
//! the oracle comparison in `docs/comparison.md`. Each constructor documents
//! the LaTeX source it models.

use crate::mathlist::{Atom, MathList};

fn sym(s: &str) -> MathList {
    MathList::symbols(s)
}

/// `x^2`
pub fn x_squared() -> MathList {
    Atom::symbol('x').with_sup(sym("2")).into()
}

/// `x_i^2`
pub fn x_sub_i_sup_2() -> MathList {
    Atom::symbol('x')
        .with_sub(sym("i"))
        .with_sup(sym("2"))
        .into()
}

/// `x^{y^z}`
pub fn x_sup_y_sup_z() -> MathList {
    let y_z: MathList = Atom::symbol('y').with_sup(sym("z")).into();
    Atom::symbol('x').with_sup(y_z).into()
}

/// `\frac{a}{b}`
pub fn frac_a_b() -> MathList {
    Atom::frac(sym("a"), sym("b")).into()
}

/// `\frac{\frac{a}{b}}{c}`
pub fn stacked_fraction() -> MathList {
    Atom::frac(frac_a_b(), sym("c")).into()
}

/// `\sqrt{x}`
pub fn sqrt_x() -> MathList {
    Atom::sqrt(sym("x")).into()
}

/// `\sqrt{\frac{a}{b}}`
pub fn sqrt_frac() -> MathList {
    Atom::sqrt(frac_a_b()).into()
}

/// `\left(\frac{a}{b}\right)`
pub fn left_right_frac() -> MathList {
    Atom::left_right(Some('('), Some(')'), frac_a_b()).into()
}

/// `\sum_{i=1}^n`
pub fn sum_limits() -> MathList {
    Atom::symbol('\u{2211}')
        .with_sub(sym("i=1"))
        .with_sup(sym("n"))
        .into()
}

/// `\int_0^1`
pub fn int_0_1() -> MathList {
    Atom::symbol('\u{222B}')
        .with_sub(sym("0"))
        .with_sup(sym("1"))
        .into()
}

/// `\hat{x}`
pub fn hat_x() -> MathList {
    Atom::accent('^', sym("x")).into()
}

/// `a+b`
pub fn a_plus_b() -> MathList {
    sym("a+b")
}

/// `a=b`
pub fn a_eq_b() -> MathList {
    sym("a=b")
}

/// `f(x)`
pub fn f_of_x() -> MathList {
    sym("f(x)")
}

/// `a,b`
pub fn a_comma_b() -> MathList {
    sym("a,b")
}

/// Every fixture with its LaTeX source.
pub fn all() -> Vec<(&'static str, MathList)> {
    vec![
        ("x^2", x_squared()),
        ("x_i^2", x_sub_i_sup_2()),
        ("x^{y^z}", x_sup_y_sup_z()),
        ("\\frac{\\frac{a}{b}}{c}", stacked_fraction()),
        ("\\sqrt{x}", sqrt_x()),
        ("\\sqrt{\\frac{a}{b}}", sqrt_frac()),
        ("\\left(\\frac{a}{b}\\right)", left_right_frac()),
        ("\\sum_{i=1}^n", sum_limits()),
        ("\\int_0^1", int_0_1()),
        ("\\hat{x}", hat_x()),
        ("a+b", a_plus_b()),
        ("a=b", a_eq_b()),
        ("f(x)", f_of_x()),
        ("a,b", a_comma_b()),
    ]
}
