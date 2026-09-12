//! Export-only, versioned JSON proposals. Native clients retain approval/application authority.
use super::{
    CitationRenamePlan, IndexError, LiteralReplacementPlan, ProjectIndex, TextEdit, VersionSnapshot,
};

pub const MAX_REPLACEMENT_WIRE_BYTES: usize = 32 * 1024 * 1024;

struct Json {
    text: String,
    limit: usize,
}
impl Json {
    fn raw(&mut self, text: &str) -> Result<(), IndexError> {
        if text.len() > self.limit.saturating_sub(self.text.len()) {
            return Err(IndexError::SerializationLimit);
        }
        self.text.push_str(text);
        Ok(())
    }
    fn string(&mut self, text: &str) -> Result<(), IndexError> {
        self.raw("\"")?;
        for character in text.chars() {
            match character {
                '"' => self.raw("\\\"")?,
                '\\' => self.raw("\\\\")?,
                '\u{0000}'..='\u{001f}' => self.raw(&format!("\\u{:04x}", character as u32))?,
                _ => self.raw(character.encode_utf8(&mut [0; 4]))?,
            }
        }
        self.raw("\"")
    }
    fn decimal(&mut self, value: impl std::fmt::Display) -> Result<(), IndexError> {
        self.string(&value.to_string())
    }
    fn snapshot(&mut self, snapshot: &VersionSnapshot) -> Result<(), IndexError> {
        self.raw("{\"project_id\":")?;
        self.string(&snapshot.project_id)?;
        self.raw(",\"generation\":")?;
        self.decimal(snapshot.generation)?;
        self.raw(",\"documents\":[")?;
        for (at, (file, revision)) in snapshot.documents.iter().enumerate() {
            if at != 0 {
                self.raw(",")?;
            }
            self.raw("{\"file\":")?;
            self.string(file)?;
            self.raw(",\"revision\":")?;
            self.decimal(revision)?;
            self.raw("}")?;
        }
        self.raw("]}")
    }
    fn edits(&mut self, edits: &[TextEdit]) -> Result<(), IndexError> {
        self.raw("[")?;
        for (at, edit) in edits.iter().enumerate() {
            if at != 0 {
                self.raw(",")?;
            }
            self.raw("{\"file\":")?;
            self.string(&edit.source.file)?;
            self.raw(",\"revision\":")?;
            self.decimal(edit.source.revision)?;
            self.raw(",\"start_byte\":")?;
            self.decimal(edit.source.start_byte)?;
            self.raw(",\"end_byte\":")?;
            self.decimal(edit.source.end_byte)?;
            self.raw(",\"expected_text\":")?;
            self.string(&edit.expected_text)?;
            self.raw(",\"replacement\":")?;
            self.string(&edit.replacement)?;
            self.raw("}")?;
        }
        self.raw("]")
    }
}

impl ProjectIndex {
    /// Validates first, then exports bounded UTF8 JSON. Decimal strings preserve
    /// exact u64/usize values in clients whose JSON number type is floating point.
    /// No decoder, write-to-disk or automatic-application endpoint is provided.
    pub fn serialize_literal_replacement_plan(
        &self,
        plan: &LiteralReplacementPlan,
        max_bytes: usize,
    ) -> Result<String, IndexError> {
        self.validate_literal_replacement_plan(plan)?;
        let mut json = Json {
            text: String::new(),
            limit: max_bytes.min(MAX_REPLACEMENT_WIRE_BYTES),
        };
        json.raw("{\"schema\":\"flashtex.literal-replacement-plan.v1\",\"proposal_only\":true,\"requires_user_approval\":true,\"application_order\":\"reverse_byte_offset_per_document\",\"snapshot\":")?;
        json.snapshot(&plan.search.snapshot)?;
        json.raw(",\"search\":{\"literal\":")?;
        json.string(&plan.search.request.literal)?;
        json.raw(",\"documents\":")?;
        if let Some(paths) = &plan.search.request.documents {
            json.raw("[")?;
            for (at, file) in paths.iter().enumerate() {
                if at != 0 {
                    json.raw(",")?;
                }
                json.string(file)?;
            }
            json.raw("]")?;
        } else {
            json.raw("null")?;
        }
        json.raw(",\"max_matches\":")?;
        json.decimal(plan.search.request.max_matches)?;
        json.raw(",\"max_work\":")?;
        json.decimal(plan.search.request.max_work)?;
        json.raw(",\"work_used\":")?;
        json.decimal(plan.search.work_used)?;
        json.raw(",\"termination\":\"complete\"},\"replacement\":")?;
        json.string(&plan.replacement)?;
        json.raw(",\"edits\":")?;
        json.edits(&plan.edits)?;
        json.raw("}")?;
        Ok(json.text)
    }

    /// Citation rename uses the same proposal/snapshot/edit envelope, with an
    /// explicit kind and separate schema. It never presents a lexical rename as
    /// an exhaustive literal search or an engine-semantic rewrite.
    pub fn serialize_citation_rename_plan(
        &self,
        plan: &CitationRenamePlan,
        max_bytes: usize,
    ) -> Result<String, IndexError> {
        self.validate_citation_rename_plan(plan)?;
        let mut json = Json {
            text: String::new(),
            limit: max_bytes.min(MAX_REPLACEMENT_WIRE_BYTES),
        };
        json.raw("{\"schema\":\"flashtex.citation-rename-plan.v1\",\"kind\":\"citation_key_rename\",\"proposal_only\":true,\"requires_user_approval\":true,\"application_order\":\"reverse_byte_offset_per_document\",\"snapshot\":")?;
        json.snapshot(&plan.snapshot)?;
        json.raw(",\"rename\":{\"old_name\":")?;
        json.string(&plan.old_name)?;
        json.raw(",\"new_name\":")?;
        json.string(&plan.new_name)?;
        json.raw("},\"replacement\":")?;
        json.string(&plan.new_name)?;
        json.raw(",\"edits\":")?;
        json.edits(&plan.edits)?;
        json.raw("}")?;
        Ok(json.text)
    }
}
