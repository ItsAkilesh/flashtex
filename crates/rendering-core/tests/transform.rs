use flashtex_rendering_core::{hit_test::*, transform::*, *};
fn p(x: i64, y: i64) -> ExactPoint {
    ExactPoint::from_point(Point {
        x: Tick(x),
        y: Tick(y),
    })
    .unwrap()
}
fn setup() -> (DisplayList, Capabilities) {
    let list = match parse(include_bytes!("fixtures/synthetic-display-list.json"))
        .unwrap()
        .message
    {
        Message::DisplayList(l) => l,
        _ => unreachable!(),
    };
    let caps = match parse(include_bytes!("fixtures/capabilities.json"))
        .unwrap()
        .message
    {
        Message::Offer(c) => c,
        _ => unreachable!(),
    };
    (list, caps)
}
#[test]
fn exact_forward_inverse_preserves_fractional_coordinates() {
    for axis in [YAxis::Down, YAxis::Up] {
        for (n, d) in [(1, 1), (3, 2), (2, 7), (7, 3)] {
            let transform = ViewportTransform::new(
                n,
                d,
                Point {
                    x: Tick(37),
                    y: Tick(-25),
                },
                axis,
            )
            .unwrap();
            for x in [-100, 0, 3, 111] {
                for y in [-13, 0, 200] {
                    let point = p(x, y);
                    assert_eq!(
                        transform
                            .inverse(transform.forward(point).unwrap())
                            .unwrap(),
                        point
                    );
                }
            }
        }
    }
}
#[test]
fn rational_caret_query_is_not_rounded_to_integer_pixel() {
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
                x: Tick(1),
                top: Tick(0),
                height: Tick(100),
            },
        ];
    }
    let index = PageIndex::build(&list, &caps).unwrap();
    let transform = ViewportTransform::new(
        3,
        2,
        Point {
            x: Tick(20),
            y: Tick(10),
        },
        YAxis::Down,
    )
    .unwrap();
    let view = ExactPoint {
        x: RationalTick::new(209, 10).unwrap(),
        y: RationalTick::from_tick(Tick(25)).unwrap(),
    };
    let hit = transform
        .hit_test(&index, "p", 2, 1, view)
        .unwrap()
        .unwrap();
    assert_eq!(hit.selection, LogicalSelection::Caret { text_byte: 10 });
    assert_eq!(hit.sources[0].start_byte, 0);
}
#[test]
fn flipped_clip_preserves_source_boundary_inclusion() {
    let rect = HitRect {
        x: Tick(0),
        top: Tick(0),
        width: Tick(100),
        height: Tick(100),
    };
    let clip = HitRect {
        x: Tick(10),
        top: Tick(20),
        width: Tick(50),
        height: Tick(30),
    };
    for axis in [YAxis::Down, YAxis::Up] {
        let transform = ViewportTransform::new(
            3,
            2,
            Point {
                x: Tick(0),
                y: Tick(200),
            },
            axis,
        )
        .unwrap();
        let visible = transform.visible_bounds(&rect, &clip).unwrap().unwrap();
        assert!(visible.contains(transform.forward(p(10, 20)).unwrap()));
        assert!(!visible.contains(transform.forward(p(60, 20)).unwrap()));
        assert!(!visible.contains(transform.forward(p(10, 50)).unwrap()));
        assert!(visible.contains(transform.forward(p(59, 49)).unwrap()));
    }
}
#[test]
fn empty_clip_and_overflow_fail_without_approximation() {
    let transform = ViewportTransform::new(
        1,
        1,
        Point {
            x: Tick(0),
            y: Tick(0),
        },
        YAxis::Down,
    )
    .unwrap();
    let a = HitRect {
        x: Tick(0),
        top: Tick(0),
        width: Tick(10),
        height: Tick(10),
    };
    let b = HitRect {
        x: Tick(10),
        top: Tick(0),
        width: Tick(10),
        height: Tick(10),
    };
    assert!(transform.visible_bounds(&a, &b).unwrap().is_none());
    assert!(ViewportTransform::new(
        0,
        1,
        Point {
            x: Tick(0),
            y: Tick(0)
        },
        YAxis::Down
    )
    .is_err());
    assert!(RationalTick::new(1, 0).is_err());
    assert!(RationalTick::new(1, 1_000_001).is_err());
    let huge = ViewportTransform::new(
        2,
        1,
        Point {
            x: Tick(0),
            y: Tick(0),
        },
        YAxis::Down,
    )
    .unwrap();
    assert!(huge.forward(p(MAX_EXACT_INTEGER, 0)).is_err());
}
#[test]
fn transformed_hits_and_source_hits_have_same_provenance() {
    let (list, caps) = setup();
    let index = PageIndex::build(&list, &caps).unwrap();
    for axis in [YAxis::Down, YAxis::Up] {
        let transform = ViewportTransform::new(
            7,
            3,
            Point {
                x: Tick(50),
                y: Tick(1000),
            },
            axis,
        )
        .unwrap();
        for x in [-1, 0, 50, 99, 100] {
            let source = index
                .hit_test(
                    "p",
                    2,
                    1,
                    Point {
                        x: Tick(x),
                        y: Tick(50),
                    },
                )
                .unwrap();
            let view = transform
                .hit_test(&index, "p", 2, 1, transform.forward(p(x, 50)).unwrap())
                .unwrap();
            assert_eq!(
                source.as_ref().map(|h| h.selection.clone()),
                view.as_ref().map(|h| h.selection.clone())
            );
        }
    }
}
