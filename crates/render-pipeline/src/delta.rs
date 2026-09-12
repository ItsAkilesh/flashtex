//! `display-list-v2-delta` (ISOLATED implementation of
//! `docs/proposals/display-list-v2-delta.md` r5; not activated on any product
//! route). One retained snapshot per worker (`RenderCache::delta`), the
//! `dl2-canon-1` digests, the exact `page_bytes` accounting from the real
//! writer, source relocation, page classification, the `display_list_delta`
//! envelope, and the reference reconstruction used by the byte-identity gate.
//!
//! Nothing here changes the full `display_list` line, the `compile_result`
//! or the decline fallback: a delta is emitted only when the request lists
//! the capability AND acknowledges the producer's last emitted snapshot in
//! `payload.display_list_base`; every other case is the unchanged full reply.

use flashtex_compiler::json::{self, Value};
use flashtex_font_engine::sha256;

use crate::display::{self, DisplayList, Diagnostic, Item, Page, Provenance, SourceRange};

pub const CAP_DELTA: &str = "display-list-v2-delta";
pub const DIGEST_SCHEME: &str = "dl2-canon-1";
pub const MESSAGE_TYPE: &str = "display_list_delta";
/// A snapshot with more pages than this is never retained (proposal §6.2).
pub const MAX_SNAPSHOT_PAGES: usize = 1024;
/// Exact serialised size of a retained snapshot (proposal §6.2).
pub const MAX_SNAPSHOT_BYTES: usize = 16 * 1024 * 1024;
/// Retained request texts: per document and in total (proposal §6.2).
pub const MAX_TEXT_BYTES: usize = 8 * 1024 * 1024;
pub const MAX_TOTAL_TEXT_BYTES: usize = 32 * 1024 * 1024;
/// A delta is emitted only when its line is at most this fraction of the
/// full line it replaces (policy, not contract; proposal §3).
pub const POLICY_NUM: usize = 3;
pub const POLICY_DEN: usize = 4;

/// What the producer retains after emitting a sibling (full or delta):
/// exactly one per worker, replaced by every emitted sibling, cleared by any
/// reply without a sibling (proposal §6.2 "old").
#[derive(Clone)]
pub struct Snapshot {
    pub request_id: String,
    pub list: DisplayList,
    pub page_digests: Vec<[u8; 32]>,
    pub list_digest: [u8; 32],
    /// Exact byte length of each page object as the writer emits it.
    pub page_bytes: Vec<usize>,
    /// `(path, text)` of every request document, for the relocation diff.
    pub texts: Vec<(String, String)>,
}

/// The consumer's installed-base acknowledgement from the request.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BaseAck {
    pub request_id: String,
    pub project_id: String,
    pub revision: u64,
    pub page_count: usize,
    pub list_digest: String,
}

impl BaseAck {
    /// Parses `payload.display_list_base`; `None` when absent or malformed
    /// (a malformed acknowledgement is simply "no acknowledgement").
    pub fn parse(payload: &Value) -> Option<BaseAck> {
        let b = payload.get("display_list_base")?;
        Some(BaseAck {
            request_id: b.get("request_id")?.as_str()?.to_string(),
            project_id: b.get("project_id")?.as_str()?.to_string(),
            revision: u64::try_from(b.get("revision")?.as_i64()?).ok()?,
            page_count: usize::try_from(b.get("page_count")?.as_i64()?).ok()?,
            list_digest: b.get("list_digest")?.as_str()?.to_string(),
        })
    }

    pub fn matches(&self, s: &Snapshot) -> bool {
        self.request_id == s.request_id
            && self.project_id == s.list.project_id
            && self.revision == s.list.revision
            && self.page_count == s.list.pages.len()
            && self.list_digest == sha256::hex(&s.list_digest)
    }
}

// ---------------------------------------------------------------------------
// dl2-canon-1

struct Canon(Vec<u8>);

