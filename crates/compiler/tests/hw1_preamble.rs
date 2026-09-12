//! HW1 preamble coverage: geometry, enumitem labels and package warnings.
use flashtex_compiler::parser::parse;

const HW1: &str = include_str!("../../../fixtures/real-world/hw1/HW1.tex");

fn messages(source: &str) -> Vec<String> {
    parse(source)
        .diagnostics
        .into_iter()
        .map(|d| d.message)
        .collect()
}

#[test]
fn hw1_diagnostic_inventory() {
    let parsed = parse(HW1);
    for d in &parsed.diagnostics {
        println!("{:?} | {}", d.severity, d.message);
    }
    println!("HW1 diagnostics: {}", parsed.diagnostics.len());
}

#[test]
fn hw1_enumerate_option_does_not_leak_as_text() {
    let blocks = format!("{:?}", parse(HW1).blocks);
    assert!(
        !blocks.contains("[(a)]"),
        "enumerate option typeset as text"
    );
    assert!(blocks.contains("\"(a)\"") && blocks.contains("\"(b)\""));
}

#[test]
fn enumerate_label_templates() {
    let doc = |options: &str| {
        format!(
            "\\documentclass{{article}}\\begin{{document}}\\begin{{enumerate}}{options}\\item x\\item y\\item z\\item w\\end{{enumerate}}\\end{{document}}"
        )
    };
    let blocks = |options: &str| format!("{:?}", parse(&doc(options)).blocks);
    assert!(blocks("[(a)]").contains("\"(d)\""));
    assert!(blocks("[i.]").contains("\"iv.\""));
    assert!(blocks("[A)]").contains("\"C)\""));
    assert!(blocks("[label=\\Roman*.]").contains("\"III.\""));
    assert!(blocks("").contains("\"4.\""));
}

#[test]
fn packages_matching_the_fixed_layout_do_not_warn() {
    let preamble = |line: &str| {
        messages(&format!(
            "\\documentclass{{article}}{line}\\begin{{document}}x\\end{{document}}"
        ))
    };
    for line in [
        "\\usepackage[margin=1in]{geometry}",
        "\\usepackage[margin=72.27pt]{geometry}",
        "\\usepackage[utf8]{inputenc}",
        "\\usepackage[T1]{fontenc}",
        "\\usepackage[shortlabels]{enumitem}",
    ] {
        assert!(preamble(line).is_empty(), "{line}: {:?}", preamble(line));
    }
    for line in [
        "\\usepackage[margin=2cm]{geometry}",
        "\\usepackage{geometry}",
        "\\usepackage[a4paper,margin=1in]{geometry}",
        "\\usepackage{microtype}",
    ] {
        assert!(
            preamble(line)
                .iter()
                .any(|m| m.contains("recognised but not implemented")),
            "{line} must still report a gap"
        );
    }
    assert!(preamble("\\setlist[enumerate]{itemsep=1em}")
        .iter()
        .any(|m| m.contains("\\setlist")));
}
