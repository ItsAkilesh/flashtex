//! Evaluation: scoped named lengths and typed arithmetic.
//!
//! Named lengths are lazy thunks stored in a shared, mutable per-`{}`-group
//! frame (`Rc<RefCell<Frame>>`). Forcing a thunk "blackholes" it (marks it
//! `InProgress`) for the duration of its own evaluation; if resolving it
//! requires resolving itself again — directly or through any chain of other
//! names — the second force sees `InProgress` and returns a typed
//! [`CalcError::CyclicLength`] naming the full chain (see `stack` below)
//! instead of recursing forever. A hard [`MAX_RESOLUTION_DEPTH`] backstops
//! any other unbounded nesting.
//!
//! The `stack` threaded through [`eval_expr`]/`force` records, in resolution
//! order, which named lengths are currently being forced. When a cycle is
//! detected the slice of `stack` from that name's first occurrence to the
//! present — plus the name repeated once more to show where the loop closes
//! — becomes the [`CalcError::CyclicLength`] chain, e.g. `[a, b, c, a]`.
//!
//! A handful of items below are `pub(crate)` rather than private so
//! [`crate::deps::LengthTable`] can build a dependency-tracked, incrementally
//! recomputed table of named lengths directly on top of this same thunk
//! machinery — sharing its exact arithmetic, cycle detection, and depth
//! bound rather than re-implementing any of it.

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::ast::Expr;
use crate::sp::{CalcError, Sp};

/// Upper bound on evaluation/resolution nesting. Chosen generously above any
/// realistic expression while still being small enough to fail fast (and
/// well within Rust's default stack) instead of overflowing the stack on a
/// pathological input.
pub const MAX_RESOLUTION_DEPTH: usize = 200;

/// An evaluated value: either an exact dimension or a dimensionless scalar
/// (only meaningful as a `*`/`/` operand).
#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    Dim(Sp),
    Scalar(i128, i128),
}

pub(crate) type Env = Rc<RefCell<Frame>>;

pub(crate) struct Frame {
    parent: Option<Env>,
    bindings: HashMap<String, Rc<Thunk>>,
}

pub(crate) struct Thunk {
    expr: Expr,
    env: Env,
    state: RefCell<ThunkState>,
}

enum ThunkState {
    Pending,
    InProgress,
    Done(Result<Value, CalcError>),
}

pub(crate) fn new_env(parent: Option<Env>) -> Env {
    Rc::new(RefCell::new(Frame {
        parent,
        bindings: HashMap::new(),
    }))
}

pub(crate) fn lookup(env: &Env, name: &str) -> Option<Rc<Thunk>> {
    let mut current = Rc::clone(env);
    loop {
        let next = {
            let frame = current.borrow();
            if let Some(t) = frame.bindings.get(name) {
                return Some(Rc::clone(t));
            }
            frame.parent.clone()
        };
        current = next?;
    }
}

/// Bind `name` to `expr` (as a fresh, not-yet-forced thunk) directly in
/// `env`, replacing any existing binding of that name in this exact frame.
/// Used by [`crate::deps::LengthTable`] to build and redefine its table.
pub(crate) fn define_in(env: &Env, name: &str, expr: Expr) {
    let thunk = Rc::new(Thunk {
        expr,
        env: Rc::clone(env),
        state: RefCell::new(ThunkState::Pending),
    });
    env.borrow_mut().bindings.insert(name.to_string(), thunk);
}

/// Reset an already-bound thunk back to `Pending`, discarding any cached
/// value so the next force recomputes it (and, transitively, whatever reads
/// it). Used by [`crate::deps::LengthTable`] to invalidate exactly the
/// stale part of its dependency graph after a redefinition.
pub(crate) fn invalidate(thunk: &Rc<Thunk>) {
    *thunk.state.borrow_mut() = ThunkState::Pending;
}

/// Look up and force `name` in `env` from a fresh resolution stack. Used by
/// [`crate::deps::LengthTable`] as its externally-visible "resolve one named
/// length" operation.
pub(crate) fn force_named(env: &Env, name: &str) -> Result<Value, CalcError> {
    let thunk = lookup(env, name).ok_or_else(|| CalcError::UndefinedLength(name.to_string()))?;
    let mut stack = Vec::new();
    force(&thunk, name, 1, &mut stack)
}

