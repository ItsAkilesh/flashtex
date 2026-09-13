//! LaTeX kernel `\DeclareMathSymbol`s: the glyph, the font binding and the
//! spacing class.
//!
//! A sweep of every `\DeclareMathSymbol`/`\DeclareMathDelimiter`/`\mathchardef`
//! in `fontmath.ltx`, `plain.tex`, `latexsym.sty` and `amssymb.sty` through this
//! compiler found amssymb essentially complete and the *kernel* set largely
//! missing. These are the rows that needed nothing but a `COMMAND_GLYPHS` entry,
//! a `symbol_class` arm and a decided export outcome.
//!
//! The class each command carries is `fontmath.ltx`'s own (`\mathbin`,
//! `\mathrel`, `\mathop`, `\mathord`), and is checked here the way TeX makes it
//! observable: `$a\sym b$` minus `$a\mathord{\sym}b$` is exactly the inter-atom
//! glue the class adds, with the glyph's own width cancelling out. Each
//! measurement is read against a symbol of the same class this compiler already
//! carried (`+` for Bin, `=` for Rel, `\sum` for Op), so the test pins the class
//! rather than a font size.

use flashtex_compiler::json::{self, Value};
use flashtex_compiler::protocol::handle_line;
use flashtex_compiler::{export, lm_math};

fn compile(text: &str) -> Value {
    let mut doc = Value::obj();
    doc.set("path", json::str_("main.tex"));
    doc.set("text", json::str_(text));
    let mut payload = Value::obj();
    payload.set("project_id", json::str_("kernel-symbols"));
    payload.set("revision", Value::Num(1.0));
    payload.set("entry_path", json::str_("main.tex"));
    payload.set("documents", Value::Arr(vec![doc]));
    payload.set(
        "layout_capabilities",
        Value::Arr(vec![json::str_("font-hints-v1")]),
    );
    let mut env = Value::obj();
    env.set("protocol_version", Value::Num(1.0));
    env.set("id", json::str_("k"));
    env.set("type", json::str_("compile"));
    env.set("payload", payload);
    json::parse(&handle_line(&json::write(&env))).expect("valid JSON reply")
}

fn at<'a>(value: &'a Value, path: &[&str]) -> &'a Value {
    path.iter().fold(value, |v, key| {
        v.get(key).unwrap_or_else(|| panic!("missing {key}"))
    })
}

fn page_items(reply: &Value) -> Vec<&Value> {
    at(reply, &["payload", "pages"])
        .as_arr()
        .unwrap()
        .iter()
        .flat_map(|page| at(page, &["items"]).as_arr().unwrap().iter())
        .collect()
}

fn messages(reply: &Value) -> Vec<String> {
    at(reply, &["payload", "diagnostics"])
        .as_arr()
        .unwrap()
        .iter()
        .map(|d| at(d, &["message"]).as_str().unwrap().to_string())
        .collect()
}

/// The x of the last item whose text is `text`.
fn x_of(reply: &Value, text: &str) -> f64 {
    page_items(reply)
        .iter()
        .rev()
        .find(|item| item.get("text").and_then(|v| v.as_str()) == Some(text))
        .and_then(|item| item.get("x_pt"))
        .and_then(|v| match v {
            Value::Num(n) => Some(*n),
            _ => None,
        })
        .unwrap_or_else(|| panic!("no item {text:?} with an x"))
}

/// How much glue `\sym`'s class puts around it: `$a\sym b$` minus the same
/// formula with the symbol forced to Ord, so the glyph's width cancels.
fn class_glue(command: &str) -> f64 {
    let with = compile(&format!("$a\\{command} b$\n"));
    let ord = compile(&format!("$a\\mathord{{\\{command}}}b$\n"));
    for reply in [&with, &ord] {
        let messages = messages(reply);
        assert!(
            !messages.iter().any(|m| m.contains("not supported")
                || m.contains("Unknown")
                || m.contains("unknown")
                || m.contains("has no glyph")),
            "\\{command}: {messages:#?}"
        );
    }
    x_of(&with, "b") - x_of(&ord, "b")
}

#[derive(Clone, Copy, PartialEq, Debug)]
enum Class {
    Ord,
    Bin,
    Rel,
    Op,
}

