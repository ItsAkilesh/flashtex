//! Adversarial bounds: hostile math lists and hostile metrics providers must
//! each produce a typed [`Limitation`] and/or a bounded, finite [`MathBox`],
//! never a panic, a hang, or a stack overflow.
//!
//! Covers, at minimum (one `#[test]` each, in this order): nesting far past
//! any sane depth; an enormous flat list; a radical whose degree is itself
//! deeply nested; delimiters with an empty size chain; scripts attached to
//! an empty nucleus; zero and negative metric values from the metrics
//! provider; and a metrics provider that returns `None`/empty for
//! everything it can. FT-032 revision 4, part 1/2 (adversarial bounds).
//!
//! ## The rev-3 recursive-Drop hazard
//!
//! `MathList`/`Nucleus::List` and the `MathBox`/`Child` tree `layout`
//! produces are recursive data structures with no `Box<...>`/`Rc<...>`
//! indirection breaking the recursion (a `Child` owns its `MathBox` content
//! directly, indirected only through the parent's `Vec`). Nothing in this
//! crate implements `Drop` by hand — the hazard is the *compiler-generated*
//! drop glue, which recurses one native stack frame per nesting level with
//! no depth limit, exactly like the engine's own `list`/`atom`/`clean_box`
//! mutual recursion that builds the tree in the first place.
//!
//! Verified empirically before writing the tests below: a `MathList` built
//! by wrapping a single symbol in `Atom::group` 100_000 times constructs
//! fine, but *dropping* it overflows the stack of a plain `fn main` thread
//! in a debug build (`thread 'main' has overflowed its stack` /
//! `SIGABRT`) — construction alone does not trip it. Worse, `cargo test`
//! runs each test on a thread with Rust's default 2 MiB stack (much smaller
//! than a `fn main` thread's own, platform-provided stack), so the same
//! hazard trips at only ~30_000 levels there, and because it is a stack
//! overflow (not a caught panic) it aborts the *entire test binary*
//! (SIGABRT), taking every other test in the process down with it — not
//! just failing the one test that touched the deep fixture.
//!
//! **This is handled here, in the test, not in `src`:** every test that
//! needs a nesting depth deep enough to actually exercise this hazard runs
//! its *entire* body — construction, layout, assertions, and the final drop
//! when locals go out of scope — inside a thread this file spawns with an
//! explicit, generous stack (`std::thread::Builder::stack_size`), and joins
//! it so a panic on that thread still fails the test normally instead of
//! aborting the process. This was verified to survive nesting 200_000 deep
//! with a 512 MiB stack before being written into the tests below (which
//! use a smaller depth, 50_000, well past any sane document's nesting, with
//! the same 512 MiB budget). No source file was changed to make this work;
//! the engine's own recursion depth is exactly what the enlarged stack
//! exists to accommodate.

use flashtex_math_layout::{
    Atom, AtomClass, BoxKind, CmMathMetrics, FontId, Glyph, Limitation, MathBox, MathFontMetrics,
    MathList, MathParams, Nucleus, SizeClass, Style, layout, layout_with_report, positioned_runs,
};

/// Every hostile-string case below needs the same "did this panic, and is
/// the result bounded" check; only the input and the expected missing-glyph
/// count vary.
fn assert_bounded_layout(list: &MathList, m: &CmMathMetrics, expected_limitations: usize) {
    let report = layout_with_report(list, Style::TEXT, m);
    assert_finite_box(&report.root);
    assert_eq!(
        report.limitations.len(),
        expected_limitations,
        "limitations: {:?}",
        report.limitations
    );
    for lim in &report.limitations {
        assert!(
            matches!(
                lim,
                Limitation::MissingGlyph(_) | Limitation::MissingAccent(_)
            ),
            "unexpected limitation kind: {lim:?}"
        );
    }
    let runs = positioned_runs(&report.root, (0.0, 0.0));
    for g in &runs.glyphs {
        assert!(g.x.is_finite() && g.baseline_y.is_finite() && g.size.is_finite());
    }
}

