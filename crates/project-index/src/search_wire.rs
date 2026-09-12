//! Export-only, versioned JSON proposals. Native clients retain approval/application authority.
use super::{IndexError, LiteralReplacementPlan, ProjectIndex};

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
        json.raw("{\"schema\":\"flashtex.literal-replacement-plan.v1\",\"proposal_only\":true,\"requires_user_approval\":true,\"application_order\":\"reverse_byte_offset_per_document\",\"snapshot\":{\"project_id\":")?;
        json.string(&plan.search.snapshot.project_id)?;
        json.raw(",\"generation\":")?;
        json.decimal(plan.search.snapshot.generation)?;
        json.raw(",\"documents\":[")?;
        for (at, (file, revision)) in plan.search.snapshot.documents.iter().enumerate() {
            if at != 0 {
                json.raw(",")?;
            }
            json.raw("{\"file\":")?;
            json.string(file)?;
            json.raw(",\"revision\":")?;
            json.decimal(revision)?;
            json.raw("}")?;
        }
        json.raw("]},\"search\":{\"literal\":")?;
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
        json.raw(",\"edits\":[")?;
        for (at, edit) in plan.edits.iter().enumerate() {
            if at != 0 {
                json.raw(",")?;
            }
            json.raw("{\"file\":")?;
            json.string(&edit.source.file)?;
            json.raw(",\"revision\":")?;
            json.decimal(edit.source.revision)?;
            json.raw(",\"start_byte\":")?;
            json.decimal(edit.source.start_byte)?;
            json.raw(",\"end_byte\":")?;
            json.decimal(edit.source.end_byte)?;
            json.raw(",\"expected_text\":")?;
            json.string(&edit.expected_text)?;
            json.raw(",\"replacement\":")?;
            json.string(&edit.replacement)?;
            json.raw("}")?;
        }
        json.raw("]}")?;
        Ok(json.text)
    }
}
