//! `\setlist` list-spacing coverage: `itemsep`/`topsep` actually move item
//! baselines by the requested amount, unimplemented keys are named once in a
//! single diagnostic, and a document with no `\setlist` at all keeps using
//! plain `Block::Paragraph` items (so its layout is exactly today's).

use flashtex_compiler::json::{self, Value};
use flashtex_compiler::parser::{self, Block};
use flashtex_compiler::protocol::handle_line;

fn compile_line(id: &str, text: &str) -> String {
    let mut doc = Value::obj();
    doc.set("path", json::str_("main.tex"));
    doc.set("text", json::str_(text));
    let mut payload = Value::obj();
    payload.set("project_id", json::str_("setlist-spacing"));
    payload.set("revision", Value::Num(1.0));
    payload.set("entry_path", json::str_("main.tex"));
    payload.set("documents", Value::Arr(vec![doc]));
    let mut env = Value::obj();
    env.set("protocol_version", Value::Num(1.0));
    env.set("id", json::str_(id));
    env.set("type", json::str_("compile"));
    env.set("payload", payload);
    json::write(&env)
}

fn reply(line: &str) -> Value {
    json::parse(&handle_line(line)).expect("reply must be valid JSON")
}

fn items(v: &Value) -> Vec<Value> {
    v.get("payload")
        .and_then(|p| p.get("pages"))
        .and_then(Value::as_arr)
        .map(|pages| {
            pages
                .iter()
                .flat_map(|pg| {
                    pg.get("items")
                        .and_then(Value::as_arr)
                        .cloned()
                        .unwrap_or_default()
                })
                .collect()
        })
        .unwrap_or_default()
}

fn baseline_of(response: &Value, text: &str) -> f64 {
    items(response)
        .iter()
        .find(|item| item.get("text").and_then(Value::as_str) == Some(text))
        .and_then(|item| item.get("baseline_y_pt"))
        .and_then(|v| match v {
            Value::Num(n) => Some(*n),
            _ => None,
        })
        .unwrap_or_else(|| panic!("no positioned item with text {text:?} in {response:?}"))
}

fn messages(response: &Value) -> Vec<String> {
    response
        .get("payload")
        .and_then(|p| p.get("diagnostics"))
        .and_then(Value::as_arr)
        .into_iter()
        .flatten()
        .filter_map(|d| d.get("message").and_then(Value::as_str).map(str::to_string))
        .collect()
}

/// A short paragraph, a 3-item `enumerate`, then another paragraph, with
/// `extra` inserted right after `\documentclass`.
fn doc(extra: &str) -> String {
    format!(
        r"\documentclass{{article}}{extra}\begin{{document}}Intro.\begin{{enumerate}}\item Alpha\item Beta\item Gamma\end{{enumerate}}Outro.\end{{document}}"
    )
}

/// Rounded `baseline_y_pt` values carry up to 0.005pt of error each; a
/// difference of differences combines four of them.
const TOLERANCE_PT: f64 = 0.02;

#[test]
fn itemsep_adds_extra_gap_only_between_items() {
    let baseline = reply(&compile_line("itemsep-base", &doc("")));
    let spaced = reply(&compile_line(
        "itemsep-spaced",
        &doc(r"\setlist[enumerate]{itemsep=10pt}"),
    ));

    let base_intro = baseline_of(&baseline, "Intro.");
    let base_alpha = baseline_of(&baseline, "Alpha");
    let base_beta = baseline_of(&baseline, "Beta");
    let base_gamma = baseline_of(&baseline, "Gamma");
    let base_outro = baseline_of(&baseline, "Outro.");

    let sp_intro = baseline_of(&spaced, "Intro.");
    let sp_alpha = baseline_of(&spaced, "Alpha");
    let sp_beta = baseline_of(&spaced, "Beta");
    let sp_gamma = baseline_of(&spaced, "Gamma");
    let sp_outro = baseline_of(&spaced, "Outro.");

    // Before the first item, and after the list: itemsep does not apply.
    assert!((sp_intro - base_intro).abs() < TOLERANCE_PT);
    assert!(((sp_alpha - sp_intro) - (base_alpha - base_intro)).abs() < TOLERANCE_PT);
    assert!(((sp_outro - sp_gamma) - (base_outro - base_gamma)).abs() < TOLERANCE_PT);

    // Between items: each gap grows by exactly itemsep.
    assert!(
        (((sp_beta - sp_alpha) - (base_beta - base_alpha)) - 10.0).abs() < TOLERANCE_PT,
        "Alpha->Beta gap must grow by itemsep"
    );
    assert!(
        (((sp_gamma - sp_beta) - (base_gamma - base_beta)) - 10.0).abs() < TOLERANCE_PT,
        "Beta->Gamma gap must grow by itemsep"
    );
}