/// Wraps `leaf` in `depth` levels of `{...}` (`Nucleus::List`/`Atom::group`),
/// built with a plain loop (no recursion here — only the eventual layout
/// and drop of the result recurse).
fn nest_groups(depth: usize, leaf: MathList) -> MathList {
    let mut list = leaf;
    for _ in 0..depth {
        list = Atom::group(list).into();
    }
    list
}

/// Runs `f` on a freshly spawned thread with a stack generous enough to
/// build, lay out, and (see the module doc comment) *drop* a fixture nested
/// tens of thousands of levels deep, then joins it so a panic there still
/// fails this test normally rather than aborting the process.
fn run_with_room_for_deep_recursion<F: FnOnce() + Send + 'static>(f: F) {
    std::thread::Builder::new()
        .stack_size(512 * 1024 * 1024)
        .spawn(f)
        .expect("spawn a thread for the deep-recursion probe")
        .join()
        .expect("deep nesting must not panic (and, per the module doc comment, must not crash the process either)");
}

const SANE_DEPTH_MULTIPLE: usize = 50_000;

#[test]
fn nesting_far_past_any_sane_depth_is_bounded_not_a_stack_overflow() {
    run_with_room_for_deep_recursion(|| {
        let m = CmMathMetrics::latex_10pt();
        let list = nest_groups(SANE_DEPTH_MULTIPLE, MathList::symbols("x"));
        let report = layout_with_report(&list, Style::TEXT, &m);
        assert!(report.root.width.is_finite());
        assert!(report.root.height.is_finite());
        assert!(report.root.depth.is_finite());
        assert!(report.limitations.is_empty());
        let runs = positioned_runs(&report.root, (0.0, 0.0));
        assert_eq!(runs.glyphs.len(), 1);
        // `list`, `report`, and `runs` all drop here, 50_000 `HBox` levels
        // deep; the surrounding thread's stack is what keeps that from
        // aborting the process.
    });
}

#[test]
fn radical_degree_nested_far_past_any_sane_depth_is_bounded() {
    run_with_room_for_deep_recursion(|| {
        let m = CmMathMetrics::latex_10pt();
        // `\sqrt[{{{...n...}}}]{x}`: the degree, not the radicand, is what
        // nests -- `make_radical` lays the degree out at a fixed
        // `Style::SCRIPT_SCRIPT` (LaTeX's `\r@@t`), so this exercises the
        // same list/atom/clean_box recursion through that specific slot.
        let degree = nest_groups(SANE_DEPTH_MULTIPLE, MathList::symbols("n"));
        let list: MathList = Atom::root(degree, MathList::symbols("x")).into();
        let report = layout_with_report(&list, Style::DISPLAY, &m);
        assert!(report.root.width.is_finite());
        assert!(report.root.height.is_finite());
        assert!(report.root.depth.is_finite());
        assert!(report.limitations.is_empty());
    });
}

#[test]
fn enormous_flat_list_completes_without_hanging() {
    // A flat list (no nesting) only ever recurses through `list()`'s single
    // iterative loop, so no extra stack is needed here -- the risk this
    // test rules out is a hang or a memory blow-up, not a stack overflow,
    // hence the wall-clock timeout via a channel instead of a big-stack
    // thread.
    const ATOM_COUNT: usize = 300_000;
    let (tx, rx) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        let m = CmMathMetrics::latex_10pt();
        let list = MathList::new(
            std::iter::repeat_with(|| Atom::symbol('x'))
                .take(ATOM_COUNT)
                .collect(),
        );
        let report = layout_with_report(&list, Style::TEXT, &m);
        let runs = positioned_runs(&report.root, (0.0, 0.0));
        let _ = tx.send((
            report.root.width,
            report.limitations.len(),
            runs.glyphs.len(),
        ));
    });
    let (width, limitations, glyphs) = rx
        .recv_timeout(std::time::Duration::from_secs(30))
        .expect("layout of a 300_000-atom flat list must complete well within 30s, not hang");
    assert!(width.is_finite());
    assert!(width > 0.0);
    assert_eq!(limitations, 0);
    assert_eq!(glyphs, ATOM_COUNT);
}

