//! `flashtex-pdf`: read a runtime-v1 `compile_result` envelope, write a PDF.
//!
//! ```text
//! flashtex-pdf --out out.pdf < compile-result.json
//! flashtex-pdf compile-result.json --out out.pdf
//! ```
//!
//! Warnings (substituted characters, skipped item kinds) go to stderr, one per
//! line, prefixed with `warning:`, followed by a `note:` summary line when any
//! were emitted. Exit status is 0 on success even with warnings (the PDF was
//! written and nothing was dropped), 1 when the input is not a supported
//! envelope or cannot be rendered, 2 on usage errors.
//!
//! `--embed-font PATH` embeds a Unicode OpenType font; `--embed-font auto`
//! honours `FLASHTEX_UNICODE_FONT`, then Latin Modern, then macOS system
//! fonts. Setting the environment variable alone also enables embedding.
//! `--default-face embedded|lm|times` chooses whether the embedded font is
//! the document face (implied for Latin Modern) or only fills gaps behind
//! base-14 Times (implied for anything else). Default: no embedding.

use std::io::Read;
use std::process::ExitCode;

const USAGE: &str = "usage: flashtex-pdf [INPUT.json] --out OUTPUT.pdf [--verify] [--embed-font PATH|auto] [--default-face embedded|lm|times]\n\
       Reads a runtime-v1 compile_result envelope (from INPUT.json or stdin) and writes a PDF.\n\
       --embed-font PATH        embed this .ttf (subset) or .otf (whole CFF)\n\
       --embed-font auto        use $FLASHTEX_UNICODE_FONT, else Latin Modern, else a macOS system font\n\
       --default-face embedded  set all text in the embedded font (Symbol, then Times, fill gaps)\n\
       --default-face lm        same as 'embedded'\n\
       --default-face times     base-14 Times first; the embedded font only fills gaps\n\
       Without --default-face, Latin Modern implies 'embedded' and any other font 'times'.\n\
       Setting FLASHTEX_UNICODE_FONT alone also enables embedding.";

fn main() -> ExitCode {
    let mut out_path = None;
    let mut input_path = None;
    let mut verify = false;
    // The environment variable alone opts in; "auto" then resolves to it.
    let mut embed: Option<String> =
        std::env::var_os(flashtex_pdf::embed::ENV_VAR).map(|_| "auto".to_string());
    let mut face: Option<flashtex_pdf::encoding::Face> = None;
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--out" | "-o" => match args.next() {
                Some(p) => out_path = Some(p),
                None => return usage("--out needs a path"),
            },
            "--verify" => verify = true,
            "--embed-font" => match args.next() {
                Some(p) => embed = Some(p),
                None => return usage("--embed-font needs a path or 'auto'"),
            },
            "--default-face" => match args.next().as_deref() {
                Some("embedded" | "lm") => face = Some(flashtex_pdf::encoding::Face::Embedded),
                Some("times") => face = Some(flashtex_pdf::encoding::Face::Times),
                Some(other) => return usage(&format!("unknown --default-face {other:?}")),
                None => return usage("--default-face needs embedded, lm, or times"),
            },
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

    let mut options = flashtex_pdf::RenderOptions::default();
    let mut embed_note = None;
    if let Some(spec) = embed {
        let loaded = if spec == "auto" {
            flashtex_pdf::embed::EmbedFont::discover()
        } else {
            flashtex_pdf::embed::EmbedFont::load(std::path::Path::new(&spec)).map(Some)
        };
        match loaded {
            Ok(Some(font)) => {
                let chosen =
                    face.unwrap_or_else(|| flashtex_pdf::RenderOptions::default_face_for(&font));
                embed_note = Some(format!(
                    "embedding {} from {} as {}",
                    font.font.postscript_name,
                    font.source.display(),
                    match chosen {
                        flashtex_pdf::encoding::Face::Embedded => "the document face",
                        flashtex_pdf::encoding::Face::Times =>
                            "a fallback for characters outside WinAnsi/Symbol (Times remains the face)",
                    }
                ));
                options.embed_font = Some(font);
                options.face = chosen;
            }
            Ok(None) => eprintln!(
                "warning: --embed-font auto found no usable font (set {} or install a .ttf); characters outside WinAnsi/Symbol become '?'",
                flashtex_pdf::embed::ENV_VAR
            ),
            Err(e) => {
                eprintln!("error: {e}");
                return ExitCode::from(1);
            }
        }
    }

    let output = match flashtex_pdf::render_envelope_with(&input, &options) {
        Ok(o) => o,
        Err(e) => {
            eprintln!("error: {e}");
            return ExitCode::from(1);
        }
    };
    if let Some(note) = embed_note {
        eprintln!("note: {note}");
    }
    for w in &output.warnings {
        eprintln!("warning: {w}");
    }
    if !output.warnings.is_empty() {
        eprintln!(
            "note: {} warning(s); the PDF was written but some content was substituted or skipped (see above)",
            output.warnings.len()
        );
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
