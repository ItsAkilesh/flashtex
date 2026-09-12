use super::{Cff, CffIdentity, CffOutlineCache};
use crate::{
    encoding::{EncodingEntry, GlyphIdentity, MappedItem},
    invalid,
    tfm::{CharacterMetrics, Tfm, TfmItem},
    Result,
};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, sync::Arc};
fn valid_name(name: &str) -> bool {
    !name.is_empty()
        && name.len() <= 127
        && name
            .bytes()
            .all(|b| (33..=126).contains(&b) && !b"[](){}<>/%".contains(&b))
}
#[derive(Debug, Clone)]
pub struct CffGlyphNames {
    cff_sha256: String,
    by_name: BTreeMap<String, u16>,
    by_gid: Vec<String>,
    retained_bytes: usize,
}
impl Cff {
    pub fn sid_string(&self, sid: u16) -> Result<&str> {
        if sid < 391 {
            return Ok(super::standard_strings::STANDARD_STRINGS[sid as usize]);
        }
        std::str::from_utf8(
            self.custom_string(sid)
                .ok_or_else(|| invalid("CFF SID outside standard/custom strings"))?,
        )
        .map_err(|_| invalid("CFF SID string is not UTF-8"))
    }
    pub fn glyph_names(&self) -> Result<CffGlyphNames> {
        if self.charset.len() != self.glyph_count() || self.charset.first() != Some(&0) {
            return Err(invalid("CFF charset/GID0 mismatch"));
        }
        let mut retained_bytes = 256usize;
        let mut by_name = BTreeMap::new();
        let mut by_gid = Vec::with_capacity(self.glyph_count());
        for (gid, &sid) in self.charset.iter().enumerate() {
            let name = self.sid_string(sid)?;
            if !valid_name(name) {
                return Err(invalid("CFF glyph name invalid or outside127-byte limit"));
            }
            if by_name.insert(name.to_owned(), gid as u16).is_some() {
                return Err(invalid("duplicate CFF glyph name across charset SIDs"));
            }
            retained_bytes += 256 + name.len() * 2;
            if retained_bytes > 16 * 1024 * 1024 {
                return Err(invalid("CFF glyph-name index byte budget"));
            }
            by_gid.push(name.to_owned());
        }
        Ok(CffGlyphNames {
            cff_sha256: self.sha256.clone(),
            by_name,
            by_gid,
            retained_bytes,
        })
    }
}
impl CffGlyphNames {
    pub fn retained_bytes(&self) -> usize {
        self.retained_bytes
    }
    pub fn cff_sha256(&self) -> &str {
        &self.cff_sha256
    }
    pub fn glyph_name(&self, gid: u16) -> Result<&str> {
        self.by_gid
            .get(gid as usize)
            .map(String::as_str)
            .ok_or_else(|| invalid("CFF name lookup GID outside charset"))
    }
    pub fn resolve(&self, name: &str) -> Result<GlyphIdentity> {
        if !valid_name(name) {
            return Err(invalid("invalid CFF glyph name lookup"));
        }
        let gid = *self
            .by_name
            .get(name)
            .ok_or_else(|| invalid("CFF glyph name absent"))?;
        Ok(if gid == 0 {
            GlyphIdentity::Notdef
        } else {
            GlyphIdentity::Original(gid)
        })
    }
    pub fn len(&self) -> usize {
        self.by_gid.len()
    }
    pub fn is_empty(&self) -> bool {
        self.by_gid.is_empty()
    }
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CffEncodingManifest {
    pub font_sha256: String,
    pub cff_sha256: String,
    pub tfm_sha256: String,
    pub face_index: u32,
    pub encoding: Vec<EncodingEntry>,
}
#[derive(Debug, Clone)]
pub struct ResolvedCffEncoding {
    identity: CffIdentity,
    tfm_sha256: String,
    digest: String,
    mapping: BTreeMap<u8, (String, GlyphIdentity)>,
}
impl ResolvedCffEncoding {
    pub fn identity(&self) -> &CffIdentity {
        &self.identity
    }
    pub fn encoding_sha256(&self) -> &str {
        &self.digest
    }
    pub fn tfm_sha256(&self) -> &str {
        &self.tfm_sha256
    }
    pub fn resolve(&self, code: u8) -> Result<GlyphIdentity> {
        self.mapping
            .get(&code)
            .map(|(_, gid)| *gid)
            .ok_or_else(|| invalid("TFM code missing from explicit CFF encoding"))
    }
    pub fn glyph_name(&self, code: u8) -> Option<&str> {
        self.mapping.get(&code).map(|(name, _)| name.as_str())
    }
    pub(super) fn bind(
        identity: &CffIdentity,
        names: &CffGlyphNames,
        tfm: &Tfm,
        manifest: &CffEncodingManifest,
    ) -> Result<Self> {
        if manifest.font_sha256 != identity.font_sha256
            || manifest.cff_sha256 != identity.cff_sha256
            || manifest.face_index != identity.face_index
            || manifest.tfm_sha256 != tfm.source_sha256
            || names.cff_sha256() != identity.cff_sha256
        {
            return Err(invalid("CFF encoding full resource identity mismatch"));
        }
        if manifest.encoding.len() > 256 {
            return Err(invalid("CFF encoding entry budget"));
        }
        let mut mapping = BTreeMap::new();
        for entry in &manifest.encoding {
            let gid = names.resolve(&entry.glyph_name)?;
            if mapping
                .insert(entry.code, (entry.glyph_name.clone(), gid))
                .is_some()
            {
                return Err(invalid("duplicate explicit CFF encoding code"));
            }
        }
        let mut canonical = Vec::new();
        canonical.extend(b"cff-encoding-v1\0");
        for (code, (name, _)) in &mapping {
            canonical.push(*code);
            canonical.extend((name.len() as u32).to_be_bytes());
            canonical.extend(name.as_bytes());
        }
        Ok(Self {
            identity: identity.clone(),
            tfm_sha256: tfm.source_sha256.clone(),
            digest: crate::sha256(&canonical),
            mapping,
        })
    }
}
pub struct BoundCffTfmFont<'a> {
    tfm: &'a Tfm,
    encoding: Arc<ResolvedCffEncoding>,
}
impl<'a> BoundCffTfmFont<'a> {
    pub fn new(
        tfm: &'a Tfm,
        font: &CffOutlineCache,
        manifest: &CffEncodingManifest,
    ) -> Result<Self> {
        let names = font.glyph_names()?;
        let encoding = ResolvedCffEncoding::bind(font.identity(), &names, tfm, manifest)?;
        Ok(Self {
            tfm,
            encoding: Arc::new(encoding),
        })
    }
    pub fn from_resolved(tfm: &'a Tfm, encoding: Arc<ResolvedCffEncoding>) -> Result<Self> {
        if tfm.source_sha256 != encoding.tfm_sha256 {
            return Err(invalid("CFF encoding TFM identity mismatch"));
        }
        Ok(Self { tfm, encoding })
    }
    pub fn tfm(&self) -> &'a Tfm {
        self.tfm
    }
    pub fn encoding(&self) -> &Arc<ResolvedCffEncoding> {
        &self.encoding
    }
    pub fn identity(&self) -> &CffIdentity {
        self.encoding.identity()
    }
    pub fn validate_cache(&self, font: &CffOutlineCache) -> Result<()> {
        if self.identity() != font.identity() {
            return Err(invalid("CFF outline cache identity mismatch"));
        }
        Ok(())
    }
    pub fn map_code(&self, code: u8) -> Result<(GlyphIdentity, CharacterMetrics)> {
        Ok((
            self.encoding.resolve(code)?,
            self.tfm
                .char_metrics(code)
                .ok_or_else(|| invalid("CFF encoded TFM metrics absent"))?,
        ))
    }
    pub fn map_run(&self, input: &[u8]) -> Result<Vec<MappedItem>> {
        self.tfm
            .apply_ligatures_kerns(input)?
            .into_iter()
            .map(|item| match item {
                TfmItem::Kern(k) => Ok(MappedItem::Kern(k)),
                TfmItem::Glyph(g) => {
                    let (identity, metrics) = self.map_code(g.code)?;
                    Ok(MappedItem::Glyph {
                        tfm_code: g.code,
                        identity,
                        metrics,
                        input_start: g.input_start,
                        input_end: g.input_end,
                    })
                }
            })
            .collect()
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn standard_sid_registry_and_notdef() {
        let c = Cff::parse(&super::super::tests::fixture()).unwrap();
        assert_eq!(c.sid_string(34).unwrap(), "A");
        assert_eq!(c.sid_string(390).unwrap(), "Semibold");
        assert!(c.sid_string(391).is_err());
        assert_eq!(
            c.glyph_names().unwrap().resolve(".notdef").unwrap(),
            GlyphIdentity::Notdef
        );
        assert!(c.glyph_names().unwrap().resolve("A").is_err());
        assert_eq!(
            super::super::standard_strings::STANDARD_STRINGS
                .iter()
                .collect::<std::collections::BTreeSet<_>>()
                .len(),
            391
        );
    }
    #[test]
    fn custom_sid_duplicate_and_invalid_names() {
        let mut c = Cff::parse(&super::super::tests::fixture()).unwrap();
        let mut bytes = c.bytes.to_vec();
        let start = bytes.len();
        bytes.extend(b"A.alt");
        c.strings = std::iter::once(start..bytes.len()).collect();
        c.bytes = bytes.into();
        c.charset.push(391);
        c.charstrings.push(c.charstrings[0].clone());
        assert_eq!(
            c.glyph_names().unwrap().resolve("A.alt").unwrap(),
            GlyphIdentity::Original(1)
        );
        let mut duplicate = c.clone();
        let mut data = duplicate.bytes.to_vec();
        let duplicate_start = data.len();
        data.extend(b".notdef");
        duplicate.strings = std::iter::once(duplicate_start..data.len()).collect();
        duplicate.bytes = data.into();
        assert!(duplicate.glyph_names().is_err());
        c.charset[1] = 0;
        assert!(c.glyph_names().is_err());
        c.charset[1] = 392;
        assert!(c.glyph_names().is_err());
        c.charset[1] = 391;
        let mut bytes = c.bytes.to_vec();
        bytes[start] = b' ';
        c.bytes = bytes.into();
        assert!(c.glyph_names().is_err());
    }
}
