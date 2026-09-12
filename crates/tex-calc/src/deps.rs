//! Dependency tracking for named lengths, and the incremental recompute path
//! this revision proves against a full from-scratch evaluation.
//!
//! [`LengthTable`] is a flat table of named lengths built directly on the
//! same lazy-thunk machinery as [`crate::eval`] (via its `pub(crate)`
//! helpers), so cycle diagnostics, overflow diagnostics, exact scaled-point
//! arithmetic, and the `MAX_RESOLUTION_DEPTH` bound are identical to plain
//! `evaluate` — nothing here re-implements or approximates any of that.
//! Layered on top is a dependency graph computed from each definition's
//! [`free_names`]: redefining a name invalidates exactly its own cached
//! value and every name that transitively depends on it
//! ([`LengthTable::stale_dependents`]) — nothing else — so a later
//! [`LengthTable::get`] only recomputes the stale part of the graph, reusing
//! every other cached value.

use std::collections::{HashMap, HashSet};

use crate::ast::Expr;
use crate::eval::{self, Env};
use crate::sp::{CalcError, Sp};

/// The `\name`s that `expr` reads from an *enclosing* scope — i.e. every
/// `Expr::Name` reference in `expr` that is not itself bound by a `Group`
/// nested inside `expr`. These are exactly the names a definition depends
/// on, respecting the same shadowing rule as evaluation: an inner
/// `\setlength` shadows an outer name of the same spelling, so a reference
/// to that shadowed name is *not* a dependency on the outer one.
pub fn free_names(expr: &Expr) -> HashSet<String> {
    let mut out = HashSet::new();
    collect_free_names(expr, &HashSet::new(), &mut out);
    out
}

fn collect_free_names(expr: &Expr, bound: &HashSet<String>, out: &mut HashSet<String>) {
    match expr {
        Expr::Dim(..) | Expr::Scalar(..) => {}
        Expr::Name(n) => {
            if !bound.contains(n) {
                out.insert(n.clone());
            }
        }
        Expr::Neg(a) => collect_free_names(a, bound, out),
        Expr::Add(a, b) | Expr::Sub(a, b) | Expr::Mul(a, b) | Expr::Div(a, b) => {
            collect_free_names(a, bound, out);
            collect_free_names(b, bound, out);
        }
        Expr::Group(stmts, tail) => {
            let mut inner = bound.clone();
            for s in stmts {
                inner.insert(s.name.clone());
            }
            for s in stmts {
                collect_free_names(&s.expr, &inner, out);
            }
            collect_free_names(tail, &inner, out);
        }
    }
}

/// A flat table of named lengths with tracked dependency edges between them,
/// supporting incremental recomputation after a redefinition.
pub struct LengthTable {
    env: Env,
    order: Vec<String>,
    exprs: HashMap<String, Expr>,
    /// Direct dependencies of each name, per its current definition.
    deps: HashMap<String, HashSet<String>>,
    /// Reverse edges: for each name, the names that directly depend on it.
    dependents: HashMap<String, HashSet<String>>,
}

impl Default for LengthTable {
    fn default() -> Self {
        Self::new()
    }
}

impl LengthTable {
    pub fn new() -> Self {
        LengthTable {
            env: eval::new_env(None),
            order: Vec::new(),
            exprs: HashMap::new(),
            deps: HashMap::new(),
            dependents: HashMap::new(),
        }
    }

    /// Define or redefine a named length. Recomputes `name`'s direct
    /// dependency edges from `expr`'s free names, then invalidates the
    /// cached value of `name` and every name that (transitively) depends on
    /// it — nothing else keeps a stale cached value.
    pub fn define(&mut self, name: &str, expr: Expr) {
        if let Some(old_deps) = self.deps.remove(name) {
            for d in old_deps {
                if let Some(set) = self.dependents.get_mut(&d) {
                    set.remove(name);
                }
            }
        }
        let new_deps = free_names(&expr);
        for d in &new_deps {
            self.dependents
                .entry(d.clone())
                .or_default()
                .insert(name.to_string());
        }
        self.deps.insert(name.to_string(), new_deps);
        if !self.exprs.contains_key(name) {
            self.order.push(name.to_string());
        }
        self.exprs.insert(name.to_string(), expr.clone());
        eval::define_in(&self.env, name, expr);
        self.invalidate_transitive(name);
    }