/// `(command, glyph, class)` -- class from `fontmath.ltx`.
const KERNEL_SYMBOLS: &[(&str, &str, Class)] = &[
    ("amalg", "⨿", Class::Bin),
    ("asymp", "≍", Class::Rel),
    ("clubsuit", "♣", Class::Ord),
    ("dagger", "†", Class::Bin),
    ("ddagger", "‡", Class::Bin),
    ("diamondsuit", "♢", Class::Ord),
    ("heartsuit", "♡", Class::Ord),
    ("spadesuit", "♠", Class::Ord),
    ("flat", "♭", Class::Ord),
    ("natural", "♮", Class::Ord),
    ("sharp", "♯", Class::Ord),
    ("frown", "⌢", Class::Rel),
    ("smile", "⌣", Class::Rel),
    ("imath", "ı", Class::Ord),
    ("jmath", "ȷ", Class::Ord),
    ("leftharpoonup", "↼", Class::Rel),
    ("leftharpoondown", "↽", Class::Rel),
    ("rightharpoonup", "⇀", Class::Rel),
    ("rightharpoondown", "⇁", Class::Rel),
    ("nearrow", "↗", Class::Rel),
    ("nwarrow", "↖", Class::Rel),
    ("searrow", "↘", Class::Rel),
    ("swarrow", "↙", Class::Rel),
    ("odot", "⊙", Class::Bin),
    ("ominus", "⊖", Class::Bin),
    ("oslash", "⊘", Class::Bin),
    ("prec", "≺", Class::Rel),
    ("preceq", "⪯", Class::Rel),
    ("succ", "≻", Class::Rel),
    ("succeq", "⪰", Class::Rel),
    ("sqcap", "⊓", Class::Bin),
    ("sqcup", "⊔", Class::Bin),
    ("sqsubseteq", "⊑", Class::Rel),
    ("sqsupseteq", "⊒", Class::Rel),
    ("star", "⋆", Class::Bin),
    ("triangleleft", "◁", Class::Bin),
    ("triangleright", "▷", Class::Bin),
    ("uplus", "⊎", Class::Bin),
    ("wr", "≀", Class::Bin),
    ("bullet", "∙", Class::Bin),
    ("diamond", "⋄", Class::Bin),
    ("bigcirc", "◯", Class::Bin),
    ("bigsqcup", "⨆", Class::Op),
    ("biguplus", "⨄", Class::Op),
    ("varrho", "ϱ", Class::Ord),
    ("surd", "√", Class::Ord),
    ("mathdollar", "$", Class::Ord),
    ("mathparagraph", "¶", Class::Ord),
    ("mathsection", "§", Class::Ord),
    ("owns", "∋", Class::Rel),
];

#[test]
fn every_kernel_symbol_compiles_and_emits_its_glyph() {
    for (command, glyph, _) in KERNEL_SYMBOLS {
        let reply = compile(&format!("$x \\{command} y$\n"));
        let messages = messages(&reply);
        assert!(
            !messages.iter().any(|m| m.contains("not supported")
                || m.contains("unknown")
                || m.contains("has no glyph")),
            "\\{command}: {messages:#?}"
        );
        assert!(
            page_items(&reply)
                .iter()
                .any(|item| item.get("text").and_then(|v| v.as_str()) == Some(*glyph)),
            "\\{command} did not emit {glyph:?}"
        );
    }
}

#[test]
fn spacing_classes_match_fontmath_ltx() {
    let bin = class_glue("ast");
    let rel = class_glue("equiv");
    let op = class_glue("sum");
    assert!(bin > 0.0 && rel > bin && op > 0.0 && op < bin, "{bin} {rel} {op}");
    for (command, _, class) in KERNEL_SYMBOLS {
        let expected = match class {
            Class::Ord => 0.0,
            Class::Bin => bin,
            Class::Rel => rel,
            Class::Op => op,
        };
        let actual = class_glue(command);
        assert!(
            // Item x is reported rounded to 0.01pt, and `class_glue` is a
            // difference of two such x, so four roundings can accumulate.
            (actual - expected).abs() < 0.021,
            "\\{command} is {class:?}: expected {expected} of glue, got {actual}"
        );
    }
}

/// Export outcome, and the reason five of them are deliberately not bound to
/// the Latin Modern Math resource: `map_char` consults `lm_math::advance`
/// before WinAnsi, so a row there would take over the *text* face's own dagger
/// and section sign in ordinary prose.
#[test]
fn export_outcomes_are_decided_and_text_glyphs_stay_on_the_text_face() {
    for (command, glyph, _) in KERNEL_SYMBOLS {
        for c in glyph.chars() {
            assert!(
                export::reason(c).is_none(),
                "\\{command} renders {c:?}, which the base-14 writer cannot draw"
            );
        }
    }
    for c in ['\u{2020}', '\u{2021}', '\u{00A7}', '\u{00B6}', '$'] {
        assert_eq!(lm_math::advance(c), None, "{c:?} must stay on the text face");
        assert_eq!(
            export::map_char(c),
            export::Glyph::Encodable {
                font: export::ExportFont::Text,
                code: match c {
                    '\u{2020}' => 0x86,
                    '\u{2021}' => 0x87,
                    '\u{00A7}' => 0xA7,
                    '\u{00B6}' => 0xB6,
                    _ => 0x24,
                },
            },
            "{c:?}"
        );
    }
    // `\surd` is Adobe Symbol's own `radical`, not a Latin Modern Math glyph.
    assert_eq!(lm_math::advance('\u{221A}'), None);
    assert_eq!(
        export::map_char('\u{221A}'),
        export::Glyph::Encodable {
            font: export::ExportFont::Symbol,
            code: 0xD6
        }
    );
}

/// `\bigsqcup`/`\biguplus` are `\mathop`s with `\displaylimits`, like `\sum`:
/// a subscript in display style is set below the operator, not beside it.
#[test]
fn the_two_large_operators_take_display_limits() {
    for command in ["bigsqcup", "biguplus"] {
        let inline = compile(&format!("$\\{command}_{{i}} x$\n"));
        let display = compile(&format!("\\[\\{command}_{{i}} x\\]\n"));
        let op = match command {
            "bigsqcup" => "⨆",
            _ => "⨄",
        };
        let beside = x_of(&inline, "i") - x_of(&inline, op);
        let below = x_of(&display, "i") - x_of(&display, op);
        assert!(
            beside > 0.0,
            "\\{command} inline: the subscript should follow the operator ({beside})"
        );
        assert!(
            below < beside,
            "\\{command} display: the subscript should be centred under the operator \
             ({below} vs {beside} inline)"
        );
    }
}