impl Canon {
    fn i(&mut self, n: i64) {
        self.0.extend_from_slice(&n.to_le_bytes());
    }
    fn u(&mut self, n: usize) {
        self.i(n as i64);
    }
    fn s(&mut self, s: &str) {
        self.u(s.len());
        self.0.extend_from_slice(s.as_bytes());
    }
    /// Raw IEEE-754 bits after normalising -0.0 to +0.0; the wire cannot
    /// carry non-finite values and `Paint` is validated finite upstream.
    fn f(&mut self, x: f64) {
        debug_assert!(x.is_finite());
        let x = if x == 0.0 { 0.0 } else { x };
        self.0.extend_from_slice(&x.to_bits().to_le_bytes());
    }
    fn ranges(&mut self, rs: &[SourceRange]) {
        self.u(rs.len());
        for r in rs {
            self.s(&r.path);
            self.u(r.start_byte);
            self.u(r.end_byte);
        }
    }
    fn provenance(&mut self, p: &Provenance) {
        match p {
            Provenance::Source(_) | Provenance::Sources(_) => {
                self.0.push(0x10);
                self.ranges(p.sources());
            }
            Provenance::Synthetic(reason) => {
                self.0.push(0x11);
                self.s(reason);
            }
        }
    }
    fn paint(&mut self, p: &display::Paint) {
        self.f(p.r);
        self.f(p.g);
        self.f(p.b);
        self.f(p.a);
    }
}

pub fn page_digest(p: &Page) -> [u8; 32] {
    let mut c = Canon(b"flashtex:dl2:page:1\0".to_vec());
    c.i(i64::from(p.number));
    c.i(p.width.0);
    c.i(p.height.0);
    c.u(p.items.len());
    for it in &p.items {
        match it {
            Item::GlyphRun(r) => {
                c.0.push(0x01);
                c.s(&r.font_id);
                c.i(r.font_size.0);
                c.s(&r.text);
                c.u(r.glyphs.len());
                for g in &r.glyphs {
                    c.i(i64::from(g.gid));
                    c.i(g.origin_x.0);
                    c.i(g.baseline_y.0);
                    c.i(g.advance_x.0);
                    c.i(g.advance_y.0);
                    c.i(i64::from(g.cluster));
                }
                c.u(r.clusters.len());
                for cl in &r.clusters {
                    c.u(cl.text_start_byte);
                    c.u(cl.text_end_byte);
                    c.u(cl.hit_rects().len());
                    for h in cl.hit_rects() {
                        c.i(h.x.0);
                        c.i(h.top.0);
                        c.i(h.width.0);
                        c.i(h.height.0);
                    }
                    c.u(cl.carets.len());
                    for k in cl.carets.iter() {
                        c.u(k.text_byte);
                        c.i(k.x.0);
                        c.i(k.top.0);
                        c.i(k.height.0);
                    }
                    c.provenance(&cl.provenance);
                }
                c.paint(&r.paint);
            }
            Item::Rule(r) => {
                c.0.push(0x02);
                c.i(r.x.0);
                c.i(r.top.0);
                c.i(r.width.0);
                c.i(r.height.0);
                c.paint(&r.paint);
                c.provenance(&r.provenance);
            }
        }
    }
    sha256::digest(&c.0)
}

pub fn header_digest(l: &DisplayList) -> [u8; 32] {
    let mut c = Canon(b"flashtex:dl2:header:1\0".to_vec());
    for s in ["display-list-v2", "bp_2pow20", "srgb", "cluster-actualtext"] {
        c.s(s);
    }
    c.s(&l.project_id);
    c.i(l.revision as i64);
    let features = l.required_features();
    c.u(features.len());
    for f in features {
        c.s(f);
    }
    c.u(l.documents.len());
    for d in &l.documents {
        c.s(&d.path);
        c.i(d.revision as i64);
        c.s(&d.sha256);
        c.i(d.byte_length as i64);
    }
    c.u(l.fonts.len());
    for f in &l.fonts {
        c.s(&f.font_id);
        c.s(&f.sha256);
        c.i(f.byte_length as i64);
        c.s(&f.format);
        c.i(i64::from(f.face_index));
        c.i(i64::from(f.units_per_em));
        c.i(i64::from(f.glyph_count));
        c.s(&f.postscript_name);
    }
    c.u(l.diagnostics.len());
    for d in &l.diagnostics {
        c.s(&d.code);
        c.s(&d.message);
        c.s(match d.severity {
            display::Severity::Warning => "warning",
            display::Severity::Error => "error",
        });
        c.ranges(&d.sources);
    }
    sha256::digest(&c.0)
}

