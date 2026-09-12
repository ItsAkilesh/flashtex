//! Lexical source navigation, not a TeX interpreter or macro expansion engine.
use std::collections::{BTreeMap, BTreeSet};
use std::time::Instant;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Category {
    Label,
    Citation,
    Command,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum SymbolKind {
    LabelDefinition,
    LabelReference,
    CitationDefinition,
    CitationReference,
    CommandDefinition,
    CommandUse,
}

impl SymbolKind {
    pub fn category(self) -> Category {
        match self {
            Self::LabelDefinition | Self::LabelReference => Category::Label,
            Self::CitationDefinition | Self::CitationReference => Category::Citation,
            Self::CommandDefinition | Self::CommandUse => Category::Command,
        }
    }

    pub fn is_definition(self) -> bool {
        matches!(
            self,
            Self::LabelDefinition | Self::CitationDefinition | Self::CommandDefinition
        )
    }
}

/// Name-only range: excludes a command's leading backslash and argument braces.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct SourceSpan {
    pub file: String,
    pub revision: u64,
    pub start_byte: usize,
    pub end_byte: usize,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Symbol {
    pub name: String,
    pub kind: SymbolKind,
    pub source: SourceSpan,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Diagnostic {
    pub message: String,
    pub source: SourceSpan,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UpdateSummary {
    pub symbol_count: usize,
    pub diagnostics: Vec<Diagnostic>,
    pub metrics: ReindexMetrics,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReindexMetrics {
    pub generation: u64,
    /// Input bytes of changed documents, not a parser instruction/byte-visit count.
    pub input_bytes_reindexed: usize,
    pub documents_reindexed: usize,
    pub document_indexes_reused: usize,
    pub definition_availability_changes: usize,
    pub reference_documents_rechecked: usize,
    pub lexical_elapsed_nanos: u128,
    pub total_elapsed_nanos: u128,
}

/// No lexical project definition exists. This is not a TeX compile error claim.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UnresolvedReference {
    pub category: Category,
    pub name: String,
    pub source: SourceSpan,
}

type SymbolKey = (Category, String);
type DependencyMap = BTreeMap<SymbolKey, BTreeSet<String>>;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum IndexError {
    InvalidProject,
    InvalidPath,
    StaleDocument {
        current_revision: u64,
        proposed_revision: u64,
    },
    StaleSnapshot,
    StaleSourceRange {
        current_revision: u64,
        source_revision: u64,
    },
    WrongProject,
    MissingDocument,
    InvalidOffset,
    GenerationExhausted,
    InvalidLabelName,
    MissingLabelDefinition,
    RenameCollision {
        name: String,
    },
    InvalidRenamePlan,
}

impl std::fmt::Display for IndexError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "project index: {self:?}")
    }
}

impl std::error::Error for IndexError {}