/// Wraps a real, otherwise fully working metrics provider but empties its
/// delimiter size chain (Rule 19's `var_delimiter` input, `left_right`'s
/// only source of delimiter glyphs) for every character, with no
/// extensible fallback either (this struct does not override
/// `delimiter_extensible`, so the trait's default — already `None` — wins).
struct NoDelimiters<'a>(&'a dyn MathFontMetrics);

impl MathFontMetrics for NoDelimiters<'_> {
    fn params(&self, size: SizeClass) -> MathParams {
        self.0.params(size)
    }
    fn font_name(&self, font: FontId) -> String {
        self.0.font_name(font)
    }
    fn glyph(&self, ch: char, size: SizeClass) -> Option<Glyph> {
        self.0.glyph(ch, size)
    }
    fn large_operator(&self, ch: char, size: SizeClass) -> Option<Glyph> {
        self.0.large_operator(ch, size)
    }
    fn delimiter_sizes(&self, _ch: char, _size: SizeClass) -> Vec<Glyph> {
        Vec::new()
    }
    fn radical_sizes(&self, size: SizeClass) -> Vec<Glyph> {
        self.0.radical_sizes(size)
    }
    fn accent_sizes(&self, ch: char, size: SizeClass) -> Vec<Glyph> {
        self.0.accent_sizes(ch, size)
    }
}

#[test]
fn delimiters_with_empty_size_chain_become_a_typed_limitation_not_a_panic() {
    let cm = CmMathMetrics::latex_10pt();
    let hostile = NoDelimiters(&cm);
    let list: MathList = Atom::left_right(Some('('), Some(')'), MathList::symbols("a")).into();
    let report = layout_with_report(&list, Style::TEXT, &hostile);
    // `var_delimiter`'s `sizes.last()?` on an empty `Vec` with no
    // extensible recipe returns `None`; `left_right_delimiter`'s `None`
    // arm turns that into a `Limitation::MissingGlyph` plus a
    // `null_delimiter_space` kern, for *each* delimiter -- a typed error,
    // not a panic, and the layout still completes.
    assert_eq!(
        report.limitations,
        vec![Limitation::MissingGlyph('('), Limitation::MissingGlyph(')')]
    );
    let p = hostile.params(SizeClass::Text);
    let inner_width = layout(&MathList::symbols("a"), Style::TEXT, &hostile).width;
    assert!(
        (report.root.width - (inner_width + 2.0 * p.null_delimiter_space)).abs() < 1e-9,
        "the bounded result is exactly the body plus two null-delimiter kerns: {} vs {}",
        report.root.width,
        inner_width + 2.0 * p.null_delimiter_space
    );
    let runs = positioned_runs(&report.root, (0.0, 0.0));
    assert_eq!(
        runs.glyphs.len(),
        1,
        "only \"a\"; no delimiter glyph exists to place"
    );
}

#[test]
fn scripts_on_an_empty_nucleus_produce_a_bounded_box_not_a_panic() {
    let m = CmMathMetrics::latex_10pt();
    // `{}_i^2`: a legitimately empty `{}` nucleus (not a missing glyph)
    // carrying both scripts, which is `make_scripts`'s `is_char = false`
    // path seeded from a zero-height/zero-depth nucleus box.
    let list: MathList = Atom::new(AtomClass::Ord, Nucleus::Empty)
        .with_sup(MathList::symbols("2"))
        .with_sub(MathList::symbols("i"))
        .into();
    let report = layout_with_report(&list, Style::TEXT, &m);
    assert!(
        report.limitations.is_empty(),
        "an empty nucleus is not a missing glyph"
    );
    assert!(report.root.width.is_finite() && report.root.width > 0.0);
    assert!(report.root.height.is_finite());
    assert!(report.root.depth.is_finite());
    let runs = positioned_runs(&report.root, (0.0, 0.0));
    assert_eq!(
        runs.glyphs.len(),
        2,
        "\"2\" and \"i\"; nothing for the empty nucleus itself"
    );
    assert_eq!(runs.rules.len(), 0);
}