pub fn list_digest(l: &DisplayList, page_digests: &[[u8; 32]]) -> [u8; 32] {
    let mut c = Canon(b"flashtex:dl2:list:1\0".to_vec());
    c.0.extend_from_slice(&header_digest(l));
    c.u(page_digests.len());
    for d in page_digests {
        c.0.extend_from_slice(d);
    }
    sha256::digest(&c.0)
}

// ---------------------------------------------------------------------------
// exact page bytes (the real writer) and the relocation arithmetic

/// The page object exactly as it appears inside the full line's `pages`.
pub fn page_wire(p: &Page) -> String {
    json::write(&display::page_json(p))
}

pub fn page_bytes(p: &Page) -> usize {
    page_wire(p).len()
}

/// Decimal width of an integral value as the writer prints it
/// (`json::write`: integral, |n| < 1e15 → `{}` of the i64).
fn digits(n: usize) -> usize {
    if n == 0 {
        1
    } else {
        (n as f64).log10().floor() as usize + 1
    }
}

/// One document's edit region in BASE coordinates: `[edit_start, edit_end)`
/// was replaced by `edit_end - edit_start + delta` bytes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Relocation {
    pub path: String,
    pub edit_start: usize,
    pub edit_end: usize,
    pub delta: i64,
}

impl Relocation {
    /// Common-prefix / common-suffix diff of one document; `None` when equal.
    pub fn diff(path: &str, old: &str, new: &str) -> Option<Relocation> {
        if old == new {
            return None;
        }
        let (o, n) = (old.as_bytes(), new.as_bytes());
        let mut a = 0;
        while a < o.len() && a < n.len() && o[a] == n[a] {
            a += 1;
        }
        let mut s = 0;
        while s < o.len() - a && s < n.len() - a && o[o.len() - 1 - s] == n[n.len() - 1 - s] {
            s += 1;
        }
        Some(Relocation {
            path: path.to_string(),
            edit_start: a,
            edit_end: o.len() - s,
            delta: n.len() as i64 - o.len() as i64,
        })
    }

    /// Moves one range; `None` when it intersects the edited region.
    fn apply(&self, r: &SourceRange) -> Option<SourceRange> {
        if r.end_byte <= self.edit_start {
            Some(r.clone())
        } else if r.start_byte >= self.edit_end {
            let mv = |v: usize| usize::try_from(v as i64 + self.delta).ok();
            Some(SourceRange {
                path: r.path.clone(),
                start_byte: mv(r.start_byte)?,
                end_byte: mv(r.end_byte)?,
            })
        } else {
            None
        }
    }

    /// Byte-length change of one range's decimal offsets under this move.
    fn width_delta(&self, r: &SourceRange) -> Option<i64> {
        let moved = self.apply(r)?;
        Some((digits(moved.start_byte) as i64 - digits(r.start_byte) as i64) + (digits(moved.end_byte) as i64 - digits(r.end_byte) as i64))
    }
}

fn reloc_for<'a>(relocs: &'a [Relocation], path: &str) -> Option<&'a Relocation> {
    relocs.iter().find(|r| &*r.path == path)
}

fn relocate_prov(p: &Provenance, relocs: &[Relocation]) -> Option<Provenance> {
    let mv = |r: &SourceRange| match reloc_for(relocs, &r.path) {
        Some(rl) => rl.apply(r),
        None => Some(r.clone()),
    };
    Some(match p {
        Provenance::Source(r) => Provenance::Source(mv(r)?),
        Provenance::Sources(v) => Provenance::Sources(v.iter().map(mv).collect::<Option<Vec<_>>>()?),
        Provenance::Synthetic(s) => Provenance::Synthetic(s.clone()),
    })
}

/// A NEW page equal to `p` with every source range moved; `None` when any
/// range intersects an edited region (the page cannot be "unchanged").
pub fn relocate_page(p: &Page, relocs: &[Relocation]) -> Option<Page> {
    let mut out = p.clone();
    for it in &mut out.items {
        match it {
            Item::GlyphRun(r) => {
                for c in &mut r.clusters {
                    c.provenance = relocate_prov(&c.provenance, relocs)?;
                }
            }
            Item::Rule(r) => r.provenance = relocate_prov(&r.provenance, relocs)?,
        }
    }
    Some(out)
}