    /// The names `name` directly reads from, per its current definition.
    /// Empty if `name` is undefined or its definition references nothing
    /// (or only names it shadows locally).
    pub fn direct_dependencies(&self, name: &str) -> HashSet<String> {
        self.deps.get(name).cloned().unwrap_or_default()
    }

    /// Every name that (transitively) depends on `name`, i.e. would need
    /// recomputing if `name`'s definition changed. Does not include `name`
    /// itself.
    pub fn stale_dependents(&self, name: &str) -> HashSet<String> {
        let mut out = HashSet::new();
        let mut stack: Vec<String> = self
            .dependents
            .get(name)
            .cloned()
            .unwrap_or_default()
            .into_iter()
            .collect();
        while let Some(n) = stack.pop() {
            if out.insert(n.clone())
                && let Some(ds) = self.dependents.get(&n)
            {
                stack.extend(ds.iter().cloned());
            }
        }
        out
    }

    fn invalidate_transitive(&mut self, name: &str) {
        let mut stack = vec![name.to_string()];
        let mut seen = HashSet::new();
        while let Some(n) = stack.pop() {
            if !seen.insert(n.clone()) {
                continue;
            }
            if let Some(thunk) = eval::lookup(&self.env, &n) {
                eval::invalidate(&thunk);
            }
            if let Some(ds) = self.dependents.get(&n) {
                stack.extend(ds.iter().cloned());
            }
        }
    }

    /// Resolve `name`'s exact value, reusing whatever is already cached in
    /// this table — the "incremental" half of the fresh-vs-incremental
    /// proof.
    pub fn get(&self, name: &str) -> Result<Sp, CalcError> {
        eval::force_named(&self.env, name).and_then(eval::value_to_sp)
    }

    /// Resolve every defined name incrementally (via [`Self::get`]).
    pub fn eval_all_incremental(&self) -> HashMap<String, Result<Sp, CalcError>> {
        self.order
            .iter()
            .map(|name| (name.clone(), self.get(name)))
            .collect()
    }

