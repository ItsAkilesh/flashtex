//! Figure/table floats, numbered captions and `\includegraphics` boxes
//! (issue #69). Positions follow article.cls float parameters in this
//! layout's fixed-glue model: 12pt body, 72pt margins, 648pt text height.

use flashtex_compiler::diagnostics::Severity;
use flashtex_compiler::incremental::{compile_full, CompileOutput, LayoutConstraints, Session};
use flashtex_compiler::layout::TextItem;

fn compile(source: &str) -> CompileOutput {
    compile_full(source, LayoutConstraints::default())
}

fn find<'a>(output: &'a CompileOutput, text: &str) -> (usize, &'a TextItem) {
    output
        .pages
        .iter()
        .enumerate()
        .find_map(|(index, page)| {
            page.items
                .iter()
                .find(|item| item.text == text)
                .map(|item| (index, item))
        })
        .unwrap_or_else(|| panic!("no item {text:?}"))
}

fn rules(output: &CompileOutput, page: usize) -> Vec<&TextItem> {
    output.pages[page]
        .items
        .iter()
        .filter(|item| item.rule.is_some())
        .collect()
}

fn filler(words: usize) -> String {
    "word ".repeat(words)
}

#[test]
fn here_float_sits_between_intextsep_glue_with_caption_below() {
    let source = "First line.\n\n\\begin{figure}[h]\\centering\\includegraphics[width=0.5\\textwidth,height=2in]{plot.pdf}\\caption{Cache}\\label{f}\\end{figure}\n\nAfter \\ref{f} on \\pageref{f}.";
    let output = compile(source);
    assert_eq!(output.pages.len(), 1);
    let frame = rules(&output, 0);
    assert_eq!(frame.len(), 4, "a draft frame is four rules");
    let top = frame[0];
    let rule = top.rule.unwrap();
    // Previous baseline 84 + depth 2.4 + \intextsep 14 (12pt class).
    assert_eq!(rule.y_pt, 100.4);
    assert_eq!(rule.width_pt, 234.0, "0.5\\textwidth of a 468pt measure");
    // Centred by \centering.
    assert_eq!(top.x_pt, 189.0);
    assert_eq!(
        &source[top.span.start..top.span.end],
        "\\includegraphics[width=0.5\\textwidth,height=2in]{plot.pdf}"
    );
    // 2in = 144.54pt; caption baseline = box bottom + \abovecaptionskip 10 + 1.2 x 12.
    let (_, caption) = find(&output, "Figure 1:");
    assert_eq!(caption.baseline_y_pt, 269.34);
    assert_eq!(&source[caption.span.start..caption.span.end], "\\caption");
    // Float bottom 271.74 + \intextsep 14, then the ordinary paragraph step.
    let (_, after) = find(&output, "After");
    assert_eq!(after.baseline_y_pt, 306.14);
    let texts: Vec<_> = output.pages[0]
        .items
        .iter()
        .map(|i| i.text.as_str())
        .collect();
    assert!(texts.windows(2).any(|w| w == ["After", "1"]), "{texts:?}");
    assert!(texts.windows(2).any(|w| w == ["on", "1"]), "{texts:?}");
}

#[test]
fn one_line_captions_centre_and_long_captions_are_paragraphs() {
    let long = "A caption long enough that it cannot possibly fit on one line of the measure and therefore wraps.";
    let source = format!(
        "\\begin{{figure}}[h]\\caption{{Short}}\\end{{figure}}\\begin{{figure}}[h]\\caption{{{long}}}\\end{{figure}}"
    );
    let output = compile(&source);
    let (_, short) = find(&output, "Figure 1:");
    let (_, short_word) = find(&output, "Short");
    let right_gap = 540.0 - (short_word.x_pt + 27.0);
    assert!(short.x_pt > 250.0, "centred: {}", short.x_pt);
    assert!((short.x_pt - 72.0 - right_gap).abs() < 3.0);
    let (_, wrapped) = find(&output, "Figure 2:");
    assert_eq!(wrapped.x_pt, 72.0);
}

#[test]
fn figure_and_table_counters_are_independent_and_short_caption_is_not_typeset() {
    let output = compile(
        "\\begin{table}[ht]\\caption[Short form]{Tab}\\end{table}\\begin{figure}[ht]\\caption{Fig}\\end{figure}\\begin{table}[ht]\\caption{Tab two}\\end{table}",
    );
    let texts: Vec<_> = output.pages[0]
        .items
        .iter()
        .map(|i| i.text.as_str())
        .collect();
    for expected in ["Table 1:", "Figure 1:", "Table 2:"] {
        assert!(texts.contains(&expected), "{texts:?}");
    }
    assert!(!texts.contains(&"form"), "{texts:?}");
    assert!(output.diagnostics.is_empty(), "{:?}", output.diagnostics);
}

