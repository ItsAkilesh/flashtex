//! Probes, at build time, for a real TeX binary in the same fixed set of
//! absolute paths `tests/tex_oracle.rs`'s own `find_tex()` checks at runtime,
//! and exposes the result as the `tex_oracle_available` cfg.
//!
//! Why this exists: the oracle tests already skip cleanly (print a message
//! and return) when `find_tex()` comes up empty, but a test function that
//! just returns `Ok(())` without asserting anything is reported by cargo's
//! test harness as **passed** -- identical, in the summary count, to a test
//! that actually ran the comparison and passed. Since this project publishes
//! test counts as evidence, that is misleading: the count would look the
//! same whether or not a real TeX ever ran. Tagging each oracle test
//! `#[cfg_attr(not(tex_oracle_available), ignore = "...")]` (see
//! `tests/tex_oracle.rs`) makes a missing `tex` show up as "ignored" in the
//! summary instead, which is what actually happened.
//!
//! The runtime `find_tex()` check inside each test is kept as-is even though
//! this cfg now gates the same tests: it is a harmless safety net for the
//! edge case where `tex` is installed or removed between building the test
//! binary and running it, and it is what actually locates the binary to
//! invoke, not just whether one exists.

use std::path::PathBuf;

fn find_tex() -> Option<PathBuf> {
    [
        "/Library/TeX/texbin/tex",
        "/usr/local/texlive/2026/bin/universal-darwin/tex",
        "/usr/local/texlive/2025/bin/universal-darwin/tex",
        "/usr/local/texlive/2024/bin/universal-darwin/tex",
        "/usr/bin/tex",
        "/usr/local/bin/tex",
    ]
    .iter()
    .map(PathBuf::from)
    .find(|p| p.is_file())
}

fn main() {
    // Declare the cfg so `--cfg tex_oracle_available` (or its absence) never
    // trips the `unexpected_cfg` lint.
    println!("cargo::rustc-check-cfg=cfg(tex_oracle_available)");
    if find_tex().is_some() {
        println!("cargo::rustc-cfg=tex_oracle_available");
    }
    // The candidate paths are fixed absolute locations outside this crate,
    // not inputs cargo can watch; only re-run if this probe logic itself
    // changes.
    println!("cargo::rerun-if-changed=build.rs");
}
