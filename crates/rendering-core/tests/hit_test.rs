use flashtex_rendering_core::{hit_test::*, *};
fn setup() -> (DisplayList, Capabilities) {
    let list = match parse(include_bytes!("fixtures/synthetic-display-list.json"))
        .unwrap()
        .message
    {
        Message::DisplayList(list) => list,
        _ => unreachable!(),
    };
    let caps = match parse(include_bytes!("fixtures/capabilities.json"))
        .unwrap()
        .message
    {
        Message::Offer(caps) => caps,
        _ => unreachable!(),
    };
    (list, caps)
}
fn hit(index: &PageIndex, x: i64, y: i64) -> Option<Hit> {
    index
        .hit_test(
            "p",
            2,
            1,
            Point {
                x: Tick(x),
                y: Tick(y),
            },
        )
        .unwrap()
}
#[test]
fn half_open_hit_boxes_preserve_whole_utf8_cluster_sources() {
    let (list, caps) = setup();
    let index = PageIndex::build(&list, &caps).unwrap();
    let selected = hit(&index, 99, 99).unwrap();
    assert_eq!(
        selected.selection,
        LogicalSelection::WholeCluster {
            start_byte: 0,
            end_byte: 10
        }
    );
    assert_eq!(selected.sources[0].path, "main.tex");
    assert_eq!(selected.sources[0].end_byte, 10);
    assert!(hit(&index, 100, 50).is_none());
    assert!(hit(&index, 50, 100).is_none());
    assert!(hit(&index, -1, 50).is_none());
}
#[test]
fn exact_carets_select_without_fabricating_tex_source_offsets() {
    let (mut list, caps) = setup();
    if let Item::GlyphRun(run) = &mut list.pages[0].items[0] {
        run.clusters[0].carets = vec![
            Caret {
                text_byte: 0,
                x: Tick(0),
                top: Tick(0),
                height: Tick(100),
            },
            Caret {
                text_byte: 10,
                x: Tick(99),
                top: Tick(0),
                height: Tick(100),
            },
        ];
    }
    let index = PageIndex::build(&list, &caps).unwrap();
    let selected = hit(&index, 98, 50).unwrap();
    assert_eq!(
        selected.selection,
        LogicalSelection::Caret { text_byte: 10 }
    );
    assert_eq!(selected.sources[0].start_byte, 0);
    assert_eq!(selected.sources[0].end_byte, 10);
}
#[test]
fn later_paint_order_wins_and_rules_are_explicit() {
    let (mut list, caps) = setup();
    list.pages[0].items.push(Item::Rule(Rule {
        x: Tick(0),
        top: Tick(0),
        width: Tick(50),
        height: Tick(50),
        paint: Paint {
            r: 0.,
            g: 0.,
            b: 0.,
            a: 1.,
        },
        sources: None,
        synthetic_reason: Some("fraction rule".into()),
    }));
    let index = PageIndex::build(&list, &caps).unwrap();
    let selected = hit(&index, 10, 10).unwrap();
    assert_eq!(selected.selection, LogicalSelection::Rule);
    assert!(selected.sources.is_empty());
    assert_eq!(selected.synthetic_reason.as_deref(), Some("fraction rule"));
    assert!(matches!(
        hit(&index, 75, 75).unwrap().selection,
        LogicalSelection::WholeCluster { .. }
    ));
}
#[test]
fn spatial_index_handles_unsorted_and_overlapping_vertical_extents() {
    let (mut list, caps) = setup();
    if let Item::GlyphRun(run) = &mut list.pages[0].items[0] {
        run.clusters[0].hit_rects = vec![
            HitRect {
                x: Tick(0),
                top: Tick(300),
                width: Tick(100),
                height: Tick(100),
            },
            HitRect {
                x: Tick(0),
                top: Tick(0),
                width: Tick(100),
                height: Tick(200),
            },
            HitRect {
                x: Tick(0),
                top: Tick(100),
                width: Tick(10),
                height: Tick(5),
            },
        ];
    }
    let index = PageIndex::build(&list, &caps).unwrap();
    assert!(hit(&index, 50, 150).is_some());
    assert!(hit(&index, 50, 250).is_none());
    assert!(hit(&index, 50, 350).is_some());
}
#[test]
fn stale_revision_and_unknown_page_are_rejected() {
    let (list, caps) = setup();
    let index = PageIndex::build(&list, &caps).unwrap();
    let p = Point {
        x: Tick(0),
        y: Tick(0),
    };
    assert!(index.hit_test("p", 3, 1, p).is_err());
    assert!(index.hit_test("other", 2, 1, p).is_err());
    assert!(index.hit_test("p", 2, 2, p).is_err());
}
#[test]
fn zero_area_and_outside_page_geometry_are_not_selectable() {
    let (mut list, caps) = setup();
    if let Item::GlyphRun(run) = &mut list.pages[0].items[0] {
        run.clusters[0].hit_rects = vec![
            HitRect {
                x: Tick(0),
                top: Tick(0),
                width: Tick(0),
                height: Tick(100),
            },
            HitRect {
                x: Tick(-100),
                top: Tick(-100),
                width: Tick(100),
                height: Tick(100),
            },
        ];
    }
    let index = PageIndex::build(&list, &caps).unwrap();
    assert!(hit(&index, 0, 0).is_none());
}
