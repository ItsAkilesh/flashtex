//! `flashtex-render`: a drop-in runtime-v1 worker (set `FLASHTEX_COMPILER`
//! to this binary). Requests on stdin, one `compile_result` line per request
//! on stdout, logs on stderr.
//!
//! Options:
//!   --v2 <out.json>    also write the rendering-v2 display list of the
//!                      last request (an explicit `display_list` envelope)
//!   --pdf <out.pdf>    also write a PDF of the last request through the
//!                      pdf sibling (v1 text items, see src/pdf.rs)
//!   --font-dir <dir>   extra font directory (repeatable; probed before
//!                      the TeX Live defaults; `FLASHTEX_FONT_DIRS` too)
//!   --class-options <opts>  class options assumed for body-only input
//!                      (default `12pt`, the compiler's implicit preamble)
//!   --secnumdepth <n>  section numbering depth when the source does not
//!                      set the counter (default 2; the oracle preamble is 0)
//!   --timing           print per-request wall time to stderr

use std::io::{self, Write};
use std::path::PathBuf;

use flashtex_compiler::json;
use flashtex_compiler::protocol::{error_envelope, read_request_line, RequestLine};
use flashtex_render_pipeline::{protocol, FontSet, RenderOptions};

fn main() {
    let mut args = std::env::args().skip(1);
    let mut v2_out: Option<PathBuf> = None;
    let mut pdf_out: Option<PathBuf> = None;
    let mut dirs: Vec<PathBuf> = Vec::new();
    let mut options = RenderOptions::default();
    let mut timing = false;
    while let Some(a) = args.next() {
        match a.as_str() {
            "--v2" => v2_out = args.next().map(PathBuf::from),
            "--pdf" => pdf_out = args.next().map(PathBuf::from),
            "--font-dir" => {
                if let Some(d) = args.next() {
                    dirs.push(PathBuf::from(d));
                }
            }
            "--class-options" => {
                if let Some(o) = args.next() {
                    options.default_class_options = o;
                }
            }
            "--secnumdepth" => {
                if let Some(n) = args.next().and_then(|n| n.parse::<u8>().ok()) {
                    options.default_secnumdepth = n;
                }
            }
            "--timing" => timing = true,
            "-h" | "--help" => {
                eprintln!("usage: flashtex-render [--v2 out.json] [--pdf out.pdf] [--font-dir DIR]... [--class-options OPTS] [--secnumdepth N] [--timing]");
                return;
            }
            other => {
                eprintln!("flashtex-render: unknown argument {other:?}");
                std::process::exit(2);
            }
        }
    }
    let fonts = FontSet::with_default_dirs(&dirs);
    let stdin = io::stdin();
    let stdout = io::stdout();
    let mut out = stdout.lock();
    let mut input = stdin.lock();
    loop {
        let reply = match read_request_line(&mut input) {
            Ok(Some(RequestLine::Data(bytes))) => match std::str::from_utf8(&bytes) {
                Ok(line) if line.trim().is_empty() => continue,
                Ok(line) => protocol::handle_line(line, &fonts, &options),
                Err(_) => protocol::Reply {
                    line: json::write(&error_envelope("", "invalid_utf8", "request line is not valid UTF-8")),
                    rendered: None,
                    id: String::new(),
                },
            },
            Ok(Some(RequestLine::TooLarge)) => protocol::Reply {
                line: json::write(&error_envelope(
                    "",
                    "payload_too_large",
                    &format!("line exceeds the {}-byte limit", protocol::MAX_LINE_BYTES),
                )),
                rendered: None,
                id: String::new(),
            },
            Ok(None) => break,
            Err(e) => {
                eprintln!("flashtex-render: read error: {e}");
                break;
            }
        };
        if writeln!(out, "{}", reply.line).is_err() {
            break;
        }
        let _ = out.flush();
        if let Some(r) = &reply.rendered {
            if timing {
                eprintln!("flashtex-render: {} rendered in {:.2} ms", reply.id, r.elapsed_ms);
            }
            if let Some(p) = &v2_out {
                let text = json::write(&r.v2.to_json(&reply.id));
                if let Err(e) = std::fs::write(p, text) {
                    eprintln!("flashtex-render: cannot write {}: {e}", p.display());
                }
            }
            if let Some(p) = &pdf_out {
                match flashtex_render_pipeline::pdf::write_pdf(&r.v2) {
                    Ok(pdf) => {
                        for w in &pdf.warnings {
                            eprintln!("flashtex-render: pdf: {w}");
                        }
                        if let Err(e) = std::fs::write(p, &pdf.bytes) {
                            eprintln!("flashtex-render: cannot write {}: {e}", p.display());
                        }
                    }
                    Err(e) => eprintln!("flashtex-render: pdf failed: {e}"),
                }
            }
        }
    }
}
