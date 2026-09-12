use flashtex_font_resources::{Coordinate, ExactPoint, ExpandedOutline, GlyphInstance};
use flashtex_rendering_core::{hit_test::Point, outlines::*, *};
fn point(x: i32, y: i32, on_curve: bool) -> ExactPoint {
    ExactPoint {
        x: Coordinate::from_integer(x),
        y: Coordinate::from_integer(y),
        on_curve,
    }
}
fn outline(points: Vec<ExactPoint>) -> ExpandedOutline {
    ExpandedOutline {
        font_id: "synthetic".into(),
        font_sha256: "a".repeat(64),
        face_index: 0,
        glyph_id: 1,
        contour_ends: vec![points.len() as u32 - 1],
        instances: vec![GlyphInstance {
            glyph_id: 1,
            point_start: 0,
            point_count: points.len() as u32,
        }],
        points,
    }
}
#[test]
fn design_units_are_scaled_and_baseline_flipped_exactly() {
    let p = position_font_point(
        point(333, 500, true),
        Tick(12 * TICKS_PER_BP),
        1000,
        Point {
            x: Tick(72 * TICKS_PER_BP),
            y: Tick(84 * TICKS_PER_BP),
        },
    )
    .unwrap();
    assert_eq!(
        p.x.numerator() * 1000,
        (72i128 * 1000 + 333 * 12) * i128::from(TICKS_PER_BP) * p.x.denominator() as i128
    );
    assert_eq!(p.y.numerator(), 78 * i128::from(TICKS_PER_BP));
    assert_eq!(p.y.denominator(), 1);
}
#[test]
fn contour_path_closure_and_quadratic_implied_midpoints_are_retained() {
    let shape = outline(vec![
        point(0, 0, false),
        point(100, 200, false),
        point(200, 0, true),
    ]);
    let commands = place_outline(
        &shape,
        Tick(1000),
        1000,
        Point {
            x: Tick(0),
            y: Tick(0),
        },
    )
    .unwrap();
    assert!(matches!(
        commands.first(),
        Some(PlacedPathCommand::MoveTo(_))
    ));
    assert_eq!(commands.last(), Some(&PlacedPathCommand::Close));
    let implied = commands
        .iter()
        .find_map(|c| {
            if let PlacedPathCommand::QuadTo { end, .. } = c {
                Some(end)
            } else {
                None
            }
        })
        .unwrap();
    assert_eq!(implied.x.numerator(), 50);
    assert_eq!(implied.y.numerator(), -100);
}
#[test]
fn dyadic_design_units_never_round_to_integer_ticks() {
    let a = Coordinate::from_integer(1);
    let half = a.midpoint(Coordinate::from_integer(0)).unwrap();
    let p = position_font_point(
        ExactPoint {
            x: half,
            y: half,
            on_curve: true,
        },
        Tick(1),
        1000,
        Point {
            x: Tick(0),
            y: Tick(0),
        },
    )
    .unwrap();
    assert_eq!((p.x.numerator(), p.x.denominator()), (1, 2000));
    assert_eq!((p.y.numerator(), p.y.denominator()), (-1, 2000));
}
#[test]
fn empty_outline_is_empty_and_invalid_geometry_errors() {
    let empty = ExpandedOutline {
        font_id: "synthetic".into(),
        font_sha256: "a".repeat(64),
        face_index: 0,
        glyph_id: 1,
        points: vec![],
        contour_ends: vec![],
        instances: vec![],
    };
    assert!(place_outline(
        &empty,
        Tick(1),
        1000,
        Point {
            x: Tick(0),
            y: Tick(0)
        }
    )
    .unwrap()
    .is_empty());
    assert!(place_outline(
        &empty,
        Tick(0),
        1000,
        Point {
            x: Tick(0),
            y: Tick(0)
        }
    )
    .is_err());
    assert!(position_font_point(
        point(32767, 0, true),
        Tick(MAX_EXACT_INTEGER),
        16,
        Point {
            x: Tick(0),
            y: Tick(0)
        }
    )
    .is_err());
}
