use flashtex_project_index::*;
use std::collections::BTreeSet;

fn index() -> ProjectIndex {
    let mut index = ProjectIndex::new("search").unwrap();
    index.replace_document("z.tex", 7, "é文 é文").unwrap();
    index
        .replace_document("a.tex", 2, "% é文\n\\verb|é文| aa aaa")
        .unwrap();
    index
}

#[test]
fn literal_matches_include_comments_and_verbatim_in_exact_utf8_order() {
    let index = index();
    let snapshot = index.snapshot();
    let result = index
        .search_literal(&snapshot, &SearchRequest::literal("é文"), || false)
        .unwrap();
    assert_eq!(result.termination, SearchTermination::Complete);
    assert_eq!(
        result
            .matches
            .iter()
            .map(|s| s.file.as_str())
            .collect::<Vec<_>>(),
        ["a.tex", "a.tex", "z.tex", "z.tex"]
    );
    for source in result.matches {
        assert_eq!(index.source_text(&snapshot, &source).unwrap(), "é文");
    }
    let result = index
        .search_literal(&snapshot, &SearchRequest::literal("aa"), || false)
        .unwrap();
    assert_eq!(result.matches.len(), 2); // aaa contributes one nonoverlapping match.
}

#[test]
fn selection_is_explicit_and_all_paths_validated_before_scanning() {
    let index = index();
    let mut request = SearchRequest::literal("é文");
    request.documents = Some(BTreeSet::from(["z.tex".into()]));
    assert_eq!(
        index
            .search_literal(&index.snapshot(), &request, || false)
            .unwrap()
            .matches
            .len(),
        2
    );
    request.documents = Some(BTreeSet::new());
    let empty = index
        .search_literal(&index.snapshot(), &request, || false)
        .unwrap();
    assert_eq!(empty.termination, SearchTermination::Complete);
    assert_eq!(empty.work_used, 0);
    request.documents = Some(BTreeSet::from(["missing.tex".into()]));
    assert_eq!(
        index.search_literal(&index.snapshot(), &request, || false),
        Err(IndexError::MissingDocument)
    );
    request.documents = Some(BTreeSet::from(["../a.tex".into()]));
    assert_eq!(
        index.search_literal(&index.snapshot(), &request, || false),
        Err(IndexError::InvalidPath)
    );
}

#[test]
fn match_work_and_cancellation_limits_remain_distinct() {
    let index = index();
    let mut request = SearchRequest::literal("é文");
    request.max_matches = 1;
    let result = index
        .search_literal(&index.snapshot(), &request, || false)
        .unwrap();
    assert_eq!(result.termination, SearchTermination::MatchLimit);
    assert_eq!(result.matches.len(), 1);
    request.max_matches = 100;
    request.max_work = 2;
    let result = index
        .search_literal(&index.snapshot(), &request, || false)
        .unwrap();
    assert_eq!(result.termination, SearchTermination::WorkLimit);
    assert_eq!(result.work_used, 2); // Includes preprocessing.
    request.max_work = 100;
    let mut polls = 0;
    let result = index
        .search_literal(&index.snapshot(), &request, || {
            polls += 1;
            polls > 10
        })
        .unwrap();
    assert_eq!(result.termination, SearchTermination::Cancelled);
    assert_eq!(result.work_used, 9);
    request.max_work = 0;
    assert_eq!(
        index
            .search_literal(&index.snapshot(), &request, || true)
            .unwrap()
            .termination,
        SearchTermination::Cancelled
    );
}

#[test]
fn guards_and_edge_limits_are_deterministic() {
    let mut index = index();
    let old = index.snapshot();
    assert_eq!(
        index.search_literal(&old, &SearchRequest::literal(""), || false),
        Err(IndexError::InvalidSearchRequest)
    );
    let mut request = SearchRequest::literal("文");
    request.max_matches = MAX_SEARCH_MATCHES + 1;
    assert_eq!(
        index.search_literal(&old, &request, || false),
        Err(IndexError::InvalidSearchRequest)
    );
    request.max_matches = 0;
    assert_eq!(
        index
            .search_literal(&old, &request, || false)
            .unwrap()
            .termination,
        SearchTermination::MatchLimit
    );
    index.replace_document("a.tex", 3, "new").unwrap();
    assert_eq!(
        index.search_literal(&old, &request, || false),
        Err(IndexError::StaleSnapshot)
    );
}

#[test]
fn kmp_matches_standard_nonoverlapping_literal_search() {
    for source in ["aaaaabaaaab", "éé文é文", "ababababbabab", "", "no matches"] {
        for needle in [
            "a",
            "aa",
            "aaaab",
            "abab",
            "é",
            "é文",
            "longer than any input source here",
        ] {
            let mut index = ProjectIndex::new("search").unwrap();
            index.replace_document("a.tex", 1, source).unwrap();
            let result = index
                .search_literal(&index.snapshot(), &SearchRequest::literal(needle), || false)
                .unwrap();
            assert_eq!(result.termination, SearchTermination::Complete);
            assert_eq!(
                result
                    .matches
                    .iter()
                    .map(|s| s.start_byte)
                    .collect::<Vec<_>>(),
                source
                    .match_indices(needle)
                    .map(|(at, _)| at)
                    .collect::<Vec<_>>(),
                "{source} / {needle}"
            );
        }
    }
}
