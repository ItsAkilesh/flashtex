//! Original ligature GIDs, source intervals and declared text stay separate.
use flashtex_pdf::{exact::parse_to_unicode, reader::PdfFile};
use serde_json::Value;
#[test]
fn actual_ligatures_preserve_original_gids_and_multi_byte_source_clusters() {
    let display: Value =
        serde_json::from_slice(include_bytes!("fixtures/ligatures-reference/display.json"))
            .unwrap();
    let request: Value =
        serde_json::from_slice(include_bytes!("fixtures/ligatures-reference/request.jsonl"))
            .unwrap();
    assert_eq!(display["payload"]["diagnostics"], serde_json::json!([]));
    let source = request["payload"]["documents"][0]["text"].as_str().unwrap();
    let items = display["payload"]["pages"][0]["items"].as_array().unwrap();
    let file = PdfFile::parse(include_bytes!("fixtures/ligatures-reference/original.pdf")).unwrap();
    let pages = file.pages().unwrap();
    let fonts = file.page_fonts(pages[0]);
    let cmap = file
        .decode_stream(file.resolve(&fonts["F1"]["ToUnicode"]))
        .unwrap();
    let unicode = parse_to_unicode(&cmap).unwrap();
    for (text, gid) in [("ffi", 123), ("fl", 126), ("ff", 122), ("fi", 125)] {
        let item = items.iter().find(|i| i["text"] == text).unwrap();
        assert_eq!(item["glyphs"].as_array().unwrap().len(), 1);
        assert_eq!(item["glyphs"][0]["gid"], gid);
        let span = &item["clusters"][0]["sources"][0];
        assert_eq!(
            &source[span["start_byte"].as_u64().unwrap() as usize
                ..span["end_byte"].as_u64().unwrap() as usize],
            text
        );
        assert_eq!(unicode.get(&(gid as u16)).map(String::as_str), Some(text));
    }
    let waffle = items.iter().find(|i| i["text"] == "waffle,").unwrap();
    assert_eq!(waffle["glyphs"][2]["gid"], 124);
    let span = &waffle["clusters"][2]["sources"][0];
    assert_eq!(
        &source[span["start_byte"].as_u64().unwrap() as usize
            ..span["end_byte"].as_u64().unwrap() as usize],
        "ffl"
    );
    assert_eq!(unicode.get(&124).map(String::as_str), Some("ffl"));
    let reference =
        PdfFile::parse(include_bytes!("fixtures/ligatures-reference/reference.pdf")).unwrap();
    let pages = reference.pages().unwrap();
    for font in reference.page_fonts(pages[0]).values() {
        let cmap = reference
            .decode_stream(reference.resolve(&font["ToUnicode"]))
            .unwrap();
        let values = parse_to_unicode(&cmap).unwrap();
        for (code, text) in [(27, "ff"), (28, "fi"), (29, "fl"), (30, "ffi"), (31, "ffl")] {
            assert_eq!(values.get(&code).map(String::as_str), Some(text));
        }
    }
    assert_eq!(
        include_bytes!("fixtures/ligatures-reference/original.txt"),
        include_bytes!("fixtures/ligatures-reference/reference.txt")
    );
}
