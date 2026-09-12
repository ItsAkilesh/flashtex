//! Local metadata inspection; no bibliography formatting or engine evaluation.
use super::{next_char, span, trivia, Diagnostic, ProjectIndex, SourceSpan, SymbolKind};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ValuePartKind {
    Literal,
    Number,
    Macro,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ValuePart {
    pub kind: ValuePartKind,
    pub text: String,
    pub source: SourceSpan,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BibliographyField {
    pub name: String,
    pub name_source: SourceSpan,
    pub source: SourceSpan,
    pub parts: Vec<ValuePart>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BibliographyRecord {
    pub entry_type: String,
    pub key: Option<SourceSpan>,
    pub source: SourceSpan,
    pub fields: Vec<BibliographyField>,
    pub diagnostics: Vec<Diagnostic>,
    pub well_formed: bool,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MetadataStatus {
    Missing,
    NoBibliographyRecord,
    Ambiguous,
    Malformed,
    Incomplete,
    Resolved,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MetadataField {
    pub source: SourceSpan,
    pub value: Option<String>,
    pub issues: Vec<String>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CitationMetadata {
    pub key: String,
    pub status: MetadataStatus,
    pub records: Vec<BibliographyRecord>,
    pub fields: BTreeMap<String, MetadataField>,
    /// Includes missing and transitively used macros, normalized to ASCII lowercase.
    pub macro_dependencies: BTreeSet<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct MetadataCacheMetrics {
    pub keys_recomputed: usize,
    pub keys_reused: usize,
    pub keys_removed: usize,
}

impl CitationMetadata {
    pub(super) fn missing(key: &str) -> Self {
        Self {
            key: key.into(),
            status: MetadataStatus::Missing,
            records: Vec::new(),
            fields: BTreeMap::new(),
            macro_dependencies: BTreeSet::new(),
        }
    }
    /// Exact expression span for local navigation; no inferred author/title/year formatting.
    pub fn field(&self, name: &str) -> Option<&MetadataField> {
        self.fields.get(&name.to_ascii_lowercase())
    }
}
pub(super) struct RecordBounds {
    pub kind: String,
    pub key: Option<SourceSpan>,
    pub start: usize,
    pub body_start: usize,
    pub body_end: usize,
    pub end: usize,
    pub closed: bool,
}

struct Parser<'a> {
    file: &'a str,
    revision: u64,
    source: &'a str,
    at: usize,
    end: usize,
}
impl Parser<'_> {
    fn space(&mut self) {
        self.at = trivia(&self.source[..self.end], self.at);
    }
    fn byte(&self) -> Option<u8> {
        (self.at < self.end).then(|| self.source.as_bytes()[self.at])
    }
    fn ident(&mut self) -> (usize, usize) {
        let start = self.at;
        while self
            .byte()
            .is_some_and(|b| b.is_ascii_alphanumeric() || b"_:-.".contains(&b))
        {
            self.at += 1;
        }
        (start, self.at)
    }
    fn atom(&mut self) -> Result<ValuePart, &'static str> {
        self.space();
        let start = self.at;
        let opening = self.byte().ok_or("Missing value")?;
        let (kind, text_start, text_end) = if opening == b'{' || opening == b'"' {
            self.at += 1;
            let content = self.at;
            let mut depth = usize::from(opening == b'{');
            loop {
                let b = self.byte().ok_or("Unclosed value")?;
                if self.at - start > 1024 * 1024 || depth > 128 {
                    return Err("Value exceeds lexical bounds");
                }
                if b == b'\\' {
                    self.at += 1;
                    if self.at < self.end {
                        self.at = next_char(self.source, self.at);
                    }
                    continue;
                }
                if (opening == b'"' && b == b'"' && depth == 0)
                    || (opening == b'{' && b == b'}' && depth == 1)
                {
                    let end = self.at;
                    self.at += 1;
                    break (ValuePartKind::Literal, content, end);
                }
                if b == b'{' {
                    depth += 1;
                }
                if b == b'}' {
                    if depth == 0 {
                        return Err("Unbalanced value brace");
                    }
                    depth -= 1;
                }
                self.at = next_char(self.source, self.at);
            }
        } else {
            let (_, end) = self.ident();
            if end == start {
                return Err("Invalid value atom");
            }
            let kind = if self.source[start..end].bytes().all(|b| b.is_ascii_digit()) {
                ValuePartKind::Number
            } else {
                ValuePartKind::Macro
            };
            (kind, start, end)
        };
        Ok(ValuePart {
            kind,
            text: self.source[text_start..text_end].into(),
            source: span(self.file, self.revision, start, self.at),
        })
    }
    fn field(&mut self) -> Result<BibliographyField, &'static str> {
        self.space();
        let (start, end) = self.ident();
        if start == end {
            return Err("Expected field name");
        }
        self.space();
        if self.byte() != Some(b'=') {
            return Err("Expected equals after field name");
        }
        self.at += 1;
        self.space();
        let value_start = self.at;
        let mut parts = Vec::new();
        loop {
            if parts.len() == 256 {
                return Err("Too many concatenated atoms");
            }
            parts.push(self.atom()?);
            let value_end = self.at;
            self.space();
            if self.byte() != Some(b'#') {
                if self.byte().is_some_and(|b| b != b',') {
                    return Err("Expected comma after value");
                }
                return Ok(BibliographyField {
                    name: self.source[start..end].to_ascii_lowercase(),
                    name_source: span(self.file, self.revision, start, end),
                    source: span(self.file, self.revision, value_start, value_end),
                    parts,
                });
            }
            self.at += 1;
        }
    }
}

pub(super) fn parse(
    file: &str,
    revision: u64,
    source: &str,
    bounds: RecordBounds,
) -> BibliographyRecord {
    let mut record = BibliographyRecord {
        entry_type: bounds.kind,
        key: bounds.key,
        source: span(file, revision, bounds.start, bounds.end),
        fields: Vec::new(),
        diagnostics: Vec::new(),
        well_formed: bounds.closed,
    };
    let mut parser = Parser {
        file,
        revision,
        source,
        at: bounds.body_start,
        end: bounds.body_end,
    };
    let mut names = BTreeSet::new();
    while parser.at < parser.end {
        parser.space();
        if parser.at == parser.end {
            break;
        }
        let start = parser.at;
        let result = if record.fields.len() >= 128 {
            Err("Too many fields")
        } else {
            parser.field()
        };
        match result {
            Ok(field) => {
                if !names.insert(field.name.clone()) {
                    record.diagnostics.push(Diagnostic {
                        message: "Duplicate field name".into(),
                        source: field.name_source.clone(),
                    });
                    record.well_formed = false;
                }
                record.fields.push(field);
                if parser.byte() == Some(b',') {
                    parser.at += 1;
                }
            }
            Err(message) => {
                record.well_formed = false;
                // Preserve a malformed declaration's name so it cannot disappear
                // and leave another declaration falsely unique.
                if record.entry_type == "string" {
                    let mut name_parser = Parser {
                        file,
                        revision,
                        source,
                        at: start,
                        end: bounds.body_end,
                    };
                    let (a, b) = name_parser.ident();
                    if b > a {
                        record.fields.push(BibliographyField {
                            name: source[a..b].to_ascii_lowercase(),
                            name_source: span(file, revision, a, b),
                            source: span(file, revision, b, b),
                            parts: Vec::new(),
                        });
                    }
                }
                record.diagnostics.push(Diagnostic {
                    message: message.into(),
                    source: span(file, revision, start, parser.at.max(start)),
                });
                // Conservative recovery: the outer scanner resumes the next safely delimited entry.
                break;
            }
        }
    }
    if record.entry_type == "string" && record.fields.len() != 1 {
        record.well_formed = false;
    }
    record
}

struct Evaluation<'a> {
    macros: BTreeMap<String, Vec<(&'a BibliographyRecord, &'a BibliographyField)>>,
    dependencies: BTreeSet<String>,
    visits: usize,
}
impl Evaluation<'_> {
    fn value(
        &mut self,
        field: &BibliographyField,
        stack: &mut BTreeSet<String>,
    ) -> Result<String, String> {
        let mut text = String::new();
        for part in &field.parts {
            self.visits += 1;
            if self.visits > 4096 {
                return Err("Macro expansion work bound exceeded".into());
            }
            let value = if part.kind != ValuePartKind::Macro {
                part.text.clone()
            } else {
                let name = part.text.to_ascii_lowercase();
                self.dependencies.insert(name.clone());
                if stack.len() >= 32 || !stack.insert(name.clone()) {
                    return Err(format!("Cyclic or over-depth macro: {name}"));
                }
                let entries = self
                    .macros
                    .get(&name)
                    .ok_or_else(|| format!("Missing macro: {name}"))?;
                if entries.len() != 1 {
                    return Err(format!("Ambiguous macro: {name}"));
                }
                let (record, field) = entries[0];
                if !record.well_formed {
                    return Err(format!("Malformed macro: {name}"));
                }
                let value = self.value(field, stack)?;
                stack.remove(&name);
                value
            };
            if text.len() + value.len() > 256 * 1024 {
                return Err("Expanded value byte bound exceeded".into());
            }
            text.push_str(&value);
        }
        Ok(text)
    }
}

pub(super) fn resolve(index: &ProjectIndex, key: &str) -> CitationMetadata {
    let mut result = CitationMetadata {
        key: key.into(),
        status: MetadataStatus::Missing,
        records: Vec::new(),
        fields: BTreeMap::new(),
        macro_dependencies: BTreeSet::new(),
    };
    let mut count = 0;
    let mut evaluation = Evaluation {
        macros: BTreeMap::new(),
        dependencies: BTreeSet::new(),
        visits: 0,
    };
    for doc in index.documents.values() {
        count += doc
            .symbols
            .iter()
            .filter(|s| s.kind == SymbolKind::CitationDefinition && s.name == key)
            .count();
        for record in &doc.records {
            if record
                .key
                .as_ref()
                .is_some_and(|s| &doc.source[s.start_byte..s.end_byte] == key)
            {
                result.records.push(record.clone());
            }
            if record.entry_type == "string" {
                for field in &record.fields {
                    evaluation
                        .macros
                        .entry(field.name.clone())
                        .or_default()
                        .push((record, field));
                }
            }
        }
    }
    result.status = if count == 0 {
        MetadataStatus::Missing
    } else if count > 1 {
        MetadataStatus::Ambiguous
    } else if result.records.is_empty() {
        MetadataStatus::NoBibliographyRecord
    } else if !result.records[0].well_formed {
        MetadataStatus::Malformed
    } else {
        MetadataStatus::Resolved
    };
    if result.status == MetadataStatus::Resolved {
        for field in &result.records[0].fields {
            evaluation.visits = 0;
            let (value, issues) = match evaluation.value(field, &mut BTreeSet::new()) {
                Ok(value) => (Some(value), Vec::new()),
                Err(issue) => {
                    result.status = MetadataStatus::Incomplete;
                    (None, vec![issue])
                }
            };
            result.fields.insert(
                field.name.clone(),
                MetadataField {
                    source: field.source.clone(),
                    value,
                    issues,
                },
            );
        }
    }
    result.macro_dependencies = evaluation.dependencies;
    result
}
