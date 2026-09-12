//! One paired 500KB durable undo allocation observation; not a native latency benchmark.
use flashtex_edit_ledger::{history::HistoryMove, Document, Store};
use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering::Relaxed};
struct Count;
static ENABLED: AtomicBool = AtomicBool::new(false);
static CALLS: AtomicUsize = AtomicUsize::new(0);
static BYTES: AtomicUsize = AtomicUsize::new(0);
#[global_allocator]
static ALLOC: Count = Count;
unsafe impl GlobalAlloc for Count {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        if ENABLED.load(Relaxed) {
            CALLS.fetch_add(1, Relaxed);
            BYTES.fetch_add(layout.size(), Relaxed);
        }
        System.alloc(layout)
    }
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        System.dealloc(ptr, layout)
    }
    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, size: usize) -> *mut u8 {
        if ENABLED.load(Relaxed) {
            CALLS.fetch_add(1, Relaxed);
            BYTES.fetch_add(size, Relaxed);
        }
        System.realloc(ptr, layout, size)
    }
}
fn measured<T>(action: impl FnOnce() -> T) -> (T, usize, usize) {
    CALLS.store(0, Relaxed);
    BYTES.store(0, Relaxed);
    ENABLED.store(true, Relaxed);
    let result = action();
    ENABLED.store(false, Relaxed);
    (result, CALLS.load(Relaxed), BYTES.load(Relaxed))
}
fn main() {
    let a = tempfile::tempdir().unwrap();
    let b = tempfile::tempdir().unwrap();
    let mut full = Store::open(a.path()).unwrap();
    let original = Document::new("p".into(), "main.tex".into(), 1, "α".repeat(250_000)).unwrap();
    full.initialize(original.clone()).unwrap();
    std::fs::copy(
        a.path().join("document.json"),
        b.path().join("document.json"),
    )
    .unwrap();
    let mut compact = Store::open(b.path()).unwrap();
    for store in [&mut full, &mut compact] {
        store
            .replace_document(1, &original.source_sha256, "β".repeat(250_000))
            .unwrap();
    }
    let command = HistoryMove {
        command_id: "undo".into(),
        expected_revision: 2,
        expected_sha256: full.document().unwrap().unwrap().source_sha256.clone(),
    };
    let other = command.clone();
    let (f, fc, fb) = measured(|| full.undo(command).unwrap());
    let (c, cc, cb) = measured(|| compact.undo_status(other).unwrap());
    assert_eq!(f.document.text, original.text);
    assert_eq!(compact.document().unwrap().unwrap().text, original.text);
    assert_eq!(f.command_revision, c.command_revision);
    assert!(
        std::fs::read(a.path().join("document.json")).unwrap()
            == std::fs::read(b.path().join("document.json")).unwrap(),
        "persisted bytes differ"
    );
    println!(
        "{}",
        serde_json::json!({"source_bytes":500000,"action":"durable undo",
        "full":{"allocation_calls":fc,"requested_bytes":fb},
        "status":{"allocation_calls":cc,"requested_bytes":cb},
        "saved_requested_bytes":fb as i64-cb as i64,
        "scope":"single pair, counting allocation/reallocation requests during existing full API and status API; excludes setup, response encoding, native IO and paint; not peak memory or elapsed time"})
    );
}