fn zero_neg_glyph(ch: char) -> Glyph {
    Glyph {
        font_id: FontId(0),
        gid: ch as u32 as u16,
        ch,
        size: 0.0,
        width: -1.0,
        height: 0.0,
        depth: -1.0,
        italic: -0.5,
        skew: 0.0,
    }
}

fn zero_neg_params() -> MathParams {
    MathParams {
        size: 0.0,
        x_height: -1.0,
        quad: 0.0,
        num1: 0.0,
        num2: -1.0,
        num3: 0.0,
        denom1: 0.0,
        denom2: -1.0,
        sup1: 0.0,
        sup2: -1.0,
        sup3: 0.0,
        sub1: 0.0,
        sub2: -1.0,
        sup_drop: -1.0,
        sub_drop: -1.0,
        delim1: 0.0,
        delim2: -1.0,
        axis_height: -2.0,
        default_rule_thickness: -0.5,
        big_op_spacing1: 0.0,
        big_op_spacing2: -1.0,
        big_op_spacing3: 0.0,
        big_op_spacing4: -1.0,
        big_op_spacing5: 0.0,
        script_space: 0.0,
        null_delimiter_space: -1.0,
        delimiter_factor: 0.0,
        delimiter_shortfall: -5.0,
    }
}

/// Every metric this provider can hand back is zero or negative: font
/// parameters, glyph widths/heights/depths/italics, everything. TeX's own
/// arithmetic (subtracting depths, multiplying by fixed constants like
/// `0.6`/`3.0`/`0.25`, comparing against `wanted`) never divides by a
/// caller-supplied metric anywhere in this crate (checked: every division
/// in `src/layout.rs` is by a literal constant), so there is no
/// hostile-input division-by-zero/NaN path to begin with -- this test pins
/// that down as a tested contract rather than an unverified reading of the
/// source.
struct ZeroNegativeMetrics;

impl MathFontMetrics for ZeroNegativeMetrics {
    fn params(&self, _size: SizeClass) -> MathParams {
        zero_neg_params()
    }
    fn font_name(&self, _font: FontId) -> String {
        "zero-negative".to_string()
    }
    fn glyph(&self, ch: char, _size: SizeClass) -> Option<Glyph> {
        Some(zero_neg_glyph(ch))
    }
    fn large_operator(&self, _ch: char, _size: SizeClass) -> Option<Glyph> {
        None
    }
    fn delimiter_sizes(&self, ch: char, _size: SizeClass) -> Vec<Glyph> {
        vec![zero_neg_glyph(ch)]
    }
    fn radical_sizes(&self, _size: SizeClass) -> Vec<Glyph> {
        vec![zero_neg_glyph('\u{221A}')]
    }
    fn accent_sizes(&self, ch: char, _size: SizeClass) -> Vec<Glyph> {
        vec![zero_neg_glyph(ch)]
    }
}

/// A `{}` provider returning `None`/empty for every `Option`/`Vec`-returning
/// method: no glyph, no large-operator variant, no delimiter/radical/accent
/// size chain, for anything.
struct NoneMetrics;

