//! Bounded, revision-safe convergence for the classic contents-list
//! circularity: a `\tableofcontents`/`\listoffigures` entry's absolute page
//! number depends on how many pages the front matter occupies, but the
//! front matter's own page count can depend on what got typeset into it —
//! same shape as LaTeX needing repeated passes to stabilize `.toc`/`.lof`
//! cross-references.
//!
//! This module never approximates that fixed point and never loops
//! unbounded: [`converge_front_matter_pages`] iterates a caller-supplied
//! [`FrontMatterModel`] at most [`MAX_CONVERGENCE_ITERATIONS`] times and
//! returns a typed [`ConvergenceError`] — carrying the full guess history —
//! the moment that bound is reached without a repeated (stable) value.

/// The typed adapter contract for one revision (pass) of front-matter
/// pagination: given a candidate page count for the front matter, return
/// how many pages the front matter is actually computed to need. A
/// consumer (compiler/native layout) implements this against its own
/// pagination; this crate only drives the fixed-point search.
pub trait FrontMatterModel {
    /// Must be a pure function of `candidate` (and `self`): calling it
    /// twice with the same candidate must return the same answer, or
    /// convergence has no meaning.
    fn pages_for(&self, candidate_front_matter_pages: u32) -> u32;
}

/// Hard bound on convergence passes. Chosen generously above the 2-3
/// passes real TOC/LOF pagination needs in practice; reaching it is treated
/// as non-convergence, never as "close enough".
pub const MAX_CONVERGENCE_ITERATIONS: usize = 8;

/// The model did not settle on a stable front-matter page count within
/// [`MAX_CONVERGENCE_ITERATIONS`] passes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConvergenceError {
    pub bound: usize,
    /// Every candidate tried, in order, starting with the initial guess.
    pub history: Vec<u32>,
}

impl std::fmt::Display for ConvergenceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "front-matter pagination did not converge within {} passes; history: {:?}",
            self.bound, self.history
        )
    }
}

impl std::error::Error for ConvergenceError {}

/// Finds a stable front-matter page count: repeatedly asks `model` what
/// page count a candidate implies, stopping as soon as a candidate maps to
/// itself. Bounded by [`MAX_CONVERGENCE_ITERATIONS`]; returns
/// [`ConvergenceError`] rather than looping forever or accepting an
/// unstable answer.
pub fn converge_front_matter_pages(
    model: &dyn FrontMatterModel,
    initial_guess: u32,
) -> Result<u32, ConvergenceError> {
    let mut history = Vec::with_capacity(MAX_CONVERGENCE_ITERATIONS + 1);
    let mut current = initial_guess;
    history.push(current);

    for _ in 0..MAX_CONVERGENCE_ITERATIONS {
        let next = model.pages_for(current);
        if next == current {
            return Ok(current);
        }
        current = next;
        history.push(current);
    }

    Err(ConvergenceError {
        bound: MAX_CONVERGENCE_ITERATIONS,
        history,
    })
}