#[test]
fn topsep_adds_extra_gap_only_around_the_list() {
    let baseline = reply(&compile_line("topsep-base", &doc("")));
    let spaced = reply(&compile_line(
        "topsep-spaced",
        &doc(r"\setlist[enumerate]{topsep=8pt}"),
    ));

    let base_intro = baseline_of(&baseline, "Intro.");
    let base_alpha = baseline_of(&baseline, "Alpha");
    let base_beta = baseline_of(&baseline, "Beta");
    let base_gamma = baseline_of(&baseline, "Gamma");
    let base_outro = baseline_of(&baseline, "Outro.");

    let sp_intro = baseline_of(&spaced, "Intro.");
    let sp_alpha = baseline_of(&spaced, "Alpha");
    let sp_beta = baseline_of(&spaced, "Beta");
    let sp_gamma = baseline_of(&spaced, "Gamma");
    let sp_outro = baseline_of(&spaced, "Outro.");

    // Between items: topsep does not apply there (that's itemsep's job).
    assert!(((sp_beta - sp_alpha) - (base_beta - base_alpha)).abs() < TOLERANCE_PT);
    assert!(((sp_gamma - sp_beta) - (base_gamma - base_beta)).abs() < TOLERANCE_PT);

    // Before the first item, and after the last: each grows by topsep.
    assert!(
        (((sp_alpha - sp_intro) - (base_alpha - base_intro)) - 8.0).abs() < TOLERANCE_PT,
        "gap before the first item must grow by topsep"
    );
    assert!(
        (((sp_outro - sp_gamma) - (base_outro - base_gamma)) - 8.0).abs() < TOLERANCE_PT,
        "gap after the last item must grow by topsep"
    );
}

#[test]
fn fully_implemented_keys_report_no_diagnostic() {
    let response = reply(&compile_line(
        "silent",
        &doc(r"\setlist[enumerate]{itemsep=0.45em,topsep=0.35em}"),
    ));
    let msgs = messages(&response);
    assert!(msgs.is_empty(), "{msgs:?}");
}

#[test]
fn unimplemented_keys_are_named_once_and_implemented_ones_are_not() {
    let response = reply(&compile_line(
        "named",
        &doc(r"\setlist[enumerate]{leftmargin=*,itemsep=1pt,parsep=2pt}"),
    ));
    let msgs = messages(&response);
    let setlist_msgs: Vec<&String> = msgs.iter().filter(|m| m.contains("\\setlist")).collect();
    assert_eq!(setlist_msgs.len(), 1, "{msgs:?}");
    assert!(setlist_msgs[0].contains("leftmargin"));
    assert!(setlist_msgs[0].contains("parsep"));
    assert!(!setlist_msgs[0].contains("itemsep"));
}

#[test]
fn no_setlist_keeps_items_as_plain_paragraphs() {
    // The exact HW1 shape, minus \setlist: default output must be
    // byte-identical to before this feature existed.
    let parsed = parser::parse(&doc(""));
    assert!(
        parsed
            .blocks
            .iter()
            .all(|block| !matches!(block, Block::ListItem { .. })),
        "no \\setlist means every item stays a plain Block::Paragraph: {:?}",
        parsed.blocks
    );
}

#[test]
fn setlist_spacing_is_attached_to_the_right_items() {
    let parsed = parser::parse(&doc(r"\setlist[enumerate]{itemsep=5pt,topsep=3pt}"));
    let gaps: Vec<(f64, f64)> = parsed
        .blocks
        .iter()
        .filter_map(|block| match block {
            Block::ListItem {
                extra_gap_before_pt,
                extra_gap_after_pt,
                ..
            } => Some((*extra_gap_before_pt, *extra_gap_after_pt)),
            _ => None,
        })
        .collect();
    // Alpha: topsep before, nothing after. Beta: itemsep before, nothing
    // after. Gamma: itemsep before, topsep after (it is the last item).
    assert_eq!(gaps, vec![(3.0, 0.0), (5.0, 0.0), (5.0, 3.0)]);
}

#[test]
fn setlist_without_an_environment_argument_applies_to_both_list_types() {
    let source = r"\documentclass{article}\setlist{itemsep=6pt}\begin{document}\begin{itemize}\item One\item Two\end{itemize}\end{document}";
    let baseline = reply(&compile_line(
        "both-base",
        r"\documentclass{article}\begin{document}\begin{itemize}\item One\item Two\end{itemize}\end{document}",
    ));
    let spaced = reply(&compile_line("both-spaced", source));

    let base_one = baseline_of(&baseline, "One");
    let base_two = baseline_of(&baseline, "Two");
    let sp_one = baseline_of(&spaced, "One");
    let sp_two = baseline_of(&spaced, "Two");

    assert!(
        (((sp_two - sp_one) - (base_two - base_one)) - 6.0).abs() < TOLERANCE_PT,
        "an environment-less \\setlist must still cover itemize"
    );
}
