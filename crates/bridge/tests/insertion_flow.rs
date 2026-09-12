//! Insertion-flow coverage NOT already exercised by tests/bridge.rs. Deliberately does
//! not re-test utf8_rebase_and_reviewed_idempotent_edit, prepared_edit_does_not_survive_
//! unreviewed_source_change, rejection_is_durable_and_prevents_conversion_and_insertion,
//! exact_anchor_rehydration_allows_review_after_restart, dependency_change_requires_
//! explicit_reconversion_and_new_review, unrelated_document_does_not_invalidate_review,
//! or scalar_split_and_stale_revision_are_rejected -- those already cover the "happy path"
//! rebase/idempotence/rejection/dependency-staleness stories in tests/bridge.rs.
//!
//! This file targets: multiple anchors under one edit (position-relative correctness,
//! including a zero-width anchor sitting exactly at an edit's start byte), insertion at
//! document start/EOF, an empty document, UTF-8 scalar boundaries that split a single
//! user-perceived grapheme (emoji, combining marks, RTL text), untrusted-converter output
//! bounds, and receipt-state guards around reject()/confirm_insert() that bridge.rs does
//! not reach.
use base64::{engine::general_purpose::STANDARD, Engine};
use flashtex_bridge::{store::Store, *};
use std::io::Cursor;

fn image_base64() -> String {
    let mut bytes = Cursor::new(Vec::new());
    image::DynamicImage::new_rgb8(1, 1)
        .write_to(&mut bytes, image::ImageFormat::Png)
        .unwrap();
    STANDARD.encode(bytes.into_inner())
}
fn capture_for(capture_id: &str, destination_id: &str, base_revision: u64) -> CaptureSubmit {
    CaptureSubmit {
        capture_id: capture_id.into(),
        destination_id: destination_id.into(),
        base_revision,
        image: CaptureImage {
            mime_type: "image/png".into(),
            data_base64: image_base64(),
        },
        instructions: "Keep the notation".into(),
    }
}
struct LatexConverter(String);
impl Converter for LatexConverter {
    fn convert(&self, _: &CaptureSubmit, _: &Context) -> Result<Proposal> {
        Ok(Proposal {
            latex: self.0.clone(),
            ambiguities: vec![],
            required_dependencies: vec![],
        })
    }
}
fn fixed(latex: &str) -> LatexConverter {
    LatexConverter(latex.into())
}
fn open(b: &mut Bridge, text: &str) {
    b.open_document(Document {
        project_id: "project".into(),
        path: "main.tex".into(),
        revision: 1,
        text: text.into(),
    })
    .unwrap();
}

#[test]
fn edit_simultaneously_rebases_unaffected_and_invalidates_anchors_by_position() {
    let dir = tempfile::tempdir().unwrap();
    let mut b = Bridge::new(Store::open(dir.path()).unwrap());
    open(&mut b, "0123456789");
    b.pin("early", "project", "main.tex", 1, 2, 2).unwrap();
    // "touch" sits exactly at the edit's start byte. This is a zero-width anchor
    // swallowed by a same-position, non-empty replacement (not a pure insertion,
    // which is a different branch in Bridge::edit) -- it must still be invalidated
    // rather than silently left pointing at now-replaced content.
    b.pin("touch", "project", "main.tex", 1, 5, 5).unwrap();
    b.pin("late", "project", "main.tex", 1, 8, 8).unwrap();

    b.edit(&EditRequest {
        project_id: "project".into(),
        path: "main.tex".into(),
        base_revision: 1,
        revision: 2,
        start_byte: 5,
        end_byte: 7,
        replacement: "Z".into(),
    })
    .unwrap();
    assert_eq!(b.document("project", "main.tex").unwrap().text, "01234Z789");

    b.receive(capture_for("cap-early", "early", 1)).unwrap();
    b.convert("cap-early", vec![], &fixed("E")).unwrap();
    let edit = b.prepare_insert("cap-early", 2, true).unwrap();
    assert_eq!(
        (edit.start_byte, edit.end_byte),
        (2, 2),
        "an anchor strictly before the edited range must not move"
    );

    assert_eq!(
        b.receive(capture_for("cap-touch", "touch", 1))
            .unwrap_err()
            .code,
        "destination_reselection_required",
        "an anchor whose point the edit started exactly on must be invalidated, not left stale"
    );

    b.receive(capture_for("cap-late", "late", 1)).unwrap();
    b.convert("cap-late", vec![], &fixed("L")).unwrap();
    let edit = b.prepare_insert("cap-late", 2, true).unwrap();
    assert_eq!(
        (edit.start_byte, edit.end_byte),
        (7, 7),
        "an anchor strictly after the edited range must shift by the length delta"
    );
    b.confirm_insert("cap-late", &edit.edit_id, 3).unwrap();
    assert_eq!(
        b.document("project", "main.tex").unwrap().text,
        "01234Z7L89"
    );
}