/// Exact byte length of `relocate_page(p)` from the cached length of `p`,
/// without serialising: only the decimal width of moved offsets changes.
pub fn relocated_page_bytes(p: &Page, cached: usize, relocs: &[Relocation]) -> Option<usize> {
    let mut delta: i64 = 0;
    let mut visit = |prov: &Provenance| -> Option<()> {
        for r in prov.sources() {
            if let Some(rl) = reloc_for(relocs, &r.path) {
                delta = delta.checked_add(rl.width_delta(r)?)?;
            }
        }
        Some(())
    };
    for it in &p.items {
        match it {
            Item::GlyphRun(r) => {
                for c in &r.clusters {
                    visit(&c.provenance)?;
                }
            }
            Item::Rule(r) => visit(&r.provenance)?,
        }
    }
    usize::try_from(cached as i64 + delta).ok()
}

// ---------------------------------------------------------------------------
// the delta

pub struct Delta {
    pub base: BaseAck,
    /// `required_features` of the reconstructed list, as carried on the wire.
    pub required_features: Vec<String>,
    pub relocations: Vec<Relocation>,
    pub page_count: usize,
    pub page_digests: Vec<[u8; 32]>,
    pub page_bytes: Vec<usize>,
    /// Indices (0-based) of the pages carried in full.
    pub changed: Vec<usize>,
    pub removed_pages: Vec<u32>,
    pub list_digest: [u8; 32],
}

/// Why the producer answered with the unchanged full line instead of a delta.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NoDelta {
    NotRequested,
    NoAcknowledgement,
    NoSnapshot,
    BaseMismatch,
    DocumentSetChanged,
    Oversize,
    NotSmaller,
}

/// Exact full-line length from the header (`to_json` with the pages
/// emptied) and the page lengths: `F + H + Σ page_bytes + max(N−1, 0)`.
pub fn full_line_bytes(list: &DisplayList, id: &str, features: &[&str], page_bytes: &[usize]) -> Option<usize> {
    let mut header = list.clone();
    header.pages.clear();
    let frame = json::write(&header.to_json_with(id, features)).len();
    let mut total = frame.checked_add(page_bytes.len().saturating_sub(1))?;
    for b in page_bytes {
        total = total.checked_add(*b)?;
    }
    Some(total)
}

/// Builds the snapshot the producer retains for `list` (exact page lengths
/// from the writer, digests) or `None` when it exceeds the retention caps.
pub fn snapshot(request_id: &str, list: &DisplayList, texts: &[(String, String)]) -> Option<Snapshot> {
    if list.pages.len() > MAX_SNAPSHOT_PAGES {
        return None;
    }
    if texts.iter().any(|(_, t)| t.len() > MAX_TEXT_BYTES) || texts.iter().map(|(_, t)| t.len()).sum::<usize>() > MAX_TOTAL_TEXT_BYTES {
        return None;
    }
    let page_bytes: Vec<usize> = list.pages.iter().map(page_bytes).collect();
    if full_line_bytes(list, request_id, &list.required_features(), &page_bytes)? > MAX_SNAPSHOT_BYTES {
        return None;
    }
    let page_digests: Vec<[u8; 32]> = list.pages.iter().map(page_digest).collect();
    let list_digest = list_digest(list, &page_digests);
    Some(Snapshot {
        request_id: request_id.to_string(),
        list: list.clone(),
        page_digests,
        list_digest,
        page_bytes,
        texts: texts.to_vec(),
    })
}

/// Classifies `new` against `old` and builds the delta, or says why not.
/// `page_bytes` of unchanged pages come from the cached lengths plus the
/// relocation arithmetic (never from serialising the page).
pub fn build(old: &Snapshot, ack: &BaseAck, new: &DisplayList, texts: &[(String, String)]) -> Result<Delta, NoDelta> {
    if !ack.matches(old) {
        return Err(NoDelta::BaseMismatch);
    }
    if old.list.project_id != new.project_id
        || old.texts.len() != texts.len()
        || old.texts.iter().zip(texts).any(|((a, _), (b, _))| a != b)
    {
        return Err(NoDelta::DocumentSetChanged);
    }
    let relocations: Vec<Relocation> = old
        .texts
        .iter()
        .zip(texts)
        .filter_map(|((p, o), (_, n))| Relocation::diff(p, o, n))
        .collect();
    let n = new.pages.len();
    let mut changed = Vec::new();
    let mut page_bytes = Vec::with_capacity(n);
    let mut page_digests = Vec::with_capacity(n);
    for (i, page) in new.pages.iter().enumerate() {
        let reused = old.list.pages.get(i).and_then(|b| {
            let moved = relocate_page(b, &relocations)?;
            if moved == *page {
                let bytes = relocated_page_bytes(b, old.page_bytes[i], &relocations)?;
                Some((bytes, old_page_digest_after_move(&moved)))
            } else {
                None
            }
        });
        match reused {
            Some((bytes, digest)) => {
                page_bytes.push(bytes);
                page_digests.push(digest);
            }
            None => {
                changed.push(i);
                page_bytes.push(self::page_bytes(page));
                page_digests.push(page_digest(page));
            }
        }
    }
    let removed_pages: Vec<u32> = ((n as u32 + 1)..=(old.list.pages.len() as u32)).collect();
    let list_digest = list_digest(new, &page_digests);
    Ok(Delta {
        base: ack.clone(),
        required_features: new.required_features().into_iter().map(String::from).collect(),
        relocations,
        page_count: n,
        page_digests,
        page_bytes,
        changed,
        removed_pages,
        list_digest,
    })
}

