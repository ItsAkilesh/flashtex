//! FT-041 rev 4, objective 1: an integration fixture against the *actual*
//! existing consumer of a computed plan, not an imagined one.
//!
//! ## Where the real consumer lives, and why this isn't Swift
//!
//! `apps/mac` (the Swift editor) has no dependency on this crate, or on any
//! Rust crate at all: its `Package.swift` builds `FlashTeXMac` against only
//! `FlashTeXProtocol` and `FlashTeXAccessibility`, both pure Swift. A repo
//! grep for `editor-snippets` / `flashtex_editor_snippets` / `SnippetPlan`
//! outside this crate's own directory returns nothing. So there is no FFI
//! or Cargo edge for a test in this crate to cross into Swift, and nothing
//! honest to call there — this crate has no Swift consumer today.
//!
//! What Mac and the Rust worker actually share is a JSON Lines wire
//! contract (`docs/contracts/runtime-v1.md`, `docs/contracts/transfer-v1.md`):
//! snake_case fields, zero-based end-exclusive UTF-8 byte offsets that Swift
//! converts explicitly (`apps/mac/Sources/FlashTeXProtocol/ByteOffsets.swift`,
//! `RuntimeV1.SourceRange`). The exact edit shape that contract carries is
//! `flashtex_edit_ledger::PreparedEdit`, whose own doc comment says it is
//! "Wire-compatible with transfer-v1 `PreparedEdit`"
//! (`crates/edit-ledger/src/lib.rs`) — and `docs/contracts/transfer-v1.md`
//! spells out that exact field list (`capture_id,edit_id,project_id,path,
//! expected_revision,start_byte,end_byte,removed_text,replacement,
//! document_before_sha256`), which is byte-for-byte what
//! `apps/mac/Sources/FlashTeXProtocol/TransferV1.swift`'s `CaptureEdit`
//! `CodingKeys` decode. `field_names_match_the_transfer_v1_wire_contract`
//! below pins that alignment directly against `PreparedEdit`'s own
//! `Serialize` impl, so this crate's output is proven to fit the exact
//! bytes that would cross the language boundary — the contract the bridge
//! actually carries — without pretending to invoke Swift.
//!
//! ## The real, same-language consumer this fixture drives
//!
//! `crates/edit-ledger` ("Durable document and reviewed-edit transactions
//! for FlashTeX native consumers") is the actual, in-repo, production
//! consumer that turns a byte-offset edit into a mutated, durable document.
//! It is same-language (Rust), so it can be exercised directly rather than
//! reimplemented: this crate is added only as a `[dev-dependencies]` path
//! dependency (see `Cargo.toml`), never touched or edited, and every test
//! here produces a `SnippetPlan` and applies it through `Store::apply` /
//! `Store::apply_group` exactly as that crate's own tests do.

use std::collections::HashMap;

use flashtex_edit_ledger::history::{GroupedEdit, SourceEdit};
use flashtex_edit_ledger::{Document, PreparedEdit, Store};
use flashtex_editor_snippets::{Anchor, DocumentId, Snippet, SnippetPlan};

/// A fresh, process- and call-unique temp directory for one `Store`. Real
/// filesystem state (`Store::open` creates a lock file and a directory),
/// scoped and removed per test so parallel test threads never collide.
fn temp_store_dir(name: &str) -> std::path::PathBuf {
    use std::sync::atomic::{AtomicU64, Ordering};
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let unique = COUNTER.fetch_add(1, Ordering::Relaxed);
    let mut dir = std::env::temp_dir();
    dir.push(format!(
        "flashtex-editor-snippets-consumer-fixture-{name}-{}-{unique}",
        std::process::id()
    ));
    dir
}

struct RealStore {
    store: Store,
    dir: std::path::PathBuf,
}

impl RealStore {
    fn new(name: &str, project_id: &str, path: &str, text: &str) -> Self {
        let dir = temp_store_dir(name);
        let mut store = Store::open(&dir).expect("open a fresh real edit-ledger store");
        store
            .initialize(
                Document::new(
                    project_id.to_string(),
                    path.to_string(),
                    0,
                    text.to_string(),
                )
                .expect("construct the initial document"),
            )
            .expect("initialize the store with the starting document");
        Self { store, dir }
    }

    fn document(&self) -> Document {
        self.store
            .document()
            .expect("store is not poisoned")
            .expect("document was initialized")
            .clone()
    }
}

impl Drop for RealStore {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.dir);
    }
}