#[test]
fn insert_at_document_start_and_end_produce_exact_text() {
    let dir = tempfile::tempdir().unwrap();
    let mut b = Bridge::new(Store::open(dir.path()).unwrap());
    open(&mut b, "hello");
    b.pin("start", "project", "main.tex", 1, 0, 0).unwrap();
    b.pin("end", "project", "main.tex", 1, 5, 5).unwrap();

    b.receive(capture_for("cap-start", "start", 1)).unwrap();
    b.convert("cap-start", vec![], &fixed("A")).unwrap();
    let edit = b.prepare_insert("cap-start", 1, true).unwrap();
    assert_eq!((edit.start_byte, edit.end_byte), (0, 0));
    b.confirm_insert("cap-start", &edit.edit_id, 2).unwrap();
    assert_eq!(b.document("project", "main.tex").unwrap().text, "Ahello");

    // "end" was pinned at the original EOF (byte 5); the start-of-document insertion
    // must rebase it forward to the new EOF rather than leaving it pointing mid-string.
    b.receive(capture_for("cap-end", "end", 1)).unwrap();
    b.convert("cap-end", vec![], &fixed("Z")).unwrap();
    let edit = b.prepare_insert("cap-end", 2, true).unwrap();
    assert_eq!((edit.start_byte, edit.end_byte), (6, 6));
    b.confirm_insert("cap-end", &edit.edit_id, 3).unwrap();
    assert_eq!(b.document("project", "main.tex").unwrap().text, "AhelloZ");
}

#[test]
fn empty_document_supports_pin_and_insertion_at_position_zero() {
    let dir = tempfile::tempdir().unwrap();
    let mut b = Bridge::new(Store::open(dir.path()).unwrap());
    open(&mut b, "");
    b.pin("only", "project", "main.tex", 1, 0, 0).unwrap();
    b.receive(capture_for("cap-1", "only", 1)).unwrap();
    b.convert("cap-1", vec![], &fixed("$x$")).unwrap();
    let edit = b.prepare_insert("cap-1", 1, true).unwrap();
    assert_eq!(edit.removed_text, "");
    b.confirm_insert("cap-1", &edit.edit_id, 2).unwrap();
    assert_eq!(b.document("project", "main.tex").unwrap().text, "$x$");
}