fn old_page_digest_after_move(moved: &Page) -> [u8; 32] {
    page_digest(moved)
}

impl Delta {
    /// Reads the accounting/identity fields of a `display_list_delta` line
    /// (everything except the page objects themselves, which callers decode
    /// with their own page reader). `None` on any malformed field.
    pub fn from_wire(envelope: &Value) -> Option<Delta> {
        let p = envelope.get("payload")?;
        if envelope.get("type")?.as_str()? != MESSAGE_TYPE || p.get("digest_scheme")?.as_str()? != DIGEST_SCHEME {
            return None;
        }
        let b = p.get("base")?;
        let base = BaseAck {
            request_id: b.get("request_id")?.as_str()?.to_string(),
            project_id: b.get("project_id")?.as_str()?.to_string(),
            revision: u64::try_from(b.get("revision")?.as_i64()?).ok()?,
            page_count: usize::try_from(b.get("page_count")?.as_i64()?).ok()?,
            list_digest: b.get("list_digest")?.as_str()?.to_string(),
        };
        let hex32 = |v: &Value| -> Option<[u8; 32]> {
            let s = v.as_str()?;
            if s.len() != 64 {
                return None;
            }
            let mut out = [0u8; 32];
            for i in 0..32 {
                out[i] = u8::from_str_radix(&s[2 * i..2 * i + 2], 16).ok()?;
            }
            Some(out)
        };
        let page_count = usize::try_from(p.get("page_count")?.as_i64()?).ok()?;
        let page_digests = p.get("page_digests")?.as_arr()?.iter().map(hex32).collect::<Option<Vec<_>>>()?;
        let page_bytes = p
            .get("page_bytes")?
            .as_arr()?
            .iter()
            .map(|v| usize::try_from(v.as_i64()?).ok())
            .collect::<Option<Vec<_>>>()?;
        if page_digests.len() != page_count || page_bytes.len() != page_count || page_count > MAX_SNAPSHOT_PAGES {
            return None;
        }
        let changed = p
            .get("changed_pages")?
            .as_arr()?
            .iter()
            .map(|pg| usize::try_from(pg.get("number")?.as_i64()?).ok()?.checked_sub(1))
            .collect::<Option<Vec<_>>>()?;
        if changed.iter().any(|&i| i >= page_count) || changed.windows(2).any(|w| w[0] >= w[1]) {
            return None;
        }
        let removed_pages = p
            .get("removed_pages")?
            .as_arr()?
            .iter()
            .map(|v| u32::try_from(v.as_i64()?).ok())
            .collect::<Option<Vec<_>>>()?;
        let relocations = p
            .get("relocations")?
            .as_arr()?
            .iter()
            .map(|r| {
                Some(Relocation {
                    path: r.get("path")?.as_str()?.to_string(),
                    edit_start: usize::try_from(r.get("edit_start")?.as_i64()?).ok()?,
                    edit_end: usize::try_from(r.get("edit_end")?.as_i64()?).ok()?,
                    delta: r.get("delta")?.as_i64()?,
                })
            })
            .collect::<Option<Vec<_>>>()?;
        if relocations.iter().any(|r| r.edit_start > r.edit_end) {
            return None;
        }
        let required_features = p
            .get("required_features")?
            .as_arr()?
            .iter()
            .map(|v| Some(v.as_str()?.to_string()))
            .collect::<Option<Vec<_>>>()?;
        Some(Delta {
            base,
            required_features,
            relocations,
            page_count,
            page_digests,
            page_bytes,
            changed,
            removed_pages,
            list_digest: hex32(p.get("list_digest")?)?,
        })
    }