/// Objective 1, part A: a plan's insertion, applied through the real
/// consumer's single-edit path (`Store::apply`), lands every placeholder
/// occurrence exactly where this crate said it would.
#[test]
fn snippet_plan_insertion_survives_the_real_edit_ledger_store() {
    // A LaTeX list-environment snippet: $1 is linked between \begin and
    // \end (double backslash is this crate's own escape for a literal
    // backslash — see tests/corpus_snippet_coverage.rs for why that is
    // necessary today), $0 is the final tab stop.
    let snippet = Snippet::parse("\\\\begin{${1:itemize}}\n\t\\\\item $0\n\\\\end{$1}").unwrap();
    let doc_text = "Intro paragraph.\n\n";
    let caret = doc_text.len();
    let plan = SnippetPlan::compute(&snippet, 0, doc_text, Anchor::Caret(caret), &HashMap::new())
        .expect("plan computes against a valid caret");

    let real = RealStore::new("insert", "proj", "main.tex", doc_text);
    let before = real.document();
    assert_eq!(before.revision, plan.document().revision);

    let edit = PreparedEdit {
        capture_id: "cap-insert".to_string(),
        edit_id: "edit-insert".to_string(),
        project_id: before.project_id.clone(),
        path: before.path.clone(),
        expected_revision: plan.document().revision,
        start_byte: caret,
        end_byte: caret,
        removed_text: String::new(),
        replacement: plan.expansion().text.clone(),
        document_before_sha256: before.source_sha256.clone(),
    };

    let mut real = real;
    let receipt = real
        .store
        .apply(edit)
        .expect("the real consumer accepts this crate's own plan unmodified");
    assert_eq!(receipt.new_revision, 1);

    let after = real.document();
    assert_eq!(
        after.text.len(),
        doc_text.len() + plan.expansion().text.len()
    );

    // Every occurrence this crate reported for the linked placeholder lands,
    // after a *real* byte-range splice performed by the actual consumer, on
    // exactly the text it named. This is the load-bearing assertion: it
    // would fail if this crate's byte offsets were off by even one byte, or
    // if a linked occurrence pointed at the wrong copy.
    let occurrences = plan.expansion().occurrences_of(1);
    assert_eq!(
        occurrences.len(),
        2,
        "begin{{X}} and end{{X}} both carry $1"
    );
    for occurrence in occurrences {
        let absolute = (caret + occurrence.start)..(caret + occurrence.end);
        assert_eq!(&after.text[absolute], "itemize");
    }
    assert!(after.text.contains("\\begin{itemize}"));
    assert!(after.text.contains("\\end{itemize}"));
}

/// Objective 1, part B: when the user tabs to the linked placeholder and
/// retypes it, a real editor must turn that one keystroke into edits at
/// *every* linked occurrence, applied atomically. `GroupedEdit`/`SourceEdit`
/// (`crates/edit-ledger/src/history.rs`) is the actual mechanism the real
/// consumer uses for exactly this: "nonoverlapping byte ranges in the same
/// original source snapshot" applied as one transaction. This proves
/// `Expansion::occurrences_of` hands back exactly the ranges that mechanism
/// needs to keep every linked occurrence in sync.
#[test]
fn linked_placeholder_edit_propagates_through_a_grouped_edit_transaction() {
    let snippet = Snippet::parse("\\\\begin{${1:itemize}}\n\t\\\\item $0\n\\\\end{$1}").unwrap();
    let plan = SnippetPlan::compute(&snippet, 0, "", Anchor::Caret(0), &HashMap::new()).unwrap();

    let mut real = RealStore::new("grouped", "proj", "main.tex", "");
    let before = real.document();
    real.store
        .apply(PreparedEdit {
            capture_id: "cap-insert".to_string(),
            edit_id: "edit-insert".to_string(),
            project_id: before.project_id.clone(),
            path: before.path.clone(),
            expected_revision: 0,
            start_byte: 0,
            end_byte: 0,
            removed_text: String::new(),
            replacement: plan.expansion().text.clone(),
            document_before_sha256: before.source_sha256.clone(),
        })
        .unwrap();

    let after_insert = real.document();
    let occurrences = plan.expansion().occurrences_of(1).to_vec();
    assert_eq!(occurrences.len(), 2);

    // Build one SourceEdit per linked occurrence, exactly as a real editor's
    // "retype this tab stop" handler would from this crate's own output.
    let edits: Vec<SourceEdit> = occurrences
        .iter()
        .map(|occurrence| SourceEdit {
            start_byte: occurrence.start,
            end_byte: occurrence.end,
            removed_text: after_insert.text[occurrence.clone()].to_string(),
            replacement: "enumerate".to_string(),
        })
        .collect();
    assert!(edits.iter().all(|edit| edit.removed_text == "itemize"));

    real.store
        .apply_group(GroupedEdit {
            command_id: "retype-placeholder-1".to_string(),
            expected_revision: after_insert.revision,
            expected_sha256: after_insert.source_sha256.clone(),
            label: "rename list environment".to_string(),
            edits,
        })
        .expect("the real consumer applies every linked occurrence as one atomic transaction");

    let after_rename = real.document();
    assert!(after_rename.text.contains("\\begin{enumerate}"));
    assert!(after_rename.text.contains("\\end{enumerate}"));
    assert!(!after_rename.text.contains("itemize"));
}

