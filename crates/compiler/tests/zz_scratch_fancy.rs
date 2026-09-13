use flashtex_compiler::{layout, parser};
#[test]
fn dump() {
    let src = " #1\\documentclass{} \\emph{}\\documentclass\\textbf{\\documentclass{}~\\alpha\\begin{document}\\begin{align}日本語\\begin{document}";
    let p = parser::parse(src);
    for b in &p.blocks { println!("BLOCK {:?}", b); }
    for d in &p.diagnostics { println!("DIAG {:?} {} {:?}", d.severity, d.message, d.span); }
    for page in layout::layout(&p.blocks) {
        for item in page.items { println!("ITEM {:?} {:?}", item.span, item.text); }
    }
}