    /// The `display_list_delta` envelope (proposal §4).
    pub fn to_json(&self, id: &str, new: &DisplayList) -> Value {
        let hex = |d: &[u8; 32]| json::str_(sha256::hex(d));
        let mut payload = Value::obj();
        payload.set("render_format", json::str_("display-list-v2"));
        payload.set("coordinate_unit", json::str_("bp_2pow20"));
        payload.set("color_space", json::str_("srgb"));
        payload.set("text_extraction", json::str_("cluster-actualtext"));
        payload.set("project_id", json::str_(new.project_id.clone()));
        payload.set("revision", json::num(new.revision as f64));
        payload.set(
            "required_features",
            Value::Arr(self.required_features.iter().map(|f| json::str_(f.clone())).collect()),
        );
        payload.set("digest_scheme", json::str_(DIGEST_SCHEME));
        let mut base = Value::obj();
        base.set("request_id", json::str_(self.base.request_id.clone()));
        base.set("project_id", json::str_(self.base.project_id.clone()));
        base.set("revision", json::num(self.base.revision as f64));
        base.set("page_count", json::num(self.base.page_count as f64));
        base.set("list_digest", json::str_(self.base.list_digest.clone()));
        payload.set("base", base);
        // documents / fonts / diagnostics: the full line's own values.
        let full = new.to_json(id);
        let fp = full.get("payload").expect("payload");
        for k in ["documents", "fonts", "diagnostics"] {
            payload.set(k, fp.get(k).expect(k).clone());
        }
        payload.set(
            "relocations",
            Value::Arr(
                self.relocations
                    .iter()
                    .map(|r| {
                        let mut o = Value::obj();
                        o.set("path", json::str_(r.path.clone()));
                        o.set("edit_start", json::num(r.edit_start as f64));
                        o.set("edit_end", json::num(r.edit_end as f64));
                        o.set("delta", json::num(r.delta as f64));
                        o
                    })
                    .collect(),
            ),
        );
        payload.set("page_count", json::num(self.page_count as f64));
        payload.set("page_digests", Value::Arr(self.page_digests.iter().map(hex).collect()));
        payload.set("page_bytes", Value::Arr(self.page_bytes.iter().map(|b| json::num(*b as f64)).collect()));
        payload.set(
            "changed_pages",
            Value::Arr(self.changed.iter().map(|&i| display::page_json(&new.pages[i])).collect()),
        );
        payload.set(
            "removed_pages",
            Value::Arr(self.removed_pages.iter().map(|n| json::num(f64::from(*n))).collect()),
        );
        payload.set("list_digest", hex(&self.list_digest));
        let mut v = Value::obj();
        v.set("protocol_version", json::num(display::PROTOCOL_VERSION as f64));
        v.set("id", json::str_(id));
        v.set("type", json::str_(MESSAGE_TYPE));
        v.set("payload", payload);
        v
    }
}

// ---------------------------------------------------------------------------
// reference reconstruction (the consumer algorithm, model level; used by the
// gate and by consumer implementers as the executable specification)

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Refusal {
    BaseMismatch,
    PageCount,
    RemovedPages,
    PageBytesMismatch(usize),
    TargetOversize { bytes: usize, cap: usize },
    RelocationInvalid(usize),
    DigestMismatch(usize),
    ListDigestMismatch,
}

/// Applies a delta (as the producer's `Delta` plus the new header/changed
/// pages) to an installed snapshot: verifies `page_bytes` and the exact
/// target size BEFORE building pages, then reconstructs and verifies digests.
/// `changed` are the changed pages exactly as carried on the wire.
pub fn apply(installed: &Snapshot, d: &Delta, header: &DisplayList, changed: &[Page], id: &str) -> Result<DisplayList, Refusal> {
    apply_with_cap(installed, d, header, changed, id, MAX_SNAPSHOT_BYTES)
}