/// Rev 2's staleness contract, checked against the real consumer rather
/// than only against this crate's own types: a plan computed at revision 0
/// must be refused once the real store has actually moved on, and this
/// crate's own `staleness()` must have already said so before the real
/// consumer is ever asked.
#[test]
fn stale_plan_is_rejected_by_the_real_store_exactly_when_this_crate_predicts() {
    let snippet = Snippet::parse("${1:x}").unwrap();
    let plan = SnippetPlan::compute(&snippet, 0, "", Anchor::Caret(0), &HashMap::new()).unwrap();

    let mut real = RealStore::new("stale", "proj", "main.tex", "");
    let before = real.document();
    // An unrelated edit moves the real document to revision 1 out from
    // under the plan, without the plan ever being told.
    real.store
        .apply(PreparedEdit {
            capture_id: "cap-unrelated".to_string(),
            edit_id: "edit-unrelated".to_string(),
            project_id: before.project_id.clone(),
            path: before.path.clone(),
            expected_revision: 0,
            start_byte: 0,
            end_byte: 0,
            removed_text: String::new(),
            replacement: "% unrelated edit\n".to_string(),
            document_before_sha256: before.source_sha256.clone(),
        })
        .unwrap();

    let current = real.document();
    let current_id = DocumentId::new(current.revision, &current.text);
    assert!(
        !plan.staleness(&current_id).is_fresh(),
        "this crate must flag the plan stale before the real consumer is ever asked"
    );

    let result = real.store.apply(PreparedEdit {
        capture_id: "cap-stale".to_string(),
        edit_id: "edit-stale".to_string(),
        project_id: current.project_id.clone(),
        path: current.path.clone(),
        expected_revision: plan.document().revision, // still 0: stale
        start_byte: 0,
        end_byte: 0,
        removed_text: String::new(),
        replacement: plan.expansion().text.clone(),
        document_before_sha256: flashtex_edit_ledger::digest(""),
    });
    assert!(
        result.is_err(),
        "the real consumer must independently refuse the same stale plan"
    );
}

/// Objective 1's cross-language claim, pinned directly: this crate's output,
/// carried inside the real wire type (`PreparedEdit`), serializes to the
/// exact field set `docs/contracts/transfer-v1.md` specifies and
/// `TransferV1.CaptureEdit` in `apps/mac/Sources/FlashTeXProtocol/TransferV1.swift`
/// decodes via `CodingKeys` — snake_case, and nothing more or less.
#[test]
fn field_names_match_the_transfer_v1_wire_contract() {
    let snippet = Snippet::parse("${1:x}").unwrap();
    let plan = SnippetPlan::compute(&snippet, 0, "", Anchor::Caret(0), &HashMap::new()).unwrap();

    let edit = PreparedEdit {
        capture_id: "cap-1".to_string(),
        edit_id: "edit-1".to_string(),
        project_id: "proj".to_string(),
        path: "main.tex".to_string(),
        expected_revision: plan.document().revision,
        start_byte: 0,
        end_byte: 0,
        removed_text: String::new(),
        replacement: plan.expansion().text.clone(),
        document_before_sha256: flashtex_edit_ledger::digest(""),
    };

    let json = serde_json::to_value(&edit).unwrap();
    let object = json.as_object().unwrap();
    let mut keys: Vec<&str> = object.keys().map(String::as_str).collect();
    keys.sort_unstable();
    // Exactly the field list docs/contracts/transfer-v1.md names for a
    // prepared edit, in the same snake_case TransferV1.CaptureEdit decodes.
    let mut expected = [
        "capture_id",
        "edit_id",
        "project_id",
        "path",
        "expected_revision",
        "start_byte",
        "end_byte",
        "removed_text",
        "replacement",
        "document_before_sha256",
    ];
    expected.sort_unstable();
    assert_eq!(keys, expected);
}
