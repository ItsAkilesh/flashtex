//! Lexical citation-key proposals; no bibliography engine, automatic edits or macro expansion.
use super::{
    Category, IndexError, ProjectIndex, SourceSpan, SymbolKind, TextEdit, VersionSnapshot,
    MAX_REPLACEMENT_PLAN_TEXT_BYTES, MAX_SEARCH_MATCHES,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CitationRenamePlan {
    pub snapshot: VersionSnapshot,
    pub old_name: String,
    pub new_name: String,
    pub edits: Vec<TextEdit>,
}

impl ProjectIndex {
    /// Requires exactly one well-formed declared bibliography record. Bibitems alone
    /// and duplicate definitions are refused; incomplete field macros do not alter keys.
    pub fn plan_citation_rename(
        &self,
        snapshot: &VersionSnapshot,
        old_name: &str,
        new_name: &str,
    ) -> Result<CitationRenamePlan, IndexError> {
        self.check(snapshot)?;
        if new_name.is_empty()
            || new_name.len() > 4096
            || new_name
                .chars()
                .any(|ch| ch.is_whitespace() || ch.is_control() || "{}()\\\"%=#@,".contains(ch))
        {
            return Err(IndexError::InvalidCitationKey);
        }
        let matches = self.occurrences(snapshot, Category::Citation, old_name)?;
        let definitions: Vec<_> = matches
            .iter()
            .filter(|s| s.kind == SymbolKind::CitationDefinition)
            .collect();
        if definitions.len() > 1 {
            return Err(IndexError::AmbiguousCitationDefinition);
        }
        let metadata = self.citation_metadata(snapshot, old_name)?;
        if definitions.len() != 1 || metadata.records.len() != 1 {
            return Err(IndexError::MissingBibliographyDefinition);
        }
        if !metadata.records[0].well_formed {
            return Err(IndexError::MalformedBibliographyDefinition);
        }
        if metadata.records[0].key.as_ref() != Some(&definitions[0].source) {
            return Err(IndexError::InvalidCitationRenamePlan);
        }
        if old_name != new_name
            && !self
                .occurrences(snapshot, Category::Citation, new_name)?
                .is_empty()
        {
            return Err(IndexError::RenameCollision {
                name: new_name.into(),
            });
        }
        let mut edits: Vec<TextEdit> = Vec::new();
        if old_name != new_name {
            let text_bytes = old_name
                .len()
                .checked_add(new_name.len())
                .and_then(|n| n.checked_mul(matches.len()));
            if matches.len() > MAX_SEARCH_MATCHES
                || text_bytes.is_none_or(|n| n > MAX_REPLACEMENT_PLAN_TEXT_BYTES)
            {
                return Err(IndexError::ReplacementPlanTooLarge);
            }
            for symbol in matches {
                if self.source_text(snapshot, &symbol.source)? != old_name {
                    return Err(IndexError::InvalidCitationRenamePlan);
                }
                if edits.last().is_some_and(|previous| {
                    previous.source.file == symbol.source.file
                        && previous.source.end_byte > symbol.source.start_byte
                }) {
                    return Err(IndexError::InvalidCitationRenamePlan);
                }
                edits.push(TextEdit {
                    source: symbol.source,
                    expected_text: old_name.into(),
                    replacement: new_name.into(),
                });
            }
        }
        Ok(CitationRenamePlan {
            snapshot: snapshot.clone(),
            old_name: old_name.into(),
            new_name: new_name.into(),
            edits,
        })
    }

    pub fn plan_citation_rename_at(
        &self,
        snapshot: &VersionSnapshot,
        source: &SourceSpan,
        new_name: &str,
    ) -> Result<CitationRenamePlan, IndexError> {
        self.source_text(snapshot, source)?;
        let symbol = self
            .symbols(snapshot)?
            .into_iter()
            .find(|s| s.source == *source && s.kind.category() == Category::Citation)
            .ok_or(IndexError::InvalidCitationRenamePlan)?;
        self.plan_citation_rename(snapshot, &symbol.name, new_name)
    }

    pub fn validate_citation_rename_plan(
        &self,
        plan: &CitationRenamePlan,
    ) -> Result<(), IndexError> {
        let expected = self.plan_citation_rename(&plan.snapshot, &plan.old_name, &plan.new_name)?;
        if expected != *plan {
            return Err(IndexError::InvalidCitationRenamePlan);
        }
        Ok(())
    }
}
