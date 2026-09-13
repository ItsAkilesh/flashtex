//! Records the Git revision the binary was built from as `FLASHTEX_GIT_SHA`
//! (for `flashtex --version`). `FLASHTEX_GIT_SHA` in the build environment
//! wins (an archive export has no repository); otherwise `git rev-parse` of
//! the crate's checkout, with `-dirty` when the tree has local changes;
//! `unknown` when neither is available. Never fails the build.

use std::process::Command;

fn git(args: &[&str]) -> Option<String> {
    let out = Command::new("git").args(args).current_dir(env!("CARGO_MANIFEST_DIR")).output().ok()?;
    if !out.status.success() {
        return None;
    }
    let s = String::from_utf8(out.stdout).ok()?;
    let s = s.trim().to_string();
    (!s.is_empty()).then_some(s)
}

fn main() {
    println!("cargo:rerun-if-env-changed=FLASHTEX_GIT_SHA");
    let sha = std::env::var("FLASHTEX_GIT_SHA").ok().filter(|s| !s.is_empty()).unwrap_or_else(|| {
        let short = git(&["rev-parse", "--short=12", "HEAD"]).unwrap_or_else(|| "unknown".into());
        // `status --porcelain` is empty on a clean tree; a failure (no
        // repository) leaves the plain revision.
        let dirty = git(&["status", "--porcelain", "--untracked-files=no", "--", "."]).is_some();
        if short != "unknown" && dirty {
            format!("{short}-dirty")
        } else {
            short
        }
    });
    println!("cargo:rustc-env=FLASHTEX_GIT_SHA={sha}");
    // Rebuild when HEAD moves so the SHA stays truthful in a checkout.
    if let Some(dir) = git(&["rev-parse", "--git-dir"]) {
        let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(dir);
        for f in ["HEAD", "index"] {
            let p = dir.join(f);
            if p.exists() {
                println!("cargo:rerun-if-changed={}", p.display());
            }
        }
    }
}