#[test]
fn default_float_after_text_goes_to_the_top_of_the_current_page() {
    let source = "Opening text.\n\n\\begin{figure}\\includegraphics[width=1in,height=1in]{a}\\end{figure}\n\nClosing text.";
    let output = compile(source);
    assert_eq!(output.pages.len(), 1);
    let top = rules(&output, 0)[0].rule.unwrap();
    assert_eq!(top.y_pt, 72.0);
    // The text moves down by the float (72.27pt) plus \textfloatsep (20pt).
    let (_, opening) = find(&output, "Opening");
    assert_eq!(opening.baseline_y_pt, 176.27);
    // Float items come first in reading order.
    assert!(output.pages[0].items[0].rule.is_some());
}

#[test]
fn bottom_float_sits_on_the_bottom_margin() {
    let output = compile(
        "Text.\n\n\\begin{figure}[b]\\includegraphics[width=1in,height=1in]{a}\\end{figure}",
    );
    let frame = rules(&output, 0);
    let bottom_rule = frame[1].rule.unwrap();
    assert_eq!(bottom_rule.y_pt + bottom_rule.height_pt, 720.0);
    let (_, text) = find(&output, "Text.");
    assert_eq!(text.baseline_y_pt, 84.0);
}

#[test]
fn floats_that_do_not_fit_defer_in_order_and_references_follow_their_page() {
    let big = |n: u32| {
        format!("\\begin{{figure}}\\includegraphics[height=4in]{{big{n}}}\\caption{{Big {n}}}\\label{{b{n}}}\\end{{figure}}\n\n")
    };
    let source = format!(
        "{}\n\n{}{}{}{}\n\nSee \\pageref{{b1}} \\pageref{{b2}} \\pageref{{b3}}.",
        filler(40),
        big(1),
        big(2),
        big(3),
        filler(900)
    );
    let output = compile(&source);
    let (page1, _) = find(&output, "Figure 1:");
    let (page2, _) = find(&output, "Figure 2:");
    let (page3, _) = find(&output, "Figure 3:");
    assert_eq!(page1, 0, "first float fits the top of page 1");
    assert_eq!(page2, 1, "the second is deferred to the next page");
    assert_eq!(page3, 1, "the third follows it, never overtaking");
    // Two 315.88pt floats make a float page: no text on page 2.
    assert!(output.pages[1]
        .items
        .iter()
        .all(|item| item.rule.is_some() || !item.text.starts_with("word")));
    let see = output
        .pages
        .iter()
        .flat_map(|page| &page.items)
        .skip_while(|item| item.text != "See")
        .skip(1)
        .take(3)
        .map(|item| item.text.as_str())
        .collect::<Vec<_>>();
    assert_eq!(see, ["1", "2", "2"]);
}

#[test]
fn page_only_float_is_flushed_at_the_end_on_a_vertically_centred_float_page() {
    let output = compile(
        "Text.\n\n\\begin{figure}[p]\\includegraphics[width=1in,height=1in]{a}\\end{figure}\n\nMore.",
    );
    assert_eq!(output.pages.len(), 2);
    let top = rules(&output, 1)[0].rule.unwrap();
    // (648 - 72.27) / 2 above and below.
    assert_eq!(top.y_pt, 72.0 + 287.87);
    assert!(output.pages[0].items.iter().all(|item| item.rule.is_none()));
}

#[test]
fn placement_h_requires_float_package() {
    let without = compile("\\begin{figure}[H]\\caption{X}\\end{figure}");
    assert!(without.diagnostics.iter().any(|d| {
        d.severity == Severity::Error && d.message.contains("`H'") && d.message.contains("float")
    }));
    let with = compile(
        "\\documentclass{article}\\usepackage{float}\\begin{document}Before.\n\n\\begin{figure}[H]\\includegraphics[width=1in,height=1in]{a}\\end{figure}\n\nAfter.\\end{document}",
    );
    assert!(with
        .diagnostics
        .iter()
        .all(|d| d.severity != Severity::Error));
    let top = rules(&with, 0)[0].rule.unwrap();
    assert_eq!(top.y_pt, 100.4, "set in the text, not moved to the top");
}

