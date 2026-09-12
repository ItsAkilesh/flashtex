//! Exact, bounded comparison of validated experimental geometry. No tolerance,
//! coordinate normalization, font substitution or cross-format equivalence guess.
use crate::{batch::PrimitiveId, mixed::MixedLimits, mixed_replay::ReplayBatch, *};
use serde::Serialize;
use serde_json::{json, Value};
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum InputKind {
    Mixed,
    Display,
}
pub struct ValidatedGeometry {
    kind: InputKind,
    raw_sha256: String,
    offer_sha256: Option<String>,
    value: Value,
}
impl ValidatedGeometry {
    pub fn mixed(bytes: &[u8]) -> Result<Self> {
        let replay = ReplayBatch::parse(bytes, MixedLimits::default())
            .map_err(|e| ValidationError(format!("invalid mixed comparison input: {e:?}")))?;
        Ok(Self {
            kind: InputKind::Mixed,
            raw_sha256: digest(bytes),
            offer_sha256: None,
            value: replay.metadata().clone(),
        })
    }
    pub fn display(bytes: &[u8], offer_bytes: &[u8]) -> Result<Self> {
        require(
            offer_bytes.len() <= MAX_MESSAGE_BYTES,
            "comparison offer byte budget",
        )?;
        crate::mixed_replay::parse_unique(offer_bytes)
            .map_err(|e| ValidationError(e.to_string()))?;
        let offer = crate::wire::parse_validated(offer_bytes, None)
            .map_err(|e| ValidationError(format!("invalid comparison offer: {e}")))?;
        let envelope = crate::wire::parse_validated(bytes, Some(&offer))
            .map_err(|e| ValidationError(format!("invalid display comparison input: {e}")))?;
        require(
            matches!(envelope.message, Message::DisplayList(_)),
            "comparison requires display list",
        )?;
        let value =
            crate::mixed_replay::parse_unique(bytes).map_err(|e| ValidationError(e.to_string()))?;
        Ok(Self {
            kind: InputKind::Display,
            raw_sha256: digest(bytes),
            offer_sha256: Some(digest(offer_bytes)),
            value,
        })
    }
}
#[derive(Debug, Clone, Copy)]
pub struct DiffLimits {
    pub max_differences: usize,
    pub max_report_bytes: usize,
    pub max_visited_nodes: usize,
}
impl Default for DiffLimits {
    fn default() -> Self {
        Self {
            max_differences: 1000,
            max_report_bytes: 1024 * 1024,
            max_visited_nodes: 1_000_000,
        }
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Category {
    FontResourceOrGid,
    AdvanceOrPosition,
    BaselineOrRule,
    SourceProvenance,
    Membership,
    Other,
}
#[derive(Debug, Clone, Serialize)]
pub struct ExactDelta {
    pub numerator: String,
    pub denominator: String,
}
#[derive(Debug, Serialize)]
pub struct Difference {
    pub category: Category,
    pub path: String,
    pub page: Option<u32>,
    pub primitive: Option<PrimitiveId>,
    pub left: Value,
    pub right: Value,
    pub delta: Option<ExactDelta>,
    pub delta_unsupported: bool,
}
#[derive(Debug, Serialize)]
pub struct DiffReport {
    pub left_sha256: String,
    pub right_sha256: String,
    pub left_offer_sha256: Option<String>,
    pub right_offer_sha256: Option<String>,
    pub left_kind: InputKind,
    pub right_kind: InputKind,
    pub equal: Option<bool>,
    pub truncated: bool,
    pub unsupported: bool,
    pub visited_nodes: usize,
    pub differences: Vec<Difference>,
    pub resources_verified: bool,
}
impl DiffReport {
    pub fn json_bytes(&self) -> Result<Vec<u8>> {
        serde_json::to_vec(self).map_err(|e| ValidationError(e.to_string()))
    }
}
#[derive(Clone, Copy, Default)]
struct Location {
    page: Option<u32>,
    primitive: Option<PrimitiveId>,
    rule: bool,
}
struct Walker {
    report: DiffReport,
    limits: DiffLimits,
    record_bytes: usize,
}
pub fn compare(
    left: &ValidatedGeometry,
    right: &ValidatedGeometry,
    limits: DiffLimits,
) -> Result<DiffReport> {
    require(
        (1..=10000).contains(&limits.max_differences)
            && (4096..=MAX_MESSAGE_BYTES).contains(&limits.max_report_bytes)
            && (1..=2_000_000).contains(&limits.max_visited_nodes),
        "invalid diff budget",
    )?;
    let mut walker = Walker {
        report: DiffReport {
            left_sha256: left.raw_sha256.clone(),
            right_sha256: right.raw_sha256.clone(),
            left_offer_sha256: left.offer_sha256.clone(),
            right_offer_sha256: right.offer_sha256.clone(),
            left_kind: left.kind,
            right_kind: right.kind,
            equal: None,
            truncated: false,
            unsupported: false,
            visited_nodes: 0,
            differences: vec![],
            resources_verified: false,
        },
        limits,
        record_bytes: 0,
    };
    if left.kind != right.kind {
        walker.report.unsupported = true;
    } else {
        walker.walk("", &left.value, &right.value, Location::default());
    }
    if !walker.report.truncated && !walker.report.unsupported {
        walker.report.equal = Some(walker.report.differences.is_empty());
    }
    require(
        walker.report.json_bytes()?.len() <= limits.max_report_bytes,
        "diff report budget invariant",
    )?;
    Ok(walker.report)
}
impl Walker {
    fn stopped(&self) -> bool {
        self.report.truncated
    }
    fn record(
        &mut self,
        path: &str,
        left: &Value,
        right: &Value,
        location: Location,
        category: Option<Category>,
    ) {
        if self.stopped() {
            return;
        }
        if self.report.differences.len() >= self.limits.max_differences {
            self.report.truncated = true;
            return;
        }
        let category = category.unwrap_or_else(|| classify(path, location.rule));
        let (delta, delta_unsupported) = if matches!(
            category,
            Category::AdvanceOrPosition | Category::BaselineOrRule
        ) {
            match delta(left, right) {
                Ok(v) => (v, false),
                Err(()) => (None, true),
            }
        } else {
            (None, false)
        };
        self.report.unsupported |= delta_unsupported;
        let difference = Difference {
            category,
            path: path.into(),
            page: location.page,
            primitive: location.primitive,
            left: left.clone(),
            right: right.clone(),
            delta,
            delta_unsupported,
        };
        let mut output = crate::mixed::BoundedOutput {
            bytes: vec![],
            limit: self
                .limits
                .max_report_bytes
                .saturating_sub(1024 + self.record_bytes),
        };
        if serde_json::to_writer(&mut output, &difference).is_err() {
            self.report.truncated = true;
            return;
        }
        self.record_bytes += output.bytes.len() + 1;
        self.report.differences.push(difference);
    }
    fn walk(&mut self, path: &str, left: &Value, right: &Value, mut location: Location) {
        if self.stopped() {
            return;
        }
        if self.report.visited_nodes >= self.limits.max_visited_nodes {
            self.report.truncated = true;
            return;
        }
        self.report.visited_nodes += 1;
        if left == right && !left.is_array() && !left.is_object() {
            return;
        }
        if let Some(page) = left
            .get("page")
            .and_then(Value::as_u64)
            .or_else(|| left.get("number").and_then(Value::as_u64))
        {
            location.page = u32::try_from(page).ok();
        }
        if let Some(id) = left.get("identity") {
            location.primitive = primitive_id(id);
        }
        location.rule |= left.get("kind").and_then(Value::as_str) == Some("rule")
            || left
                .get("geometry")
                .and_then(|g| g.get("kind"))
                .and_then(Value::as_str)
                == Some("rule");
        if rational(left).is_some() && rational(right).is_some() {
            if left == right {
                return;
            }
            self.record(path, left, right, location, None);
            return;
        }
        match (left, right) {
            (Value::Object(a), Value::Object(b)) => {
                let keys: BTreeSet<_> = a.keys().chain(b.keys()).collect();
                for key in keys {
                    if self.stopped() {
                        break;
                    }
                    let next = format!("{path}/{}", key.replace('~', "~0").replace('/', "~1"));
                    match (a.get(key), b.get(key)) {
                        (Some(a), Some(b)) => self.walk(&next, a, b, location),
                        (a, b) => self.record(
                            &next,
                            a.unwrap_or(&Value::Null),
                            b.unwrap_or(&Value::Null),
                            location,
                            Some(Category::Membership),
                        ),
                    }
                }
            }
            (Value::Array(a), Value::Array(b)) => {
                if path.ends_with("/primitives") {
                    self.primitives(path, a, b, location);
                    return;
                }
                let length = a.len().max(b.len());
                for index in 0..length {
                    if self.stopped() {
                        break;
                    }
                    let next = format!("{path}/{index}");
                    let mut loc = location;
                    if path.ends_with("/items") {
                        loc.primitive = Some(PrimitiveId {
                            item_index: index,
                            glyph_index: None,
                        });
                    }
                    if path.ends_with("/glyphs") {
                        if let Some(id) = &mut loc.primitive {
                            id.glyph_index = Some(index);
                        }
                    }
                    match (a.get(index), b.get(index)) {
                        (Some(a), Some(b)) => self.walk(&next, a, b, loc),
                        (a, b) => self.record(
                            &next,
                            a.unwrap_or(&Value::Null),
                            b.unwrap_or(&Value::Null),
                            loc,
                            Some(Category::Membership),
                        ),
                    }
                }
            }
            _ => self.record(path, left, right, location, None),
        }
    }
    fn primitives(&mut self, path: &str, left: &[Value], right: &[Value], location: Location) {
        let ids = |v: &[Value]| v.iter().map(|p| p["identity"].clone()).collect::<Vec<_>>();
        let a_order = ids(left);
        let b_order = ids(right);
        if a_order != b_order {
            self.record(
                &format!("{path}/order"),
                &json!(a_order),
                &json!(b_order),
                location,
                Some(Category::Membership),
            );
        }
        let a = keyed(left);
        let b = keyed(right);
        let keys: BTreeSet<_> = a.keys().chain(b.keys()).copied().collect();
        for id in keys {
            if self.stopped() {
                break;
            }
            let path = format!(
                "{path}/item:{}/glyph:{}",
                id.item_index,
                id.glyph_index
                    .map_or_else(|| "none".into(), |n| n.to_string())
            );
            let loc = Location {
                primitive: Some(id),
                ..location
            };
            match (a.get(&id), b.get(&id)) {
                (Some(a), Some(b)) => self.walk(&path, a, b, loc),
                (a, b) => self.record(
                    &path,
                    a.copied().unwrap_or(&Value::Null),
                    b.copied().unwrap_or(&Value::Null),
                    loc,
                    Some(Category::Membership),
                ),
            }
        }
    }
}
fn primitive_id(v: &Value) -> Option<PrimitiveId> {
    Some(PrimitiveId {
        item_index: usize::try_from(v["item_index"].as_u64()?).ok()?,
        glyph_index: if v["glyph_index"].is_null() {
            None
        } else {
            Some(usize::try_from(v["glyph_index"].as_u64()?).ok()?)
        },
    })
}
fn classify(path: &str, rule: bool) -> Category {
    if [
        "source",
        "documents",
        "logical",
        "synthetic",
        "text",
        "cluster",
    ]
    .iter()
    .any(|s| path.contains(s))
    {
        return Category::SourceProvenance;
    }
    if [
        "font",
        "cff",
        "gid",
        "encoding",
        "units_per_em",
        "glyph_count",
    ]
    .iter()
    .any(|s| path.contains(s))
    {
        return Category::FontResourceOrGid;
    }
    if rule || path.contains("baseline") || path.contains("/top") || path.contains("/height") {
        return Category::BaselineOrRule;
    }
    if [
        "advance", "position", "origin", "commands", "bounds", "clip", "/x", "/y", "width",
    ]
    .iter()
    .any(|s| path.contains(s))
    {
        return Category::AdvanceOrPosition;
    }
    if ["/pages", "/page", "/identity", "/kind", "/order"]
        .iter()
        .any(|s| path.contains(s))
    {
        return Category::Membership;
    }
    Category::Other
}
fn rational(v: &Value) -> Option<(i128, u128)> {
    let a = v.as_array()?;
    if a.len() != 2 {
        return None;
    }
    Some((a[0].as_str()?.parse().ok()?, a[1].as_str()?.parse().ok()?))
}
fn gcd(mut a: u128, mut b: u128) -> u128 {
    while b != 0 {
        let r = a % b;
        a = b;
        b = r;
    }
    a
}
fn delta(left: &Value, right: &Value) -> std::result::Result<Option<ExactDelta>, ()> {
    let a = rational(left).or_else(|| left.as_i64().map(|n| (n as i128, 1)));
    let b = rational(right).or_else(|| right.as_i64().map(|n| (n as i128, 1)));
    let (Some((a, ad)), Some((b, bd))) = (a, b) else {
        return Ok(None);
    };
    if ad == 0 || bd == 0 || ad > i128::MAX as u128 || bd > i128::MAX as u128 {
        return Err(());
    }
    let divisor = gcd(ad, bd);
    let af = bd / divisor;
    let bf = ad / divisor;
    let n = b
        .checked_mul(bf as i128)
        .and_then(|b| a.checked_mul(af as i128).and_then(|a| b.checked_sub(a)))
        .ok_or(())?;
    let d = ad.checked_mul(af).ok_or(())?;
    let divisor = gcd(n.unsigned_abs(), d);
    Ok(Some(ExactDelta {
        numerator: (n / i128::try_from(divisor).map_err(|_| ())?).to_string(),
        denominator: (d / divisor).to_string(),
    }))
}

fn keyed(values: &[Value]) -> BTreeMap<PrimitiveId, &Value> {
    values
        .iter()
        .filter_map(|p| primitive_id(&p["identity"]).map(|id| (id, p)))
        .collect()
}