    /// Resolve every defined name in a brand-new environment that shares no
    /// cache with this table — a full from-scratch evaluation of the exact
    /// same definitions, for comparison against
    /// [`Self::eval_all_incremental`].
    pub fn eval_all_fresh(&self) -> HashMap<String, Result<Sp, CalcError>> {
        let fresh = eval::new_env(None);
        for name in &self.order {
            if let Some(expr) = self.exprs.get(name) {
                eval::define_in(&fresh, name, expr.clone());
            }
        }
        self.order
            .iter()
            .map(|name| {
                (
                    name.clone(),
                    eval::force_named(&fresh, name).and_then(eval::value_to_sp),
                )
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sp::Sp;

    fn def(table: &mut LengthTable, name: &str, src: &str) {
        table.define(name, crate::parser::parse(src).unwrap());
    }

    fn assert_fresh_matches_incremental(table: &LengthTable) {
        assert_eq!(table.eval_all_fresh(), table.eval_all_incremental());
    }

    #[test]
    fn free_names_respects_inner_shadowing() {
        // The group's own `\a` shadows the outer one; only the trailing
        // `+ \a` outside the group is a real free reference.
        let expr = crate::parser::parse(r"{ \setlength{\a}{100pt}; \a } + \a").unwrap();
        let names = free_names(&expr);
        assert_eq!(names, ["a".to_string()].into_iter().collect());
    }

    #[test]
    fn free_names_of_fully_shadowed_expr_is_empty() {
        let expr = crate::parser::parse(r"{ \setlength{\a}{100pt}; \a }").unwrap();
        assert!(free_names(&expr).is_empty());
    }

    // --- Graph 1: a linear chain a <- b <- c <- d. ---
    #[test]
    fn linear_chain_fresh_matches_incremental_after_change() {
        let mut t = LengthTable::new();
        def(&mut t, "a", "1pt");
        def(&mut t, "b", r"\a + 1pt");
        def(&mut t, "c", r"\b + 1pt");
        def(&mut t, "d", r"\c + 1pt");
        assert_fresh_matches_incremental(&t);
        assert_eq!(t.get("d").unwrap(), Sp(4 * 65536));

        def(&mut t, "a", "10pt");
        let expected_stale: HashSet<String> =
            ["b", "c", "d"].iter().map(|s| s.to_string()).collect();
        assert_eq!(t.stale_dependents("a"), expected_stale);
        assert_fresh_matches_incremental(&t);
        assert_eq!(t.get("d").unwrap(), Sp(13 * 65536));
    }

    // --- Graph 2: a diamond, two paths from `a` converging on `d`. ---
    #[test]
    fn diamond_dependency_fresh_matches_incremental_after_change() {
        let mut t = LengthTable::new();
        def(&mut t, "a", "1pt");
        def(&mut t, "b", r"\a * 2");
        def(&mut t, "c", r"\a + 3pt");
        def(&mut t, "d", r"\b + \c");
        assert_fresh_matches_incremental(&t);
        assert_eq!(t.get("d").unwrap(), Sp(6 * 65536)); // 2pt + 4pt

        def(&mut t, "a", "5pt");
        assert_fresh_matches_incremental(&t);
        assert_eq!(t.get("d").unwrap(), Sp(18 * 65536)); // 10pt + 8pt
    }

    // --- Graph 3: an unrelated branch must not be disturbed. ---
    #[test]
    fn unrelated_branch_is_not_marked_stale_or_recomputed_differently() {
        let mut t = LengthTable::new();
        def(&mut t, "a", "1pt");
        def(&mut t, "b", r"\a + 1pt");
        def(&mut t, "x", "42pt");
        assert_fresh_matches_incremental(&t);

        def(&mut t, "a", "2pt");
        assert!(!t.stale_dependents("a").contains("x"));
        assert_eq!(t.get("x").unwrap(), Sp(42 * 65536));
        assert_fresh_matches_incremental(&t);
    }

    // --- Graph 4: a definition whose only length reference is shadowed by
    // its own inner group must not depend on the outer table entry. ---
    #[test]
    fn shadowed_inner_definition_creates_no_false_dependency() {
        let mut t = LengthTable::new();
        def(&mut t, "a", "1pt");
        def(&mut t, "c", r"{ \setlength{\a}{100pt}; \a }");
        assert!(t.direct_dependencies("c").is_empty());
        assert!(!t.stale_dependents("a").contains("c"));
        assert_eq!(t.get("c").unwrap(), Sp(100 * 65536));

        def(&mut t, "a", "999pt");
        assert_eq!(t.get("c").unwrap(), Sp(100 * 65536));
        assert_fresh_matches_incremental(&t);
    }

    // --- Graph 5: a three-length cycle, then broken and pushed to overflow
    // — fresh and incremental must agree on both typed errors. ---
    #[test]
    fn cycle_then_overflow_diagnostics_agree_between_fresh_and_incremental() {
        let mut t = LengthTable::new();
        def(&mut t, "a", r"\b + 1pt");
        def(&mut t, "b", r"\c + 1pt");
        def(&mut t, "c", r"\a + 1pt");
        let fresh = t.eval_all_fresh();
        assert_eq!(fresh, t.eval_all_incremental());
        match &fresh["a"] {
            Err(CalcError::CyclicLength(chain)) => assert_eq!(
                chain,
                &vec![
                    "a".to_string(),
                    "b".to_string(),
                    "c".to_string(),
                    "a".to_string()
                ]
            ),
            other => panic!("expected a cyclic-length error, got {other:?}"),
        }

        // Break the cycle, then push a value through the (now acyclic)
        // chain that overflows `b`'s own addition.
        def(&mut t, "c", "16383pt");
        assert_fresh_matches_incremental(&t);
        match t.get("b") {
            Err(CalcError::Overflow(info)) => assert_eq!(info.op, "+"),
            other => panic!("expected a typed overflow, got {other:?}"),
        }
        // `a` depends on `b` and must surface the identical overflow.
        assert_eq!(t.get("a"), t.get("b"));
        assert_fresh_matches_incremental(&t);
    }
}