/// Capture after the last successful update; every query checks the entire view.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VersionSnapshot {
    pub project_id: String,
    pub generation: u64,
    pub documents: BTreeMap<String, u64>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Completion {
    pub name: String,
    pub category: Category,
    pub definitions: Vec<SourceSpan>,
    pub occurrences: Vec<SourceSpan>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Navigation {
    pub origin: Symbol,
    /// Multiple lexical definitions remain visible; no TeX scope winner is guessed.
    pub definitions: Vec<Symbol>,
}

/// A proposed source edit, never applied by this library.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TextEdit {
    pub source: SourceSpan,
    pub expected_text: String,
    pub replacement: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RenamePlan {
    pub snapshot: VersionSnapshot,
    pub old_name: String,
    pub new_name: String,
    /// Canonical file/start-byte order; callers apply within each file in reverse.
    pub edits: Vec<TextEdit>,
}

struct Document {
    revision: u64,
    source: String,
    symbols: Vec<Symbol>,
    diagnostics: Vec<Diagnostic>,
}

pub struct ProjectIndex {
    project_id: String,
    generation: u64,
    documents: BTreeMap<String, Document>,
    /// Tombstones prevent an old document replacement from resurrecting a deletion.
    last_revisions: BTreeMap<String, u64>,
    definitions: DependencyMap,
    readers: DependencyMap,
    unresolved: BTreeMap<String, Vec<UnresolvedReference>>,
    last_metrics: Option<ReindexMetrics>,
}

impl ProjectIndex {
    pub fn new(project_id: impl Into<String>) -> Result<Self, IndexError> {
        let project_id = project_id.into();
        if project_id.trim().is_empty() {
            return Err(IndexError::InvalidProject);
        }
        Ok(Self {
            project_id,
            generation: 0,
            documents: BTreeMap::new(),
            last_revisions: BTreeMap::new(),
            definitions: BTreeMap::new(),
            readers: BTreeMap::new(),
            unresolved: BTreeMap::new(),
            last_metrics: None,
        })
    }

    pub fn snapshot(&self) -> VersionSnapshot {
        VersionSnapshot {
            project_id: self.project_id.clone(),
            generation: self.generation,
            documents: self
                .documents
                .iter()
                .map(|(file, doc)| (file.clone(), doc.revision))
                .collect(),
        }
    }

    fn check(&self, snapshot: &VersionSnapshot) -> Result<(), IndexError> {
        if snapshot.project_id != self.project_id {
            return Err(IndexError::WrongProject);
        }
        if *snapshot != self.snapshot() {
            return Err(IndexError::StaleSnapshot);
        }
        Ok(())
    }

    fn check_update(&self, file: &str, revision: u64) -> Result<u64, IndexError> {
        validate_path(file)?;
        if let Some(current) = self.last_revisions.get(file) {
            if revision <= *current {
                return Err(IndexError::StaleDocument {
                    current_revision: *current,
                    proposed_revision: revision,
                });
            }
        }
        self.generation
            .checked_add(1)
            .ok_or(IndexError::GenerationExhausted)
    }

    /// Atomically replaces this document's lexical symbols. Revisions must increase.
    pub fn replace_document(
        &mut self,
        file: &str,
        revision: u64,
        source: &str,
    ) -> Result<UpdateSummary, IndexError> {
        let started = Instant::now();
        let generation = self.check_update(file, revision)?;
        let lexical_started = Instant::now();
        let (symbols, diagnostics) = scan(file, revision, source);
        let lexical_elapsed_nanos = lexical_started.elapsed().as_nanos();
        let symbol_count = symbols.len();
        let diagnostic_copy = diagnostics.clone();
        let (definition_availability_changes, reference_documents_rechecked) = self
            .commit_document(
                file,
                Some(Document {
                    revision,
                    source: source.to_owned(),
                    symbols,
                    diagnostics,
                }),
            );
        self.last_revisions.insert(file.to_owned(), revision);
        self.generation = generation;
        let metrics = ReindexMetrics {
            generation,
            input_bytes_reindexed: source.len(),
            documents_reindexed: 1,
            document_indexes_reused: self.documents.len() - 1,
            definition_availability_changes,
            reference_documents_rechecked,
            lexical_elapsed_nanos,
            total_elapsed_nanos: started.elapsed().as_nanos(),
        };
        self.last_metrics = Some(metrics.clone());
        Ok(UpdateSummary {
            symbol_count,
            diagnostics: diagnostic_copy,
            metrics,
        })
    }

    pub fn remove_document(&mut self, file: &str, revision: u64) -> Result<(), IndexError> {
        let started = Instant::now();
        let generation = self.check_update(file, revision)?;
        if !self.documents.contains_key(file) {
            return Err(IndexError::MissingDocument);
        }
        let (definition_availability_changes, reference_documents_rechecked) =
            self.commit_document(file, None);
        self.last_revisions.insert(file.to_owned(), revision);
        self.generation = generation;
        self.last_metrics = Some(ReindexMetrics {
            generation,
            input_bytes_reindexed: 0,
            documents_reindexed: 0,
            document_indexes_reused: self.documents.len(),
            definition_availability_changes,
            reference_documents_rechecked,
            lexical_elapsed_nanos: 0,
            total_elapsed_nanos: started.elapsed().as_nanos(),
        });
        Ok(())
    }

    /// Update dependency memberships; only changed definition availability wakes readers.
    fn commit_document(&mut self, file: &str, replacement: Option<Document>) -> (usize, usize) {
        let (old_definitions, old_readers) = relation_keys(self.documents.get(file));
        let (new_definitions, new_readers) = relation_keys(replacement.as_ref());
        let availability: Vec<_> = old_definitions
            .union(&new_definitions)
            .map(|key| (key.clone(), self.definitions.contains_key(key)))
            .collect();
        remove_memberships(&mut self.definitions, file, &old_definitions);
        remove_memberships(&mut self.readers, file, &old_readers);
        for key in new_definitions {
            self.definitions
                .entry(key)
                .or_default()
                .insert(file.to_owned());
        }
        for key in new_readers {
            self.readers.entry(key).or_default().insert(file.to_owned());
        }
        if let Some(document) = replacement {
            self.documents.insert(file.to_owned(), document);
        } else {
            self.documents.remove(file);
            self.unresolved.remove(file);
        }
        let mut affected = BTreeSet::new();
        if self.documents.contains_key(file) {
            affected.insert(file.to_owned());
        }
        let mut changed = 0;
        for (key, before) in availability {
            if self.definitions.contains_key(&key) != before {
                changed += 1;
                if let Some(readers) = self.readers.get(&key) {
                    affected.extend(readers.iter().cloned());
                }
            }
        }
        for path in &affected {
            let document = &self.documents[path];
            let unresolved = document
                .symbols
                .iter()
                .filter(|symbol| {
                    matches!(
                        symbol.kind,
                        SymbolKind::LabelReference | SymbolKind::CitationReference
                    ) && !self
                        .definitions
                        .contains_key(&(symbol.kind.category(), symbol.name.clone()))
                })
                .map(|symbol| UnresolvedReference {
                    category: symbol.kind.category(),
                    name: symbol.name.clone(),
                    source: symbol.source.clone(),
                })
                .collect();
            self.unresolved.insert(path.clone(), unresolved);
        }
        (changed, affected.len())
    }

    pub fn unresolved_references(
        &self,
        snapshot: &VersionSnapshot,
    ) -> Result<Vec<UnresolvedReference>, IndexError> {
        self.check(snapshot)?;
        Ok(self.unresolved.values().flatten().cloned().collect())
    }

    pub fn last_reindex_metrics(
        &self,
        snapshot: &VersionSnapshot,
    ) -> Result<Option<ReindexMetrics>, IndexError> {
        self.check(snapshot)?;
        Ok(self.last_metrics.clone())
    }

    pub fn symbols(&self, snapshot: &VersionSnapshot) -> Result<Vec<Symbol>, IndexError> {
        self.check(snapshot)?;
        Ok(self
            .documents
            .values()
            .flat_map(|doc| doc.symbols.clone())
            .collect())
    }

    pub fn diagnostics(&self, snapshot: &VersionSnapshot) -> Result<Vec<Diagnostic>, IndexError> {
        self.check(snapshot)?;
        Ok(self
            .documents
            .values()
            .flat_map(|doc| doc.diagnostics.clone())
            .collect())
    }

    /// Revalidate a retained navigation result before selecting source in an editor.
    pub fn source_text(
        &self,
        snapshot: &VersionSnapshot,
        source: &SourceSpan,
    ) -> Result<&str, IndexError> {
        self.check(snapshot)?;
        validate_path(&source.file)?;
        let document = self
            .documents
            .get(&source.file)
            .ok_or(IndexError::MissingDocument)?;
        if document.revision != source.revision {
            return Err(IndexError::StaleSourceRange {
                current_revision: document.revision,
                source_revision: source.revision,
            });
        }
        document
            .source
            .get(source.start_byte..source.end_byte)
            .ok_or(IndexError::InvalidOffset)
    }

    pub fn occurrences(
        &self,
        snapshot: &VersionSnapshot,
        category: Category,
        name: &str,
    ) -> Result<Vec<Symbol>, IndexError> {
        Ok(self
            .symbols(snapshot)?
            .into_iter()
            .filter(|symbol| symbol.kind.category() == category && symbol.name == name)
            .collect())
    }

    pub fn definitions(
        &self,
        snapshot: &VersionSnapshot,
        category: Category,
        name: &str,
    ) -> Result<Vec<Symbol>, IndexError> {
        Ok(self
            .occurrences(snapshot, category, name)?
            .into_iter()
            .filter(|symbol| symbol.kind.is_definition())
            .collect())
    }

    pub fn navigate(
        &self,
        snapshot: &VersionSnapshot,
        file: &str,
        byte_offset: usize,
    ) -> Result<Option<Navigation>, IndexError> {
        self.check(snapshot)?;
        validate_path(file)?;
        let document = self
            .documents
            .get(file)
            .ok_or(IndexError::MissingDocument)?;
        if byte_offset > document.source.len() || !document.source.is_char_boundary(byte_offset) {
            return Err(IndexError::InvalidOffset);
        }
        let Some(origin) = document.symbols.iter().find(|symbol| {
            symbol.source.start_byte <= byte_offset && byte_offset < symbol.source.end_byte
        }) else {
            return Ok(None);
        };
        Ok(Some(Navigation {
            origin: origin.clone(),
            definitions: self.definitions(snapshot, origin.kind.category(), &origin.name)?,
        }))
    }

    /// Rename every lexical definition/reference of a label across this snapshot.
    /// Collisions with definitions OR unresolved references are rejected.
    pub fn plan_label_rename(
        &self,
        snapshot: &VersionSnapshot,
        old_name: &str,
        new_name: &str,
    ) -> Result<RenamePlan, IndexError> {
        self.check(snapshot)?;
        if new_name.is_empty()
            || new_name
                .chars()
                .any(|ch| ch.is_whitespace() || ch.is_control() || "\\{}%,".contains(ch))
        {
            return Err(IndexError::InvalidLabelName);
        }
        let matches = self.occurrences(snapshot, Category::Label, old_name)?;
        if !matches
            .iter()
            .any(|symbol| symbol.kind == SymbolKind::LabelDefinition)
        {
            return Err(IndexError::MissingLabelDefinition);
        }
        if old_name != new_name
            && !self
                .occurrences(snapshot, Category::Label, new_name)?
                .is_empty()
        {
            return Err(IndexError::RenameCollision {
                name: new_name.to_owned(),
            });
        }
        let mut edits: Vec<TextEdit> = Vec::new();
        if old_name != new_name {
            for symbol in matches {
                if self.source_text(snapshot, &symbol.source)? != old_name {
                    return Err(IndexError::InvalidRenamePlan);
                }
                if let Some(previous) = edits.last() {
                    if previous.source.file == symbol.source.file
                        && previous.source.end_byte > symbol.source.start_byte
                    {
                        return Err(IndexError::InvalidRenamePlan);
                    }
                }
                edits.push(TextEdit {
                    source: symbol.source,
                    expected_text: old_name.to_owned(),
                    replacement: new_name.to_owned(),
                });
            }
        }
        Ok(RenamePlan {
            snapshot: snapshot.clone(),
            old_name: old_name.to_owned(),
            new_name: new_name.to_owned(),
            edits,
        })
    }

    /// A retained selection must identify one complete current label name span.
    pub fn plan_label_rename_at(
        &self,
        snapshot: &VersionSnapshot,
        source: &SourceSpan,
        new_name: &str,
    ) -> Result<RenamePlan, IndexError> {
        self.source_text(snapshot, source)?;
        let symbol = self
            .symbols(snapshot)?
            .into_iter()
            .find(|symbol| symbol.source == *source && symbol.kind.category() == Category::Label)
            .ok_or(IndexError::InvalidRenamePlan)?;
        self.plan_label_rename(snapshot, &symbol.name, new_name)
    }

    /// Rechecks all ranges/revisions and the complete deterministic edit set.
    /// Missing, duplicated, reordered, overlapping or modified edits are rejected.
    pub fn validate_rename_plan(&self, plan: &RenamePlan) -> Result<(), IndexError> {
        let expected = self.plan_label_rename(&plan.snapshot, &plan.old_name, &plan.new_name)?;
        if expected != *plan {
            return Err(IndexError::InvalidRenamePlan);
        }
        Ok(())
    }

    /// Prefixes are literal names, without a leading backslash. Case is preserved.
    pub fn complete(
        &self,
        snapshot: &VersionSnapshot,
        category: Category,
        prefix: &str,
        limit: usize,
    ) -> Result<Vec<Completion>, IndexError> {
        let mut candidates: BTreeMap<String, Completion> = BTreeMap::new();
        for symbol in self.symbols(snapshot)? {
            if symbol.kind.category() != category || !symbol.name.starts_with(prefix) {
                continue;
            }
            let candidate = candidates
                .entry(symbol.name.clone())
                .or_insert_with(|| Completion {
                    name: symbol.name.clone(),
                    category,
                    definitions: Vec::new(),
                    occurrences: Vec::new(),
                });
            if symbol.kind.is_definition() {
                candidate.definitions.push(symbol.source.clone());
            }
            candidate.occurrences.push(symbol.source);
        }
        Ok(candidates.into_values().take(limit).collect())
    }
}

fn relation_keys(document: Option<&Document>) -> (BTreeSet<SymbolKey>, BTreeSet<SymbolKey>) {
    let mut definitions = BTreeSet::new();
    let mut readers = BTreeSet::new();
    if let Some(document) = document {
        for symbol in &document.symbols {
            if symbol.kind.category() == Category::Command {
                continue;
            }
            let key = (symbol.kind.category(), symbol.name.clone());
            if symbol.kind.is_definition() {
                definitions.insert(key);
            } else {
                readers.insert(key);
            }
        }
    }
    (definitions, readers)
}

fn remove_memberships(map: &mut DependencyMap, file: &str, keys: &BTreeSet<SymbolKey>) {
    for key in keys {
        if let Some(files) = map.get_mut(key) {
            files.remove(file);
            if files.is_empty() {
                map.remove(key);
            }
        }
    }
}

fn validate_path(path: &str) -> Result<(), IndexError> {
    if path.contains(['\\', ':', '\0'])
        || path
            .split('/')
            .any(|part| part.is_empty() || part == "." || part == "..")
    {
        return Err(IndexError::InvalidPath);
    }
    Ok(())
}

fn next_char(source: &str, at: usize) -> usize {
    at + source[at..]
        .chars()
        .next()
        .expect("caller checks source end")
        .len_utf8()
}

fn trivia(source: &str, mut at: usize) -> usize {
    while at < source.len() {
        if source.as_bytes()[at] == b'%' {
            at = source[at..]
                .find('\n')
                .map_or(source.len(), |offset| at + offset + 1);
        } else if source[at..].chars().next().unwrap().is_whitespace() {
            at = next_char(source, at);
        } else {
            break;
        }
    }
    at
}

/// Command name byte range (excluding backslash), plus token end.
fn command(source: &str, at: usize) -> Option<(usize, usize)> {
    if source.as_bytes().get(at) != Some(&b'\\') || at + 1 == source.len() {
        return None;
    }
    let start = at + 1;
    let mut end = start;
    while source
        .as_bytes()
        .get(end)
        .is_some_and(|b| b.is_ascii_alphabetic() || *b == b'@')
    {
        end += 1;
    }
    if end == start {
        end = next_char(source, start);
    }
    Some((start, end))
}

/// Balanced literal group; escaped delimiters and comments do not close it.
fn group(source: &str, at: usize, open: u8, close: u8) -> Option<(usize, usize, usize)> {
    if source.as_bytes().get(at) != Some(&open) {
        return None;
    }
    let mut stack = vec![close];
    let mut cursor = at + 1;
    while cursor < source.len() {
        let byte = source.as_bytes()[cursor];
        if byte == b'\\' {
            cursor = command(source, cursor).map_or(source.len(), |(_, end)| end);
            continue;
        }
        if byte == b'%' {
            cursor = trivia(source, cursor);
            continue;
        }
        if byte == *stack.last()? {
            stack.pop();
            if stack.is_empty() {
                return Some((at + 1, cursor, cursor + 1));
            }
        } else if byte == b'{' {
            stack.push(b'}');
        } else if byte == open && open != b'{' && stack.last() == Some(&close) {
            stack.push(close);
        }
        cursor = next_char(source, cursor);
    }
    None
}

fn span(file: &str, revision: u64, start: usize, end: usize) -> SourceSpan {
    SourceSpan {
        file: file.to_owned(),
        revision,
        start_byte: start,
        end_byte: end,
    }
}

fn push_name(
    symbols: &mut Vec<Symbol>,
    file: &str,
    revision: u64,
    source: &str,
    start: usize,
    end: usize,
    kind: SymbolKind,
) {
    let raw = &source[start..end];
    let name = raw.trim();
    if name.is_empty() || name.contains(['\\', '{', '}', '%']) {
        return;
    }
    let start = start + raw.len() - raw.trim_start().len();
    symbols.push(Symbol {
        name: name.to_owned(),
        kind,
        source: span(file, revision, start, start + name.len()),
    });
}

fn scan(file: &str, revision: u64, source: &str) -> (Vec<Symbol>, Vec<Diagnostic>) {
    let mut symbols = Vec::new();
    let mut diagnostics = Vec::new();
    let mut cursor = 0;
    while cursor < source.len() {
        if source.as_bytes()[cursor] == b'%' {
            cursor = trivia(source, cursor);
            continue;
        }
        let Some((name_start, token_end)) = command(source, cursor) else {
            cursor = next_char(source, cursor);
            continue;
        };
        let name = &source[name_start..token_end];
        let token_start = cursor;
        cursor = token_end;
        symbols.push(Symbol {
            name: name.to_owned(),
            kind: SymbolKind::CommandUse,
            source: span(file, revision, name_start, token_end),
        });
        if name == "verb" {
            if source.as_bytes().get(cursor) == Some(&b'*') {
                cursor += 1;
            }
            if cursor < source.len() {
                let end = next_char(source, cursor);
                let delimiter = &source[cursor..end];
                let line_end = source[end..]
                    .find('\n')
                    .map_or(source.len(), |offset| end + offset);
                if !delimiter.chars().next().unwrap().is_whitespace() {
                    if let Some(offset) = source[end..line_end].find(delimiter) {
                        cursor = end + offset + delimiter.len();
                        continue;
                    }
                }
                diagnostics.push(Diagnostic {
                    message: "Unclosed or invalid verb delimiter; resumed after the line".into(),
                    source: span(file, revision, token_start, token_end),
                });
                cursor = if delimiter == "\n" { end } else { line_end };
            }
            continue;
        }
        if [
            "newcommand",
            "renewcommand",
            "providecommand",
            "DeclareRobustCommand",
            "def",
            "gdef",
            "edef",
            "xdef",
        ]
        .contains(&name)
        {
            let mut target = trivia(source, cursor);
            if source.as_bytes().get(target) == Some(&b'*') {
                target = trivia(source, target + 1);
            }
            let braced = group(source, target, b'{', b'}');
            let command_start = braced.map_or(target, |(start, _, _)| trivia(source, start));
            if let Some((start, end)) = command(source, command_start) {
                if braced.is_none_or(|(_, inside_end, _)| trivia(source, end) == inside_end) {
                    symbols.push(Symbol {
                        name: source[start..end].to_owned(),
                        kind: SymbolKind::CommandDefinition,
                        source: span(file, revision, start, end),
                    });
                    cursor = braced.map_or(end, |(_, _, end)| end);
                }
            }
            continue;
        }
        let kind = match name {
            "label" => Some(SymbolKind::LabelDefinition),
            "ref" | "eqref" | "pageref" | "autoref" | "cref" | "Cref" | "nameref" => {
                Some(SymbolKind::LabelReference)
            }
            "bibitem" => Some(SymbolKind::CitationDefinition),
            "cite" | "citep" | "citet" | "citeauthor" | "citeyear" | "parencite" | "textcite"
            | "autocite" | "nocite" => Some(SymbolKind::CitationReference),
            _ => None,
        };
        if kind.is_none() && name != "begin" {
            continue;
        }
        let mut argument = trivia(source, cursor);
        if source.as_bytes().get(argument) == Some(&b'*') {
            argument = trivia(source, argument + 1);
        }
        while source.as_bytes().get(argument) == Some(&b'[') {
            let Some((_, _, end)) = group(source, argument, b'[', b']') else {
                break;
            };
            argument = trivia(source, end);
        }
        let Some((start, end, after)) = group(source, argument, b'{', b'}') else {
            if kind.is_some() {
                diagnostics.push(Diagnostic {
                    message: "Missing or unclosed literal symbol argument".into(),
                    source: span(file, revision, token_start, token_end),
                });
            }
            continue;
        };
        if name == "begin" {
            let environment = source[start..end].trim();
            if environment == "verbatim" || environment == "verbatim*" {
                let marker = format!("\\end{{{environment}}}");
                cursor = source[after..]
                    .find(&marker)
                    .map_or(source.len(), |offset| after + offset + marker.len());
            }
            continue;
        }
        let kind = kind.unwrap();
        let before = symbols.len();
        if kind == SymbolKind::CitationReference || name == "cref" || name == "Cref" {
            let mut offset = start;
            for item in source[start..end].split(',') {
                if !(name == "nocite" && item.trim() == "*") {
                    push_name(
                        &mut symbols,
                        file,
                        revision,
                        source,
                        offset,
                        offset + item.len(),
                        kind,
                    );
                }
                offset += item.len() + 1;
            }
        } else {
            push_name(&mut symbols, file, revision, source, start, end, kind);
        }
        if before == symbols.len() && !(name == "nocite" && source[start..end].trim() == "*") {
            diagnostics.push(Diagnostic {
                message: "Empty or dynamic symbol name is not indexed".into(),
                source: span(file, revision, start, end),
            });
        }
        cursor = after;
    }
    symbols.sort_by_key(|symbol| (symbol.source.start_byte, symbol.kind));
    (symbols, diagnostics)
}
