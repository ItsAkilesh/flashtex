use flashtex_rendering_core::{wire::*, *};
use serde_json::json;
fn offer() -> Envelope {
    parse_validated(include_bytes!("fixtures/capabilities.json"), None).unwrap()
}
#[test]
fn validated_glyph_and_source_roundtrip_is_canonical_and_lossless() {
    let offer = offer();
    let envelope = parse_validated(
        include_bytes!("fixtures/synthetic-display-list.json"),
        Some(&offer),
    )
    .unwrap();
    let bytes = serialize_validated(&envelope, Some(&offer), MAX_MESSAGE_BYTES).unwrap();
    let decoded = parse_validated(&bytes, Some(&offer)).unwrap();
    assert_eq!(
        bytes,
        serialize_validated(&decoded, Some(&offer), MAX_MESSAGE_BYTES).unwrap()
    );
    if let Message::DisplayList(list) = decoded.message {
        if let Item::GlyphRun(run) = &list.pages[0].items[0] {
            assert_eq!(run.text, "office e\u{301}");
            assert_eq!(run.clusters[0].sources.as_ref().unwrap()[0].end_byte, 10);
            assert_eq!(run.glyphs[0].gid, 1);
        } else {
            panic!()
        }
    } else {
        panic!()
    }
}
#[test]
fn unknown_primitives_and_messages_have_typed_errors() {
    let mut value: serde_json::Value =
        serde_json::from_slice(include_bytes!("fixtures/synthetic-display-list.json")).unwrap();
    value["payload"]["pages"][0]["items"][0]["kind"] = json!("image");
    assert_eq!(
        parse_validated(&serde_json::to_vec(&value).unwrap(), Some(&offer())).unwrap_err(),
        WireError::UnsupportedPrimitive {
            page_index: 0,
            item_index: 0,
            kind: "image".into()
        }
    );
    value["type"] = json!("future_frame");
    assert!(matches!(
        parse_validated(&serde_json::to_vec(&value).unwrap(), Some(&offer())),
        Err(WireError::UnsupportedMessage { .. })
    ));
}
#[test]
fn unsupported_versions_are_not_decoded_as_current() {
    let value = json!({"protocol_version":1,"id":"a","type":"render_capabilities","payload":{}});
    assert_eq!(
        parse_validated(&serde_json::to_vec(&value).unwrap(), None).unwrap_err(),
        WireError::UnsupportedVersion { version: 1 }
    );
}
#[test]
fn serializers_reject_invalid_inmemory_fields_and_never_return_partial_bytes() {
    let offer = offer();
    let mut envelope = parse_validated(
        include_bytes!("fixtures/synthetic-display-list.json"),
        Some(&offer),
    )
    .unwrap();
    assert_eq!(
        serialize_validated(&envelope, Some(&offer), 10).unwrap_err(),
        WireError::TooLarge { limit: 10 }
    );
    if let Message::DisplayList(list) = &mut envelope.message {
        list.pages[0].width = Tick(-1);
    }
    assert!(matches!(
        serialize_validated(&envelope, Some(&offer), MAX_MESSAGE_BYTES),
        Err(WireError::Semantic { .. })
    ));
}
#[test]
fn malformed_truncated_and_missing_offer_are_distinct_failures() {
    assert!(matches!(
        parse_validated(b"{", None),
        Err(WireError::Malformed { .. })
    ));
    assert!(matches!(
        parse_validated(include_bytes!("fixtures/synthetic-display-list.json"), None),
        Err(WireError::Semantic { .. })
    ));
}
#[test]
fn offers_selections_and_explicit_rejections_roundtrip() {
    let offer = offer();
    for (kind, payload) in [
        (
            "render_format_selected",
            json!({"render_format":"display-list-v2","required_features":["glyph_run"]}),
        ),
        (
            "render_format_rejected",
            json!({"code":"unsupported_feature","message":"image is not negotiated"}),
        ),
    ] {
        let bytes = serde_json::to_vec(
            &json!({"protocol_version":2,"id":"hello","type":kind,"payload":payload}),
        )
        .unwrap();
        let message = parse_validated(&bytes, Some(&offer)).unwrap();
        let encoded = serialize_validated(&message, Some(&offer), MAX_MESSAGE_BYTES).unwrap();
        parse_validated(&encoded, Some(&offer)).unwrap();
    }
    let bytes = serialize_validated(&offer, None, MAX_MESSAGE_BYTES).unwrap();
    parse_validated(&bytes, None).unwrap();
}
