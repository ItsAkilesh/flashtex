//! Literal text operations, independent of parsed symbols or TeX semantics.
use super::{span, IndexError, ProjectIndex, SourceSpan, VersionSnapshot};
use std::collections::BTreeSet;

pub const MAX_SEARCH_QUERY_BYTES: usize = 64 * 1024;
pub const MAX_SEARCH_MATCHES: usize = 100_000;
pub const MAX_REPLACEMENT_PLAN_TEXT_BYTES: usize = 8 * 1024 * 1024;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SearchRequest {
    pub literal: String,
    /// None searches every document; Some(empty) searches none.
    pub documents: Option<BTreeSet<String>>,
    pub max_matches: usize,
    /// Byte comparisons, including prefix-table construction, not elapsed time.
    pub max_work: usize,
}
impl SearchRequest {
    pub fn literal(text: impl Into<String>) -> Self {
        Self {
            literal: text.into(),
            documents: None,
            max_matches: 10_000,
            max_work: 10_000_000,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SearchTermination {
    Complete,
    MatchLimit,
    WorkLimit,
    Cancelled,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SearchResult {
    pub snapshot: VersionSnapshot,
    pub request: SearchRequest,
    /// Nonoverlapping exact matches, sorted by project-relative file then byte offset.
    pub matches: Vec<SourceSpan>,
    pub termination: SearchTermination,
    pub work_used: usize,
}

struct Budget<F> {
    remaining: usize,
    used: usize,
    cancelled: F,
}
impl<F: FnMut() -> bool> Budget<F> {
    fn comparison(&mut self) -> Result<(), SearchTermination> {
        if (self.cancelled)() {
            return Err(SearchTermination::Cancelled);
        }
        if self.remaining == 0 {
            return Err(SearchTermination::WorkLimit);
        }
        self.remaining -= 1;
        self.used += 1;
        Ok(())
    }
}

impl ProjectIndex {
    /// Case-sensitive UTF8 literal search; no regex, normalization or parser filtering.
    /// Cancellation is checked before each byte comparison; callback must not block.
    pub fn search_literal<F: FnMut() -> bool>(
        &self,
        snapshot: &VersionSnapshot,
        request: &SearchRequest,
        mut cancelled: F,
    ) -> Result<SearchResult, IndexError> {
        self.check(snapshot)?;
        if request.literal.is_empty()
            || request.literal.len() > MAX_SEARCH_QUERY_BYTES
            || request.max_matches > MAX_SEARCH_MATCHES
        {
            return Err(IndexError::InvalidSearchRequest);
        }
        if let Some(paths) = &request.documents {
            for path in paths {
                super::validate_path(path)?;
                if !self.documents.contains_key(path) {
                    return Err(IndexError::MissingDocument);
                }
            }
        }
        let mut result = SearchResult {
            snapshot: snapshot.clone(),
            request: request.clone(),
            matches: Vec::new(),
            termination: SearchTermination::Complete,
            work_used: 0,
        };
        if cancelled() {
            result.termination = SearchTermination::Cancelled;
            return Ok(result);
        }
        let docs: Vec<_> = self
            .documents
            .iter()
            .filter(|(path, doc)| {
                !doc.source.is_empty()
                    && request
                        .documents
                        .as_ref()
                        .is_none_or(|paths| paths.contains(*path))
            })
            .collect();
        if docs.is_empty() {
            return Ok(result);
        }
        if request.max_matches == 0 {
            result.termination = SearchTermination::MatchLimit;
            return Ok(result);
        }
        let mut budget = Budget {
            remaining: request.max_work,
            used: 0,
            cancelled,
        };
        let needle = request.literal.as_bytes();
        let mut prefix = vec![0; needle.len()];
        let outcome = (|| {
            // KMP construction and matching share one explicit comparison budget.
            let mut matched = 0;
            for at in 1..needle.len() {
                loop {
                    budget.comparison()?;
                    if needle[at] == needle[matched] {
                        matched += 1;
                        break;
                    }
                    if matched == 0 {
                        break;
                    }
                    matched = prefix[matched - 1];
                }
                prefix[at] = matched;
            }
            for (document_index, (path, doc)) in docs.iter().enumerate() {
                matched = 0;
                for (at, byte) in doc.source.bytes().enumerate() {
                    loop {
                        budget.comparison()?;
                        if byte == needle[matched] {
                            matched += 1;
                            break;
                        }
                        if matched == 0 {
                            break;
                        }
                        matched = prefix[matched - 1];
                    }
                    if matched == needle.len() {
                        result.matches.push(span(
                            path,
                            doc.revision,
                            at + 1 - needle.len(),
                            at + 1,
                        ));
                        matched = 0; // Deliberate nonoverlapping search and edit semantics.
                        if result.matches.len() == request.max_matches {
                            if document_index + 1 == docs.len() && at + 1 == doc.source.len() {
                                return Ok(());
                            }
                            return Err(SearchTermination::MatchLimit);
                        }
                    }
                }
            }
            Ok(())
        })();
        result.work_used = budget.used;
        if let Err(termination) = outcome {
            result.termination = termination;
        }
        Ok(result)
    }
}