/// `apply` with an explicit byte cap (tests exercise cap + 1).
pub fn apply_with_cap(installed: &Snapshot, d: &Delta, header: &DisplayList, changed: &[Page], id: &str, cap: usize) -> Result<DisplayList, Refusal> {
    if !d.base.matches(installed) {
        return Err(Refusal::BaseMismatch);
    }
    let n = d.page_count;
    if d.page_digests.len() != n || d.page_bytes.len() != n || d.changed.len() != changed.len() || d.changed.len() > n {
        return Err(Refusal::PageCount);
    }
    let unchanged: Vec<usize> = (0..n).filter(|i| !d.changed.contains(i)).collect();
    if unchanged.iter().any(|&i| i >= installed.list.pages.len()) {
        return Err(Refusal::PageCount);
    }
    let expected_removed: Vec<u32> = ((n as u32 + 1)..=(installed.list.pages.len() as u32)).collect();
    if d.removed_pages != expected_removed {
        return Err(Refusal::RemovedPages);
    }
    // page_bytes verification + exact target size, before any page is built.
    for (k, &i) in d.changed.iter().enumerate() {
        if page_bytes(&changed[k]) != d.page_bytes[i] || changed[k].number as usize != i + 1 {
            return Err(Refusal::PageBytesMismatch(i + 1));
        }
    }
    for &i in &unchanged {
        let got = relocated_page_bytes(&installed.list.pages[i], installed.page_bytes[i], &d.relocations).ok_or(Refusal::RelocationInvalid(i + 1))?;
        if got != d.page_bytes[i] {
            return Err(Refusal::PageBytesMismatch(i + 1));
        }
    }
    let features: Vec<&str> = d.required_features.iter().map(String::as_str).collect();
    let target = full_line_bytes(header, id, &features, &d.page_bytes).ok_or(Refusal::TargetOversize { bytes: usize::MAX, cap })?;
    if target > cap || n > MAX_SNAPSHOT_PAGES {
        return Err(Refusal::TargetOversize { bytes: target, cap });
    }
    // reconstruction
    let mut pages = Vec::with_capacity(n);
    let mut k = 0;
    for i in 0..n {
        let page = if d.changed.contains(&i) {
            let p = changed[k].clone();
            k += 1;
            p
        } else {
            relocate_page(&installed.list.pages[i], &d.relocations).ok_or(Refusal::RelocationInvalid(i + 1))?
        };
        if page_digest(&page) != d.page_digests[i] {
            return Err(Refusal::DigestMismatch(i + 1));
        }
        pages.push(page);
    }
    let mut list = header.clone();
    list.pages = pages;
    if list_digest(&list, &d.page_digests) != d.list_digest {
        return Err(Refusal::ListDigestMismatch);
    }
    Ok(list)
}

/// Diagnostics convenience for tests: the diagnostics of a list, relocated.
pub fn relocated_diagnostics(diags: &[Diagnostic], relocs: &[Relocation]) -> Option<Vec<Diagnostic>> {
    diags
        .iter()
        .map(|d| {
            let mut d = d.clone();
            for s in &mut d.sources {
                if let Some(rl) = reloc_for(relocs, &s.path) {
                    *s = rl.apply(s)?;
                }
            }
            Some(d)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn digits_match_the_writer() {
        for n in [0usize, 1, 9, 10, 99, 100, 999, 1000, 65535, 1_000_000] {
            assert_eq!(digits(n), json::write(&json::num(n as f64)).len(), "{n}");
        }
    }

    #[test]
    fn diff_is_prefix_suffix() {
        let r = Relocation::diff("m", "abcXdef", "abcYYdef").unwrap();
        assert_eq!((r.edit_start, r.edit_end, r.delta), (3, 4, 1));
        let r = Relocation::diff("m", "Hi\nBye", "Hio\nBye").unwrap();
        assert_eq!((r.edit_start, r.edit_end, r.delta), (2, 2, 1));
        assert!(Relocation::diff("m", "same", "same").is_none());
        let r = Relocation::diff("m", "abcdef", "abef").unwrap();
        assert_eq!((r.edit_start, r.edit_end, r.delta), (2, 4, -2));
    }

    #[test]
    fn zero_pages_have_zero_separators() {
        let l = DisplayList {
            project_id: "p".into(),
            revision: 1,
            documents: vec![],
            fonts: vec![],
            pages: vec![],
            diagnostics: vec![],
        };
        assert_eq!(full_line_bytes(&l, "x", &l.required_features(), &[]).unwrap(), json::write(&l.to_json("x")).len());
    }
}