/// UTF-8 scalar boundaries are not grapheme-cluster boundaries. `range()` (src/lib.rs)
/// only guarantees `is_char_boundary`, so a pinned anchor can legally sit between the two
/// codepoints of a single flag emoji, or between a base letter and its combining accent.
/// This confirms that splitting a grapheme this way never panics and still splices at the
/// exact byte offset requested -- it is a documented "scalars, not graphemes" contract,
/// not a crash.
#[test]
fn scalar_boundaries_split_grapheme_clusters_without_panicking() {
    let dir = tempfile::tempdir().unwrap();
    let flag = "\u{1F1FA}\u{1F1F8}"; // US flag: two regional-indicator scalars, one glyph
    let rtl = "\u{05E9}\u{05DC}\u{05D5}\u{05DD}"; // Hebrew "shalom", right-to-left
    let flag_mid = flag.chars().next().unwrap().len_utf8();
    let mark_mid = flag.len() + "e".len();
    let text = format!("{flag}e\u{0301}{rtl}"); // flag + 'e' + combining acute + RTL text
    assert!(text.is_char_boundary(flag_mid));
    assert!(text.is_char_boundary(mark_mid));

    let mut b = Bridge::new(Store::open(dir.path()).unwrap());
    open(&mut b, &text);
    b.pin("flag-split", "project", "main.tex", 1, flag_mid, flag_mid)
        .unwrap();
    b.pin("mark-split", "project", "main.tex", 1, mark_mid, mark_mid)
        .unwrap();

    // Insert exactly between the flag's two scalars: legal per UTF-8 scalar boundaries,
    // even though it visually splits one emoji glyph in half.
    b.receive(capture_for("cap-flag", "flag-split", 1)).unwrap();
    b.convert("cap-flag", vec![], &fixed("<F>")).unwrap();
    let edit = b.prepare_insert("cap-flag", 1, true).unwrap();
    assert_eq!(edit.start_byte, flag_mid);
    b.confirm_insert("cap-flag", &edit.edit_id, 2).unwrap();
    let mut expected_after_flag = String::new();
    expected_after_flag.push_str(&flag[..flag_mid]);
    expected_after_flag.push_str("<F>");
    expected_after_flag.push_str(&flag[flag_mid..]);
    expected_after_flag.push('e');
    expected_after_flag.push('\u{0301}');
    expected_after_flag.push_str(rtl);
    assert_eq!(
        b.document("project", "main.tex").unwrap().text,
        expected_after_flag
    );

    // The second anchor (between the base letter and its combining accent) must have
    // rebased across the first insertion without landing mid-scalar.
    b.receive(capture_for("cap-mark", "mark-split", 1)).unwrap();
    b.convert("cap-mark", vec![], &fixed("<M>")).unwrap();
    let edit = b.prepare_insert("cap-mark", 2, true).unwrap();
    assert_eq!(edit.start_byte, mark_mid + "<F>".len());
    b.confirm_insert("cap-mark", &edit.edit_id, 3).unwrap();
    let mut expected_final = String::new();
    expected_final.push_str(&flag[..flag_mid]);
    expected_final.push_str("<F>");
    expected_final.push_str(&flag[flag_mid..]);
    expected_final.push('e');
    expected_final.push_str("<M>");
    expected_final.push('\u{0301}');
    expected_final.push_str(rtl);
    assert_eq!(
        b.document("project", "main.tex").unwrap().text,
        expected_final
    );
}

#[test]
fn oversized_and_malformed_converter_output_never_mutates_document_or_journal() {
    let dir = tempfile::tempdir().unwrap();
    let mut b = Bridge::new(Store::open(dir.path()).unwrap());
    open(&mut b, "keep me exactly as-is");
    b.pin("anchor-1", "project", "main.tex", 1, 5, 5).unwrap();
    let original = b.document("project", "main.tex").unwrap().clone();

    let oversized = "x".repeat(MAX_LATEX_BYTES + 1);
    let with_nul = "$x$\0evil";
    for (name, latex) in [
        ("oversized", oversized.as_str()),
        ("embedded_nul", with_nul),
    ] {
        let capture_id = format!("cap-{name}");
        b.receive(capture_for(&capture_id, "anchor-1", 1)).unwrap();
        let error = b.convert(&capture_id, vec![], &fixed(latex)).unwrap_err();
        assert_eq!(error.code, "invalid_proposal", "{name}");
        assert!(
            b.store.require(&capture_id).unwrap().proposal.is_none(),
            "{name}: a rejected proposal must not be journaled"
        );
        assert_eq!(
            b.document("project", "main.tex").unwrap(),
            &original,
            "{name}: a rejected conversion must not touch the document"
        );
    }
}