fn force(
    thunk: &Rc<Thunk>,
    name: &str,
    depth: usize,
    stack: &mut Vec<String>,
) -> Result<Value, CalcError> {
    if depth > MAX_RESOLUTION_DEPTH {
        return Err(CalcError::ResolutionDepthExceeded);
    }
    {
        let state = thunk.state.borrow();
        match &*state {
            ThunkState::Done(v) => return v.clone(),
            ThunkState::InProgress => {
                // `name` is already being forced somewhere below us on
                // `stack`: the cycle is that suffix, plus `name` once more
                // to show where it closes, e.g. [a, b, c, a].
                let start = stack.iter().position(|n| n == name).unwrap_or(0);
                let mut chain: Vec<String> = stack[start..].to_vec();
                chain.push(name.to_string());
                return Err(CalcError::CyclicLength(chain));
            }
            ThunkState::Pending => {}
        }
    }
    *thunk.state.borrow_mut() = ThunkState::InProgress;
    stack.push(name.to_string());
    let result = eval_expr(&thunk.expr, &thunk.env, depth + 1, stack);
    stack.pop();
    *thunk.state.borrow_mut() = ThunkState::Done(result.clone());
    result
}

/// Evaluate a parsed expression to an exact dimension. A top-level result
/// that is a dimensionless scalar (e.g. the whole input was just `2 + 2`) is
/// a typed [`CalcError::TypeMismatch`], not an implicit `0pt`.
pub fn eval(expr: &Expr) -> Result<Sp, CalcError> {
    let env = new_env(None);
    let mut stack = Vec::new();
    value_to_sp(eval_expr(expr, &env, 0, &mut stack)?)
}

/// Unwrap a [`Value`] to the exact dimension it must be at the point a named
/// length (or a whole expression) is expected to have settled to one:
/// [`CalcError::TypeMismatch`], not an implicit `0pt`, if it is instead a
/// dimensionless scalar.
pub(crate) fn value_to_sp(v: Value) -> Result<Sp, CalcError> {
    match v {
        Value::Dim(sp) => Ok(sp),
        Value::Scalar(n, d) => Err(CalcError::TypeMismatch(format!(
            "expression has no unit (dimensionless value {n}/{d})"
        ))),
    }
}

fn eval_expr(
    expr: &Expr,
    env: &Env,
    depth: usize,
    stack: &mut Vec<String>,
) -> Result<Value, CalcError> {
    if depth > MAX_RESOLUTION_DEPTH {
        return Err(CalcError::ResolutionDepthExceeded);
    }
    match expr {
        Expr::Dim(n, d, unit) => Ok(Value::Dim(unit.to_sp(*n, *d)?)),
        Expr::Scalar(n, d) => Ok(Value::Scalar(*n, *d)),
        Expr::Name(name) => {
            let thunk =
                lookup(env, name).ok_or_else(|| CalcError::UndefinedLength(name.clone()))?;
            force(&thunk, name, depth + 1, stack)
        }
        Expr::Neg(inner) => match eval_expr(inner, env, depth + 1, stack)? {
            Value::Dim(sp) => Ok(Value::Dim(sp.checked_neg()?)),
            Value::Scalar(n, d) => Ok(Value::Scalar(-n, d)),
        },
        Expr::Add(a, b) => {
            let (a, b) = (
                eval_expr(a, env, depth + 1, stack)?,
                eval_expr(b, env, depth + 1, stack)?,
            );
            add_or_sub(a, b, true)
        }
        Expr::Sub(a, b) => {
            let (a, b) = (
                eval_expr(a, env, depth + 1, stack)?,
                eval_expr(b, env, depth + 1, stack)?,
            );
            add_or_sub(a, b, false)
        }
        Expr::Mul(a, b) => {
            let (a, b) = (
                eval_expr(a, env, depth + 1, stack)?,
                eval_expr(b, env, depth + 1, stack)?,
            );
            mul(a, b)
        }
        Expr::Div(a, b) => {
            let (a, b) = (
                eval_expr(a, env, depth + 1, stack)?,
                eval_expr(b, env, depth + 1, stack)?,
            );
            div(a, b)
        }
        Expr::Group(stmts, tail) => {
            let child = new_env(Some(Rc::clone(env)));
            for stmt in stmts {
                let thunk = Rc::new(Thunk {
                    expr: stmt.expr.clone(),
                    env: Rc::clone(&child),
                    state: RefCell::new(ThunkState::Pending),
                });
                child.borrow_mut().bindings.insert(stmt.name.clone(), thunk);
            }
            eval_expr(tail, &child, depth + 1, stack)
        }
    }
}

fn overflow_scalar_op(op: &str, n1: i128, d1: i128, n2: i128, d2: i128) -> CalcError {
    CalcError::Overflow(crate::sp::OverflowInfo {
        op: op.to_string(),
        operands: vec![format!("{n1}/{d1}"), format!("{n2}/{d2}")],
    })
}