#[test]
fn lone_h_becomes_ht_with_a_warning() {
    let output = compile("\\begin{figure}[h]\\caption{X}\\end{figure}");
    assert!(output
        .diagnostics
        .iter()
        .any(|d| d.severity == Severity::Warning && d.message.contains("changed to `ht'")));
}

#[test]
fn graphic_sizes_resolve_units_and_report_ignored_options() {
    let output = compile(
        "\\begin{quote}\\includegraphics[width=\\linewidth, height=3cm, angle=90]{a.png}\\end{quote} \\includegraphics[scale=0.5]{b.png} \\includegraphics[height=1in]{c.png}",
    );
    let widths: Vec<(f64, f64)> = output.pages[0]
        .items
        .iter()
        .filter_map(|item| item.rule)
        .filter(|rule| rule.height_pt == 0.4)
        .map(|rule| (rule.width_pt, rule.y_pt))
        .collect();
    // Quote narrows \linewidth by 25pt each side.
    assert_eq!(widths[0].0, 418.0);
    // No width or height: the unknown natural size (144pt) scaled.
    assert_eq!(widths[2].0, 72.0);
    // Height only: the missing side mirrors it.
    assert_eq!(widths[4].0, 72.27);
    let graphics: Vec<_> = output
        .diagnostics
        .iter()
        .filter(|d| d.message.contains("image bytes are not available"))
        .collect();
    assert_eq!(graphics.len(), 3, "exactly one diagnostic per graphic");
    assert!(graphics[0].message.contains("angle=90"));
    let file = output.pages[0]
        .items
        .iter()
        .find(|item| item.text == "a.png")
        .expect("file name shown in the frame");
    assert_eq!(file.font, flashtex_compiler::layout::Font::Courier);
}

#[test]
fn nested_float_is_diagnosed_and_its_content_kept() {
    let output = compile(
        "\\begin{figure}[h]\\begin{table}Inner\\caption{T}\\end{table}\\caption{F}\\end{figure}",
    );
    assert!(output
        .diagnostics
        .iter()
        .any(|d| d.message.contains("inside another float")));
    let texts: Vec<_> = output.pages[0]
        .items
        .iter()
        .map(|i| i.text.as_str())
        .collect();
    for expected in ["Inner", "Figure 1:", "Figure 2:"] {
        assert!(texts.contains(&expected), "{texts:?}");
    }
}

#[test]
fn caption_outside_a_float_stays_an_error() {
    let output = compile("\\caption{Loose}");
    assert!(output
        .diagnostics
        .iter()
        .any(|d| d.severity == Severity::Error && d.message.contains("outside float")));
}

#[test]
fn incremental_edits_around_floats_match_a_clean_build() {
    let constraints = LayoutConstraints::default();
    let base = format!(
        "{}\n\n\\begin{{figure}}\\includegraphics[height=4in]{{a}}\\caption{{A}}\\end{{figure}}\n\n{}\n\n\\begin{{figure}}\\includegraphics[height=4in]{{b}}\\caption{{B}}\\end{{figure}}\n\n{}",
        filler(30),
        filler(200),
        filler(300)
    );
    let edited = base.replacen("word", "longer words here", 1);
    let mut session = Session::new();
    session.compile(&base, constraints);
    let incremental = session.compile(&edited, constraints);
    let clean = compile_full(&edited, constraints);
    assert_eq!(format!("{:?}", incremental.output), format!("{clean:?}"));
    assert!(incremental.stats.full_recompile);
}

#[test]
fn float_body_ignores_the_enclosing_list_and_the_list_resumes_after_it() {
    let output = compile(
        "\\begin{itemize}\\item Before\\begin{figure}[ht]Body\\caption{C}\\end{figure}\\item After\\end{itemize}",
    );
    let (_, body) = find(&output, "Body");
    assert_eq!(body.x_pt, 72.0, "full column width inside the float");
    assert_eq!(
        body.span.end - body.span.start,
        4,
        "[ht]Body keeps an exact span"
    );
    let (_, before) = find(&output, "Before");
    let (_, after) = find(&output, "After");
    assert_eq!(before.x_pt, after.x_pt);
    assert!(after.x_pt > 72.0);
    assert_eq!(
        output.pages[0]
            .items
            .iter()
            .filter(|item| item.text == "•")
            .count(),
        2
    );
}
