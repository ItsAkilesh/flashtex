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
//!
//! Non-convergence is not one failure mode. [`ConvergenceError`]
//! distinguishes two shapes a caller must react to differently:
//!
//! - [`ConvergenceError::Cycle`]: a candidate seen earlier in the history
//!   recurred (e.g. a two-state oscillation `A -> B -> A -> B -> ...`). The
//!   model is proven to loop forever; more passes would never help.
//! - [`ConvergenceError::Unresolved`]: every candidate tried within the
//!   bound was distinct — the page count kept changing without ever
//!   settling *or* repeating. More passes might help, or the model might
//!   diverge; this crate does not guess which.
//!
//! Both variants carry the full `bound` and `history` so a caller can
//! inspect exactly what was tried.

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
/// [`MAX_CONVERGENCE_ITERATIONS`] passes. See the module docs for how
/// [`Cycle`](ConvergenceError::Cycle) and
/// [`Unresolved`](ConvergenceError::Unresolved) differ.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConvergenceError {
    /// Every one of the `bound + 1` candidates tried was distinct: the
    /// page count never repeated a prior value and never settled.
    Unresolved {
        bound: usize,
        /// Every candidate tried, in order, starting with the initial guess.
        history: Vec<u32>,
    },
    /// `history[cycle_start..]` (before the repeat) is the smallest proven
    /// repeating segment: `history[cycle_start]` recurred later in
    /// `history`, and because [`FrontMatterModel::pages_for`] is a pure
    /// function of its candidate, that guarantees the model loops through
    /// exactly `cycle` forever from `cycle_start` on.
    Cycle {
        bound: usize,
        /// Every candidate tried, in order, starting with the initial guess.
        history: Vec<u32>,
        /// Index into `history` where the repeating segment starts.
        cycle_start: usize,
        /// The repeating segment itself, e.g. `[2, 5]` for `A -> B -> A ->
        /// B -> ...`.
        cycle: Vec<u32>,
    },
}

impl ConvergenceError {
    /// The iteration bound this error was produced under.
    pub fn bound(&self) -> usize {
        match self {
            ConvergenceError::Unresolved { bound, .. } | ConvergenceError::Cycle { bound, .. } => {
                *bound
            }
        }
    }

    /// Every candidate tried, in order, starting with the initial guess.
    pub fn history(&self) -> &[u32] {
        match self {
            ConvergenceError::Unresolved { history, .. }
            | ConvergenceError::Cycle { history, .. } => history,
        }
    }
}

impl std::fmt::Display for ConvergenceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConvergenceError::Unresolved { bound, history } => write!(
                f,
                "front-matter pagination did not converge within {bound} passes and never repeated a prior value; history: {history:?}"
            ),
            ConvergenceError::Cycle {
                bound,
                history,
                cycle_start,
                cycle,
            } => write!(
                f,
                "front-matter pagination cycles instead of converging: {cycle:?} repeating forever from pass {cycle_start} (bound {bound}); history: {history:?}"
            ),
        }
    }
}

impl std::error::Error for ConvergenceError {}

/// Finds a stable front-matter page count: repeatedly asks `model` what
/// page count a candidate implies, stopping as soon as a candidate maps to
/// itself. Bounded by [`MAX_CONVERGENCE_ITERATIONS`]; returns
/// [`ConvergenceError`] rather than looping forever or accepting an
/// unstable answer. Runs the full bound of passes even once a repeat could
/// be detected early, so `history` (and therefore the result) never
/// depends on how eagerly a caller might have wanted to bail out — only on
/// `model` and `initial_guess`.
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

    Err(classify_non_convergence(
        MAX_CONVERGENCE_ITERATIONS,
        history,
    ))
}

/// Classifies a completed (non-converging) history as a proven [`Cycle`]
/// or an [`Unresolved`] open sequence, by finding the earliest value that
/// recurs later in `history`.
///
/// [`Cycle`]: ConvergenceError::Cycle
/// [`Unresolved`]: ConvergenceError::Unresolved
fn classify_non_convergence(bound: usize, history: Vec<u32>) -> ConvergenceError {
    for later in 1..history.len() {
        if let Some(earlier) = history[..later].iter().position(|&v| v == history[later]) {
            let cycle = history[earlier..later].to_vec();
            return ConvergenceError::Cycle {
                bound,
                history,
                cycle_start: earlier,
                cycle,
            };
        }
    }
    ConvergenceError::Unresolved { bound, history }
}
