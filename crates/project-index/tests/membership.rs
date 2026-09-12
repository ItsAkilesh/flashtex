use flashtex_project_index::{Category, DocumentKind, ProjectIndex};
#[test]
fn membership_is_atomic_monotonic_and_retains_revision_watermarks() {
    let mut index = ProjectIndex::new("p").unwrap();
    index.replace_document("main.tex", 1, "main").unwrap();
    let initial = index.snapshot();
    let main = ("main.tex", 1, "main", DocumentKind::Latex);
    let chapter = ("chapter.tex", 4, "\\label{chapter}", DocumentKind::Latex);
    index
        .replace_membership(&initial, &[main, chapter])
        .unwrap();
    let added = index.snapshot();
    index.replace_membership(&added, &[main]).unwrap();
    let removed = index.snapshot();
    assert!(index.replace_document("chapter.tex", 4, "stale").is_err());
    assert!(index.replace_membership(&added, &[main, chapter]).is_err());
    index
        .replace_membership(&removed, &[main, chapter])
        .unwrap();
    let reopened = index.snapshot();
    assert!(reopened.generation > removed.generation);
    assert!(index
        .definitions(&added, Category::Label, "chapter")
        .is_err());
    assert_eq!(
        index
            .definitions(&reopened, Category::Label, "chapter")
            .unwrap()
            .len(),
        1
    );
    assert!(index
        .replace_membership(
            &reopened,
            &[main, ("chapter.tex", 3, "old", DocumentKind::Latex)]
        )
        .is_err());
    assert!(index.replace_membership(&reopened, &[main, main]).is_err());
    assert!(index
        .replace_membership(
            &reopened,
            &[("main.tex", 1, "different", DocumentKind::Latex)]
        )
        .is_err());
    assert_eq!(index.snapshot(), reopened);
}
