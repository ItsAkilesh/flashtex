//! Print the width of a string in points.
//!
//! ```sh
//! cargo run --quiet --example measure -- Times-Roman 12 "Hello"
//! ```
//!
//! If the text argument is `-`, the text is read from stdin instead (one
//! trailing newline is stripped). `tools/verify-against-coretext.swift` uses
//! that form because Foundation's `Process` rewrites command-line arguments
//! into decomposed (NFD) Unicode on macOS, which would turn `é` into `e` plus
//! a combining mark.
//!
//! Output: `<width_pt> <unknown_chars>` on one line, width to six decimals.

use flashtex_fontmetrics::{measure, Face};

fn face_from_name(name: &str) -> Option<Face> {
    Face::ALL
        .into_iter()
        .find(|f| f.postscript_name().eq_ignore_ascii_case(name))
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let (face, size, text) = match args.as_slice() {
        [face, size, text] => (face, size, text),
        _ => {
            eprintln!("usage: measure <PostScript face name> <size_pt> <text>");
            std::process::exit(2);
        }
    };
    let Some(face) = face_from_name(face) else {
        eprintln!("unknown face {face:?}; expected one of: {}", names());
        std::process::exit(2);
    };
    let Ok(size) = size.parse::<f64>() else {
        eprintln!("size must be a number, got {size:?}");
        std::process::exit(2);
    };
    let stdin_text;
    let text: &str = if text == "-" {
        let mut buf = String::new();
        if let Err(e) = std::io::Read::read_to_string(&mut std::io::stdin(), &mut buf) {
            eprintln!("could not read stdin: {e}");
            std::process::exit(2);
        }
        stdin_text = buf.strip_suffix('\n').map(str::to_owned).unwrap_or(buf);
        &stdin_text
    } else {
        text
    };
    let m = measure(face, text, size);
    println!("{:.6} {}", m.width_pt, m.unknown_chars);
}

fn names() -> String {
    Face::ALL
        .iter()
        .map(|f| f.postscript_name())
        .collect::<Vec<_>>()
        .join(", ")
}
