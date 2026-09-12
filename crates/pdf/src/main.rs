//! `flashtex-pdf`: read a runtime-v1 `compile_result` envelope, write a PDF.
//!
//! ```text
//! flashtex-pdf --out out.pdf < compile-result.json
//! flashtex-pdf compile-result.json --out out.pdf
//! ```
//!
//! Warnings (substituted characters, skipped item kinds) go to stderr, one per
//! line, prefixed with `warning:`. Exit status is 0 on success, 1 when the input
//! is not a supported envelope or cannot be rendered, 2 on usage errors.

use std::io::Read;
use std::process::ExitCode;

const USAGE: &str = "usage: flashtex-pdf [INPUT.json] --out OUTPUT.pdf [--verify]\n\
       Reads a runtime-v1 compile_result envelope (from INPUT.json or stdin) and writes a PDF.";

fn main() -> ExitCode {
    let mut out_path = None;
    let mut input_path = None;
    let mut verify = false;
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--out" | "-o" => match args.next() {
                Some(p) => out_path = Some(p),
                None => return usage("--out needs a path"),
            },
            "--verify" => verify = true,
            "-h" | "--help" => {
                println!("{USAGE}");
                return ExitCode::SUCCESS;
            }
            s if s.starts_with('-') => return usage(&format!("unknown option {s}")),
            _ if input_path.is_none() => input_path = Some(arg),
            _ => return usage("only one input path is accepted"),
        }
    }
    let Some(out_path) = out_path else {
        return usage("--out is required");
    };

    let input = match &input_path {
        Some(p) => std::fs::read_to_string(p).map_err(|e| format!("{p}: {e}")),
        None => {
            let mut s = String::new();
            std::io::stdin()
                .read_to_string(&mut s)
                .map(|_| s)
                .map_err(|e| format!("stdin: {e}"))
        }
    };
    let input = match input {
        Ok(s) => s,
        Err(e) => {
            eprintln!("error: {e}");
            return ExitCode::from(1);
        }
    };

    let output = match flashtex_pdf::render_envelope(&input) {
        Ok(o) => o,
        Err(e) => {
            eprintln!("error: {e}");
            return ExitCode::from(1);
        }
    };
    for w in &output.warnings {
        eprintln!("warning: {w}");
    }
    if verify && let Err(e) = flashtex_pdf::verify::check_structure(&output.bytes) {
        eprintln!("error: generated PDF failed self-check: {e}");
        return ExitCode::from(1);
    }
    if let Err(e) = std::fs::write(&out_path, &output.bytes) {
        eprintln!("error: {out_path}: {e}");
        return ExitCode::from(1);
    }
    ExitCode::SUCCESS
}

fn usage(msg: &str) -> ExitCode {
    eprintln!("error: {msg}\n{USAGE}");
    ExitCode::from(2)
}