impl MathFontMetrics for NoneMetrics {
    fn params(&self, _size: SizeClass) -> MathParams {
        // `params` cannot itself be `None`; give it ordinary, finite,
        // strictly-positive numbers so any limitation below is caused by
        // the *glyph-lookup* side returning `None`, not by degenerate
        // parameters (that combination is `ZeroNegativeMetrics`, above).
        MathParams {
            size: 10.0,
            x_height: 4.30554,
            quad: 18.0,
            num1: 6.77401,
            num2: 3.94398,
            num3: 4.4344,
            denom1: 6.85951,
            denom2: 3.44841,
            sup1: 4.13791,
            sup2: 3.62892,
            sup3: 2.89757,
            sub1: 1.5,
            sub2: 2.4694,
            sup_drop: 3.86,
            sub_drop: 0.5,
            delim1: 12.9,
            delim2: 10.35625,
            axis_height: 2.5,
            default_rule_thickness: 0.4,
            big_op_spacing1: 1.11111,
            big_op_spacing2: 1.66667,
            big_op_spacing3: 2.0,
            big_op_spacing4: 6.0,
            big_op_spacing5: 1.11111,
            script_space: 0.5,
            null_delimiter_space: 1.2,
            delimiter_factor: 0.901,
            delimiter_shortfall: 5.0,
        }
    }
    fn font_name(&self, _font: FontId) -> String {
        "none".to_string()
    }
    fn glyph(&self, _ch: char, _size: SizeClass) -> Option<Glyph> {
        None
    }
    fn large_operator(&self, _ch: char, _size: SizeClass) -> Option<Glyph> {
        None
    }
    fn delimiter_sizes(&self, _ch: char, _size: SizeClass) -> Vec<Glyph> {
        Vec::new()
    }
    fn radical_sizes(&self, _size: SizeClass) -> Vec<Glyph> {
        Vec::new()
    }
    fn accent_sizes(&self, _ch: char, _size: SizeClass) -> Vec<Glyph> {
        Vec::new()
    }
}

/// One atom of each nucleus kind this crate has (symbol via scripts,
/// fraction, radical-with-degree, accent, delimited, upright text, overline,
/// underline), so the hostile-provider tests below exercise every
/// `atom()`/`make_*` code path in one layout.
fn kitchen_sink() -> MathList {
    MathList::new(vec![
        Atom::frac(MathList::symbols("a"), MathList::symbols("b")),
        Atom::root(MathList::symbols("n"), MathList::symbols("x")),
        Atom::accent('^', MathList::symbols("y")),
        Atom::left_right(Some('('), Some(')'), MathList::symbols("z")),
        Atom::text_op("lim").with_sub(MathList::symbols("k")),
        Atom::overline(MathList::symbols("p")),
        Atom::underline(MathList::symbols("q")),
        Atom::symbol('w')
            .with_sup(MathList::symbols("2"))
            .with_sub(MathList::symbols("i")),
    ])
}

fn assert_finite_box(b: &MathBox) {
    assert!(b.width.is_finite(), "width {} is not finite", b.width);
    assert!(b.height.is_finite(), "height {} is not finite", b.height);
    assert!(b.depth.is_finite(), "depth {} is not finite", b.depth);
    if let BoxKind::HBox(children) | BoxKind::VBox(children) = &b.kind {
        for c in children {
            assert!(c.dx.is_finite(), "child dx {} is not finite", c.dx);
            assert!(c.dy.is_finite(), "child dy {} is not finite", c.dy);
            assert_finite_box(&c.content);
        }
    }
}

#[test]
fn zero_and_negative_metric_values_never_produce_nan_or_a_panic() {
    let m = ZeroNegativeMetrics;
    for style in [
        Style::DISPLAY,
        Style::TEXT,
        Style::SCRIPT,
        Style::SCRIPT_SCRIPT,
    ] {
        let report = layout_with_report(&kitchen_sink(), style, &m);
        assert_finite_box(&report.root);
        let runs = positioned_runs(&report.root, (0.0, 0.0));
        for g in &runs.glyphs {
            assert!(g.x.is_finite() && g.baseline_y.is_finite() && g.size.is_finite());
        }
        for r in &runs.rules {
            assert!(r.x.is_finite() && r.y.is_finite() && r.w.is_finite() && r.h.is_finite());
        }
    }
}

