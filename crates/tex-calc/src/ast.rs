//! The expression AST. A `Group` (`{ ... }`) is TeX's own scoping construct:
//! `\setlength` statements inside it bind names only for the rest of that
//! group, exactly mirroring how TeX's own local assignments do not survive
//! past the closing brace.

use crate::sp::Unit;

#[derive(Clone, Debug)]
pub enum Expr {
    /// A literal dimension: `numerator/denominator` in the given unit.
    Dim(i128, i128, Unit),
    /// A bare dimensionless number (only valid as a `*`/`/` operand).
    Scalar(i128, i128),
    /// A reference to a named length, e.g. `\foo`.
    Name(String),
    Neg(Box<Expr>),
    Add(Box<Expr>, Box<Expr>),
    Sub(Box<Expr>, Box<Expr>),
    Mul(Box<Expr>, Box<Expr>),
    Div(Box<Expr>, Box<Expr>),
    /// `{ stmt* expr }`: a new scope whose `\setlength` bindings do not leak
    /// past the closing brace; its value is the trailing expression's.
    Group(Vec<Stmt>, Box<Expr>),
}

#[derive(Clone, Debug)]
pub struct Stmt {
    pub name: String,
    pub expr: Expr,
}