/// FINDING (see delegation report): `Proposal::validate` (src/lib.rs) bounds only the
/// size and NUL-byte content of `latex`; it performs no structural or security-relevant
/// inspection of the untrusted model's output. Nothing in `prepare_insert`/
/// `confirm_insert` requires that the optional compiler-backed `validate_capture`
/// round-trip (src/validation.rs) ever ran. This test documents the resulting fact:
/// a shell-escape primitive, an absolute-path `\input`, a redefined `\documentclass`,
/// and unbalanced braces all flow verbatim into the user's document once a caller sets
/// `approved: true`. There is no independent enforcement boundary inside this crate for
/// any of it. This is reported as a security-posture question for a human decision, not
/// asserted here as either correct or incorrect.
#[test]
fn adversarial_latex_from_the_model_receives_no_content_level_validation() {
    let dir = tempfile::tempdir().unwrap();
    let mut b = Bridge::new(Store::open(dir.path()).unwrap());
    open(&mut b, "before AFTER");
    b.pin("anchor-1", "project", "main.tex", 1, 7, 7).unwrap();

    let payloads = [
        "\\write18{rm -rf ~}",
        "\\input{/etc/passwd}",
        "\\documentclass{evil}\\begin{document}",
        "$x${{{unbalanced",
    ];
    for (i, payload) in payloads.into_iter().enumerate() {
        let capture_id = format!("cap-adv-{i}");
        b.receive(capture_for(&capture_id, "anchor-1", 1)).unwrap();
        let record = b
            .convert(&capture_id, vec![], &fixed(payload))
            .expect("Proposal::validate has no content checks beyond size/NUL bytes");
        assert_eq!(record.proposal.unwrap().latex, payload);
        let edit = b.prepare_insert(&capture_id, 1, true).unwrap();
        assert_eq!(edit.replacement, payload);
    }

    // Confirm one lands byte-for-byte in the real document text, not just the receipt.
    b.receive(capture_for("cap-adv-confirm", "anchor-1", 1))
        .unwrap();
    b.convert("cap-adv-confirm", vec![], &fixed(payloads[0]))
        .unwrap();
    let edit = b.prepare_insert("cap-adv-confirm", 1, true).unwrap();
    b.confirm_insert("cap-adv-confirm", &edit.edit_id, 2)
        .unwrap();
    assert_eq!(
        b.document("project", "main.tex").unwrap().text,
        format!("before {}AFTER", payloads[0])
    );
}

#[test]
fn reject_is_refused_once_an_edit_has_been_prepared_or_applied() {
    let dir = tempfile::tempdir().unwrap();
    let mut b = Bridge::new(Store::open(dir.path()).unwrap());
    open(&mut b, "keep");
    b.pin("anchor-1", "project", "main.tex", 1, 4, 4).unwrap();
    b.receive(capture_for("cap-1", "anchor-1", 1)).unwrap();
    b.convert("cap-1", vec![], &fixed("$x$")).unwrap();
    let edit = b.prepare_insert("cap-1", 1, true).unwrap();

    // A prepared-but-unconfirmed edit is an issued receipt: silently rejecting it here
    // would let the bridge disagree with a Mac client that already has the edit in hand.
    assert_eq!(b.reject("cap-1").unwrap_err().code, "receipt_conflict");
    assert!(!b.store.require("cap-1").unwrap().rejected);

    b.confirm_insert("cap-1", &edit.edit_id, 2).unwrap();
    assert_eq!(b.reject("cap-1").unwrap_err().code, "receipt_conflict");
    assert!(!b.store.require("cap-1").unwrap().rejected);
    assert_eq!(b.document("project", "main.tex").unwrap().text, "keep$x$");
}

#[test]
fn confirm_insert_rejects_non_advancing_revision_and_wrong_edit_id() {
    let dir = tempfile::tempdir().unwrap();
    let mut b = Bridge::new(Store::open(dir.path()).unwrap());
    open(&mut b, "keep");
    b.pin("anchor-1", "project", "main.tex", 1, 4, 4).unwrap();
    b.receive(capture_for("cap-1", "anchor-1", 1)).unwrap();
    b.convert("cap-1", vec![], &fixed("$x$")).unwrap();
    let edit = b.prepare_insert("cap-1", 1, true).unwrap();

    assert_eq!(
        b.confirm_insert("cap-1", &edit.edit_id, 1)
            .unwrap_err()
            .code,
        "receipt_conflict",
        "new_revision must strictly advance past expected_revision"
    );
    assert_eq!(
        b.confirm_insert("cap-1", &edit.edit_id, 0)
            .unwrap_err()
            .code,
        "receipt_conflict"
    );
    assert_eq!(
        b.confirm_insert("cap-1", "wrong-edit-id", 2)
            .unwrap_err()
            .code,
        "receipt_conflict"
    );
    assert!(b.store.require("cap-1").unwrap().applied.is_none());
    assert_eq!(b.document("project", "main.tex").unwrap().text, "keep");

    b.confirm_insert("cap-1", &edit.edit_id, 2).unwrap();
    assert_eq!(b.document("project", "main.tex").unwrap().text, "keep$x$");
}