#[test]
fn metrics_provider_returning_none_for_everything_is_bounded_not_a_panic() {
    let m = NoneMetrics;
    for style in [
        Style::DISPLAY,
        Style::TEXT,
        Style::SCRIPT,
        Style::SCRIPT_SCRIPT,
    ] {
        let report = layout_with_report(&kitchen_sink(), style, &m);
        assert_finite_box(&report.root);
        // Every symbol/text character, the accent, the radical sign, and
        // both delimiters are missing; each becomes a typed `Limitation`
        // rather than a panic, and there is nothing else this provider
        // could report as missing.
        assert!(!report.limitations.is_empty());
        for lim in &report.limitations {
            assert!(
                matches!(
                    lim,
                    Limitation::MissingGlyph(_) | Limitation::MissingAccent(_)
                ),
                "unexpected limitation kind: {lim:?}"
            );
        }
        let runs = positioned_runs(&report.root, (0.0, 0.0));
        assert!(runs.glyphs.is_empty(), "no provider glyph exists to place");
    }
}

// ---------------------------------------------------------------------
// Malformed/Unicode string input: NUL bytes, non-NFC sequences, RTL
// overrides, and true empty input, through both `MathList::symbols` (one
// atom per `char`) and `Atom::text_op` (a single `Nucleus::Text(String)`).
// `char` and `String` are always well-formed Unicode scalar values in Rust,
// so none of this can produce invalid UTF-8; what it tests is that a
// metrics table with no entry for an unusual `char` degrades to a typed
// `Limitation::MissingGlyph`, never a panic, for every character class below.
// ---------------------------------------------------------------------

#[test]
fn nul_byte_in_symbols_and_text_op_is_a_typed_missing_glyph_not_a_panic() {
    let m = CmMathMetrics::latex_10pt();
    // `MathList::symbols`: one atom per `char`, so the NUL sits between two
    // ordinary, present glyphs.
    let list = MathList::symbols("a\u{0}b");
    assert_bounded_layout(&list, &m, 1);

    // `Atom::text_op`: the NUL sits inside a single `Nucleus::Text(String)`
    // instead of becoming its own atom -- a different code path
    // (`make_text`'s per-`char` loop) hitting the same hazard.
    let list: MathList = Atom::text_op("a\u{0}b").into();
    assert_bounded_layout(&list, &m, 1);

    // A lone NUL, with nothing else in the list at all.
    let list = MathList::symbols("\u{0}");
    assert_bounded_layout(&list, &m, 1);
}

#[test]
fn non_nfc_combining_sequences_are_bounded_not_a_panic() {
    let m = CmMathMetrics::latex_10pt();
    // `MathList::symbols` is defined over `char`s, so a combining sequence
    // ("e" + COMBINING ACUTE ACCENT, non-NFC) becomes two independent atoms,
    // never one grapheme -- deliberately: this crate takes no dependency on
    // Unicode normalization or grapheme segmentation, so there is no
    // normalization step whose absence could be hostile-input bait. Compare
    // the precomposed NFC form ('\u{00E9}', LATIN SMALL LETTER E WITH ACUTE),
    // which CM does not carry either: both must be bounded, and the
    // non-NFC form must produce exactly one *more* limitation than the
    // precomposed form (the combining mark's own missing glyph).
    let non_nfc = MathList::symbols("e\u{0301}");
    let precomposed = MathList::symbols("\u{00E9}");
    assert_bounded_layout(&non_nfc, &m, 1); // 'e' present, combining mark missing
    assert_bounded_layout(&precomposed, &m, 1); // 'é' itself missing

    let non_nfc_report = layout_with_report(&non_nfc, Style::TEXT, &m);
    let precomposed_report = layout_with_report(&precomposed, Style::TEXT, &m);
    assert_eq!(
        positioned_runs(&non_nfc_report.root, (0.0, 0.0))
            .glyphs
            .len(),
        1,
        "only 'e' has a glyph; the combining mark does not"
    );
    assert_eq!(
        positioned_runs(&precomposed_report.root, (0.0, 0.0))
            .glyphs
            .len(),
        0,
        "'\\u{{00E9}}' has no glyph at all in this table"
    );
}

