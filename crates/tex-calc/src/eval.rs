//! Evaluation: scoped named lengths and typed arithmetic.
//!
//! Named lengths are lazy thunks stored in a shared, mutable per-`{}`-group
//! frame (`Rc<RefCell<Frame>>`). Forcing a thunk "blackholes" it (marks it
//! `InProgress`) for the duration of its own evaluation; if resolving it
//! requires resolving itself again — directly or through any chain of other
//! names — the second force sees `InProgress` and returns a typed
//! [`CalcError::CyclicLength`] instead of recursing forever. A hard
//! [`MAX_RESOLUTION_DEPTH`] backstops any other unbounded nesting.

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

type Env = Rc<RefCell<Frame>>;

struct Frame {
    parent: Option<Env>,
    bindings: HashMap<String, Rc<Thunk>>,
}

struct Thunk {
    expr: Expr,
    env: Env,
    state: RefCell<ThunkState>,
}

enum ThunkState {
    Pending,
    InProgress,
    Done(Result<Value, CalcError>),
}

fn new_env(parent: Option<Env>) -> Env {
    Rc::new(RefCell::new(Frame {
        parent,
        bindings: HashMap::new(),
    }))
}

fn lookup(env: &Env, name: &str) -> Option<Rc<Thunk>> {
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

fn force(thunk: &Rc<Thunk>, name: &str, depth: usize) -> Result<Value, CalcError> {
    if depth > MAX_RESOLUTION_DEPTH {
        return Err(CalcError::ResolutionDepthExceeded);
    }
    {
        let state = thunk.state.borrow();
        match &*state {
            ThunkState::Done(v) => return v.clone(),
            ThunkState::InProgress => return Err(CalcError::CyclicLength(name.to_string())),
            ThunkState::Pending => {}
        }
    }
    *thunk.state.borrow_mut() = ThunkState::InProgress;
    let result = eval_expr(&thunk.expr, &thunk.env, depth + 1);
    *thunk.state.borrow_mut() = ThunkState::Done(result.clone());
    result
}

/// Evaluate a parsed expression to an exact dimension. A top-level result
/// that is a dimensionless scalar (e.g. the whole input was just `2 + 2`) is
/// a typed [`CalcError::TypeMismatch`], not an implicit `0pt`.
pub fn eval(expr: &Expr) -> Result<Sp, CalcError> {
    let env = new_env(None);
    match eval_expr(expr, &env, 0)? {
        Value::Dim(sp) => Ok(sp),
        Value::Scalar(n, d) => Err(CalcError::TypeMismatch(format!(
            "expression has no unit (dimensionless value {n}/{d})"
        ))),
    }
}

fn eval_expr(expr: &Expr, env: &Env, depth: usize) -> Result<Value, CalcError> {
    if depth > MAX_RESOLUTION_DEPTH {
        return Err(CalcError::ResolutionDepthExceeded);
    }
    match expr {
        Expr::Dim(n, d, unit) => Ok(Value::Dim(unit.to_sp(*n, *d)?)),
        Expr::Scalar(n, d) => Ok(Value::Scalar(*n, *d)),
        Expr::Name(name) => {
            let thunk =
                lookup(env, name).ok_or_else(|| CalcError::UndefinedLength(name.clone()))?;
            force(&thunk, name, depth + 1)
        }
        Expr::Neg(inner) => match eval_expr(inner, env, depth + 1)? {
            Value::Dim(sp) => Ok(Value::Dim(sp.checked_neg()?)),
            Value::Scalar(n, d) => Ok(Value::Scalar(-n, d)),
        },
        Expr::Add(a, b) => {
            let (a, b) = (eval_expr(a, env, depth + 1)?, eval_expr(b, env, depth + 1)?);
            add_or_sub(a, b, true)
        }
        Expr::Sub(a, b) => {
            let (a, b) = (eval_expr(a, env, depth + 1)?, eval_expr(b, env, depth + 1)?);
            add_or_sub(a, b, false)
        }
        Expr::Mul(a, b) => {
            let (a, b) = (eval_expr(a, env, depth + 1)?, eval_expr(b, env, depth + 1)?);
            mul(a, b)
        }
        Expr::Div(a, b) => {
            let (a, b) = (eval_expr(a, env, depth + 1)?, eval_expr(b, env, depth + 1)?);
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
            eval_expr(tail, &child, depth + 1)
        }
    }
}

fn checked_scalar_mul(n1: i128, d1: i128, n2: i128, d2: i128) -> Result<(i128, i128), CalcError> {
    let n = n1.checked_mul(n2).ok_or(CalcError::Overflow)?;
    let d = d1.checked_mul(d2).ok_or(CalcError::Overflow)?;
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
            let n1d2 = n1.checked_mul(d2).ok_or(CalcError::Overflow)?;
            let n2d1 = n2.checked_mul(d1).ok_or(CalcError::Overflow)?;
            let n = if is_add {
                n1d2.checked_add(n2d1)
            } else {
                n1d2.checked_sub(n2d1)
            }
            .ok_or(CalcError::Overflow)?;
            let d = d1.checked_mul(d2).ok_or(CalcError::Overflow)?;
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
            if n2 == 0 {
                return Err(CalcError::DivisionByZero);
            }
            // (n1/d1) / (n2/d2) = (n1*d2) / (d1*n2), sign normalized so the
            // denominator stays positive.
            let mut n = n1.checked_mul(d2).ok_or(CalcError::Overflow)?;
            let mut d = d1.checked_mul(n2).ok_or(CalcError::Overflow)?;
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
