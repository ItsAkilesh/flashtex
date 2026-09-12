use flashtex_rendering_core::{digest, geometry_diff::*};
use serde_json::{json, Value};
fn fixture() -> Value {
    serde_json::from_slice(include_bytes!("fixtures/synthetic-mixed.json")).unwrap()
}
fn input(v: &Value) -> ValidatedGeometry {
    ValidatedGeometry::mixed(&serde_json::to_vec(v).unwrap()).unwrap()
}
#[test]
fn exact_position_delta_and_provenance_keep_stable_id_and_raw_hashes() {
    let a = fixture();
    let mut b = a.clone();
    b["primitives"][0]["geometry"]["commands"][0][1][0] = json!(["2", "3"]);
    b["primitives"][1]["synthetic_reason"] = "changed source".into();
    let report = compare(&input(&a), &input(&b), DiffLimits::default()).unwrap();
    assert_eq!(report.equal, Some(false));
    assert_eq!(report.left_sha256, digest(&serde_json::to_vec(&a).unwrap()));
    let position = report
        .differences
        .iter()
        .find(|d| d.category == Category::AdvanceOrPosition)
        .unwrap();
    assert_eq!(position.primitive.unwrap().item_index, 0);
    assert_eq!(position.page, Some(1));
    assert_eq!(position.delta.as_ref().unwrap().numerator, "1");
    assert_eq!(position.delta.as_ref().unwrap().denominator, "3");
    assert!(report
        .differences
        .iter()
        .any(|d| d.category == Category::SourceProvenance));
}
#[test]
fn reordering_reports_membership_without_pairing_unrelated_glyphs() {
    let a = fixture();
    let mut b = a.clone();
    b["primitives"].as_array_mut().unwrap().swap(0, 1);
    let report = compare(&input(&a), &input(&b), DiffLimits::default()).unwrap();
    assert_eq!(report.differences.len(), 1);
    assert_eq!(report.differences[0].category, Category::Membership);
    assert!(report.differences[0].path.ends_with("/order"));
}
#[test]
fn resource_gid_rule_and_membership_are_classified() {
    let a = fixture();
    let mut b = a.clone();
    b["primitives"][0]["font_sha256"] = "3".repeat(64).into();
    b["primitives"][0]["original_gid"] = 2.into();
    b["primitives"][2]["geometry"]["bounds"][0] = json!(["2", "3"]);
    let report = compare(&input(&a), &input(&b), DiffLimits::default()).unwrap();
    assert_eq!(
        report
            .differences
            .iter()
            .filter(|d| d.category == Category::FontResourceOrGid)
            .count(),
        2
    );
    let rule = report
        .differences
        .iter()
        .find(|d| d.category == Category::BaselineOrRule)
        .unwrap();
    assert_eq!(rule.delta.as_ref().unwrap().numerator, "1");
    assert_eq!(rule.delta.as_ref().unwrap().denominator, "6");
}
#[test]
fn bounded_or_unsupported_comparisons_never_claim_equality() {
    let a = fixture();
    let same = compare(
        &input(&a),
        &input(&a),
        DiffLimits {
            max_visited_nodes: 1,
            ..DiffLimits::default()
        },
    )
    .unwrap();
    assert!(same.truncated);
    assert_eq!(same.equal, None);
    assert_eq!(same.visited_nodes, 1);
    let mut b = a.clone();
    b["primitives"][0]["synthetic_reason"] = "x".into();
    b["primitives"][1]["synthetic_reason"] = "y".into();
    let count = compare(
        &input(&a),
        &input(&b),
        DiffLimits {
            max_differences: 1,
            ..DiffLimits::default()
        },
    )
    .unwrap();
    assert!(count.truncated);
    assert_eq!(count.equal, None);
    assert_eq!(count.differences.len(), 1);
    let mut a = a;
    a["primitives"][0]["geometry"]["commands"][0][1][0] = json!(["1", (1u128 << 126).to_string()]);
    b["primitives"][0]["geometry"]["commands"][0][1][0] =
        json!(["1", ((1u128 << 126) - 1).to_string()]);
    let report = compare(&input(&a), &input(&b), DiffLimits::default()).unwrap();
    assert!(report.unsupported);
    assert_eq!(report.equal, None);
    assert!(report.differences.iter().any(|d| d.delta_unsupported));
}
#[test]
fn oversized_report_value_is_truncated_at_strict_byte_cap() {
    let a = fixture();
    let mut b = a.clone();
    b["primitives"][0]["synthetic_reason"] = "x".repeat(4096).into();
    let report = compare(
        &input(&a),
        &input(&b),
        DiffLimits {
            max_report_bytes: 4096,
            ..DiffLimits::default()
        },
    )
    .unwrap();
    assert!(report.truncated);
    assert_eq!(report.equal, None);
    assert!(report.json_bytes().unwrap().len() <= 4096);
}
#[test]
fn validated_display_inputs_report_exact_origin_and_baseline_changes() {
    let raw = include_bytes!("fixtures/synthetic-display-list.json");
    let offer = include_bytes!("fixtures/capabilities.json");
    let a = ValidatedGeometry::display(raw, offer).unwrap();
    let mut value: Value = serde_json::from_slice(raw).unwrap();
    let glyph = &mut value["payload"]["pages"][0]["items"][0]["glyphs"][0];
    let x = glyph["origin_x"].as_i64().unwrap();
    let y = glyph["baseline_y"].as_i64().unwrap();
    glyph["origin_x"] = (x + 1).into();
    glyph["baseline_y"] = (y + 2).into();
    let b = ValidatedGeometry::display(&serde_json::to_vec(&value).unwrap(), offer).unwrap();
    let report = compare(&a, &b, DiffLimits::default()).unwrap();
    assert_eq!(report.equal, Some(false));
    assert!(report
        .differences
        .iter()
        .any(|d| d.category == Category::BaselineOrRule
            && d.delta.as_ref().unwrap().numerator == "2"));
    assert!(report
        .differences
        .iter()
        .any(|d| d.category == Category::AdvanceOrPosition
            && d.delta.as_ref().unwrap().numerator == "1"));
    let mixed = input(&fixture());
    let report = compare(&a, &mixed, DiffLimits::default()).unwrap();
    assert!(report.unsupported);
    assert_eq!(report.equal, None);
}