#[test]
fn rtl_override_control_characters_are_bounded_not_a_panic() {
    let m = CmMathMetrics::latex_10pt();
    // RIGHT-TO-LEFT OVERRIDE, RIGHT-TO-LEFT MARK, LEFT-TO-RIGHT ISOLATE: all
    // three are ordinary `char`s to this crate (no bidi algorithm runs
    // here), so each is just one more symbol this metrics table has no
    // glyph for.
    let list = MathList::symbols("a\u{202E}b\u{200F}c\u{2066}d");
    assert_bounded_layout(&list, &m, 3);

    let list: MathList = Atom::text_op("a\u{202E}b\u{200F}c\u{2066}d").into();
    assert_bounded_layout(&list, &m, 3);
}

#[test]
fn empty_input_is_a_zero_size_box_not_a_panic() {
    let m = CmMathMetrics::latex_10pt();

    // A top-level list with zero atoms.
    let list = MathList::new(vec![]);
    let report = layout_with_report(&list, Style::TEXT, &m);
    assert_finite_box(&report.root);
    assert!(report.limitations.is_empty());
    assert_eq!(report.root.width, 0.0);
    let runs = positioned_runs(&report.root, (0.0, 0.0));
    assert_eq!(runs.glyphs.len(), 0);
    assert_eq!(runs.rules.len(), 0);

    // `MathList::symbols("")`: the same zero-atom list via the other
    // constructor.
    assert_bounded_layout(&MathList::symbols(""), &m, 0);

    // `Atom::text_op("")`: an empty `Nucleus::Text`, a different code path
    // (`make_text`) from an empty `MathList`.
    let list: MathList = Atom::text_op("").into();
    assert_bounded_layout(&list, &m, 0);

    // A braced empty group nested inside an otherwise-normal list
    // (`Atom::group` around an empty `MathList`, i.e. literal `{}`).
    let list = MathList::new(vec![Atom::symbol('a'), Atom::group(MathList::new(vec![]))]);
    assert_bounded_layout(&list, &m, 0);
}

#[test]
fn absurdly_long_text_op_string_completes_without_hanging() {
    // `enormous_flat_list_completes_without_hanging` above already covers an
    // absurd *list* length (300_000 atoms); this covers the other string
    // input this crate accepts, `Atom::text_op`'s single `Nucleus::Text`,
    // which is one `MathBox::hlist` built from a *single* `String`'s `chars()`
    // rather than 300_000 separate atoms -- a different allocation/iteration
    // shape, not exercised by the flat-list case.
    const CHAR_COUNT: usize = 1_000_000;
    let (tx, rx) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        let m = CmMathMetrics::latex_10pt();
        let text: String = std::iter::repeat_n('a', CHAR_COUNT).collect();
        let list: MathList = Atom::text_op(&text).into();
        let report = layout_with_report(&list, Style::TEXT, &m);
        let runs = positioned_runs(&report.root, (0.0, 0.0));
        let _ = tx.send((
            report.root.width,
            report.limitations.len(),
            runs.glyphs.len(),
        ));
    });
    let (width, limitations, glyphs) = rx
        .recv_timeout(std::time::Duration::from_secs(30))
        .expect("layout of a 1_000_000-char text_op must complete well within 30s, not hang");
    assert!(width.is_finite());
    assert!(width > 0.0);
    assert_eq!(
        limitations, 0,
        "plain.tex upright text has a CM roman glyph for 'a'"
    );
    assert_eq!(glyphs, CHAR_COUNT);
}
