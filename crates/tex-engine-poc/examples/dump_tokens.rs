fn main() {
    let path = std::env::args().nth(1).unwrap();
    let src = std::fs::read_to_string(path).unwrap();
    let r = flashtex_tex_expansion::expand_str(&src);
    for t in &r.tokens {
        println!("{:?}", t.kind);
    }
    for d in &r.diagnostics {
        eprintln!("DIAG {:?}", d);
    }
}
