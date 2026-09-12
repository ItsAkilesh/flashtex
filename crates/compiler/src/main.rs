//! JSON Lines worker. Requests on stdin, replies on stdout, logs on stderr.

use std::io::{self, BufRead, Write};

fn main() {
    let stdin = io::stdin();
    let stdout = io::stdout();
    let mut out = stdout.lock();

    for line in stdin.lock().lines() {
        let line = match line {
            Ok(l) => l,
            Err(e) => {
                eprintln!("flashtex-compiler: read error: {}", e);
                break;
            }
        };
        if line.trim().is_empty() {
            continue;
        }
        let reply = flashtex_compiler::protocol::handle_line(&line);
        if writeln!(out, "{}", reply).is_err() {
            break;
        }
        let _ = out.flush();
    }
}