fn checked_scalar_mul(n1: i128, d1: i128, n2: i128, d2: i128) -> Result<(i128, i128), CalcError> {
    let n = n1
        .checked_mul(n2)
        .ok_or_else(|| overflow_scalar_op("*", n1, d1, n2, d2))?;
    let d = d1
        .checked_mul(d2)
        .ok_or_else(|| overflow_scalar_op("*", n1, d1, n2, d2))?;
    Ok((n, d))
}

fn add_or_sub(a: Value, b: Value, is_add: bool) -> Result<Value, CalcError> {
    match (a, b) {
        (Value::Dim(x), Value::Dim(y)) => Ok(Value::Dim(if is_add {
            x.checked_add(y)?
        } else {
            x.checked_sub(y)?
        })),
        (Value::Scalar(n1, d1), Value::Scalar(n2, d2)) => {
            // n1/d1 +/- n2/d2 = (n1*d2 +/- n2*d1) / (d1*d2)
            let op = if is_add { "+" } else { "-" };
            let mk_err = || overflow_scalar_op(op, n1, d1, n2, d2);
            let n1d2 = n1.checked_mul(d2).ok_or_else(mk_err)?;
            let n2d1 = n2.checked_mul(d1).ok_or_else(mk_err)?;
            let n = if is_add {
                n1d2.checked_add(n2d1)
            } else {
                n1d2.checked_sub(n2d1)
            }
            .ok_or_else(mk_err)?;
            let d = d1.checked_mul(d2).ok_or_else(mk_err)?;
            Ok(Value::Scalar(n, d))
        }
        (Value::Dim(_), Value::Scalar(n, d)) | (Value::Scalar(n, d), Value::Dim(_)) => {
            Err(CalcError::TypeMismatch(format!(
                "cannot add/subtract a dimension and the dimensionless scalar {n}/{d}"
            )))
        }
    }
}

fn mul(a: Value, b: Value) -> Result<Value, CalcError> {
    match (a, b) {
        (Value::Dim(x), Value::Scalar(n, d)) | (Value::Scalar(n, d), Value::Dim(x)) => {
            Ok(Value::Dim(x.checked_mul_scalar(n, d)?))
        }
        (Value::Scalar(n1, d1), Value::Scalar(n2, d2)) => {
            let (n, d) = checked_scalar_mul(n1, d1, n2, d2)?;
            Ok(Value::Scalar(n, d))
        }
        (Value::Dim(_), Value::Dim(_)) => Err(CalcError::TypeMismatch(
            "cannot multiply two dimensions together".to_string(),
        )),
    }
}

fn div(a: Value, b: Value) -> Result<Value, CalcError> {
    match (a, b) {
        (Value::Dim(x), Value::Scalar(n, d)) => Ok(Value::Dim(x.checked_div_scalar(n, d)?)),
        (Value::Scalar(n1, d1), Value::Scalar(n2, d2)) => {
            // Same contract as `Sp::checked_div_scalar`'s two checks,
            // applied to the divisor `n2/d2` here: a zero-valued divisor
            // (`n2 == 0`) is a typed error, and so is a non-positive `d2` --
            // the public `Expr::Scalar` constructor is not restricted to
            // what the parser would produce, so a caller can build a
            // divisor like `1/0` directly. Unlike `checked_div_scalar`,
            // `d2` here is a multiplicand (`d1.checked_mul(n2)`), not a
            // literal divisor, so left unchecked it never panics -- it
            // silently zeroes out the numerator (`n1 * d2 = n1 * 0 = 0`)
            // and returns the wrong `Ok` scalar `0` instead of erroring.
            if d2 <= 0 || n2 == 0 {
                return Err(CalcError::DivisionByZero);
            }
            // (n1/d1) / (n2/d2) = (n1*d2) / (d1*n2), sign normalized so the
            // denominator stays positive.
            let mk_err = || overflow_scalar_op("/", n1, d1, n2, d2);
            let mut n = n1.checked_mul(d2).ok_or_else(mk_err)?;
            let mut d = d1.checked_mul(n2).ok_or_else(mk_err)?;
            if d < 0 {
                n = -n;
                d = -d;
            }
            Ok(Value::Scalar(n, d))
        }
        (Value::Scalar(_, _), Value::Dim(_)) => Err(CalcError::TypeMismatch(
            "cannot divide a scalar by a dimension".to_string(),
        )),
        (Value::Dim(_), Value::Dim(_)) => Err(CalcError::TypeMismatch(
            "dividing a dimension by a dimension is not supported".to_string(),
        )),
    }
}
