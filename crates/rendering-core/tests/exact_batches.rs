use flashtex_font_resources::{Coordinate, ExactPoint, PathCommand};
use flashtex_rendering_core::{batch::ExactClip, outlines::*, *};
fn r(n: i128, d: u128) -> OutlineCoordinate {
    OutlineCoordinate::from_fraction(n, d).unwrap()
}
#[test]
fn rational_size_and_origin_are_placed_without_rounding() {
    let point = ExactPoint {
        on_curve: true,
        x: Coordinate::from_integer(100),
        y: Coordinate::from_integer(200),
    };
    let commands = place_path_exact(
        [PathCommand::MoveTo(point)],
        r(1, 3),
        1000,
        OutlinePoint {
            x: r(1, 2),
            y: r(7, 4),
        },
    )
    .unwrap();
    let PlacedPathCommand::MoveTo(p) = commands[0] else {
        panic!()
    };
    assert_eq!(p.x, r(8, 15));
    assert_eq!(p.y, r(101, 60));
}
#[test]
fn fractional_clip_preserves_half_open_edges_and_exact_intersection() {
    let c = ExactClip {
        left: r(1, 3),
        top: r(1, 4),
        right: r(2, 3),
        bottom: r(3, 4),
    };
    assert!(c
        .contains(OutlinePoint {
            x: r(1, 3),
            y: r(1, 4)
        })
        .unwrap());
    assert!(!c
        .contains(OutlinePoint {
            x: r(2, 3),
            y: r(1, 2)
        })
        .unwrap());
    let other = ExactClip {
        left: r(1, 2),
        top: r(0, 1),
        right: r(1, 1),
        bottom: r(1, 2),
    };
    let intersection = c.intersect(other).unwrap().unwrap();
    assert_eq!(
        intersection,
        ExactClip {
            left: r(1, 2),
            top: r(1, 4),
            right: r(2, 3),
            bottom: r(1, 2)
        }
    );
    let integer = ExactClip::from_rect(&HitRect {
        x: Tick(0),
        top: Tick(0),
        width: Tick(1),
        height: Tick(1),
    })
    .unwrap();
    assert_eq!(c.intersect(integer).unwrap(), Some(c));
}
#[test]
fn precision_overflow_and_nonpositive_size_are_explicit() {
    assert!(OutlineCoordinate::from_fraction(1, 0).is_err());
    assert!(OutlineCoordinate::from_fraction(1, 1u128 << 127).is_err());
    let tiny = r(1, 1u128 << 100);
    assert!(tiny.checked_multiply(tiny).is_err());
    let point = ExactPoint {
        on_curve: true,
        x: Coordinate::from_integer(1),
        y: Coordinate::from_integer(1),
    };
    assert!(place_path_exact(
        [PathCommand::MoveTo(point)],
        r(0, 1),
        1000,
        OutlinePoint {
            x: r(0, 1),
            y: r(0, 1)
        }
    )
    .is_err());
    assert!(r(MAX_EXACT_INTEGER as i128, 1)
        .checked_add(r(1, 1))
        .is_err());
}
#[test]
fn integral_and_rational_placement_paths_agree_exactly() {
    let point = ExactPoint {
        on_curve: true,
        x: Coordinate::from_integer(-120),
        y: Coordinate::from_integer(200),
    };
    let path = [
        PathCommand::MoveTo(point),
        PathCommand::LineTo(point),
        PathCommand::Close,
    ];
    let old = place_path(
        path,
        Tick(1024),
        1000,
        hit_test::Point {
            x: Tick(12),
            y: Tick(34),
        },
    )
    .unwrap();
    let exact = place_path_exact(
        path,
        r(1024, 1),
        1000,
        OutlinePoint {
            x: r(12, 1),
            y: r(34, 1),
        },
    )
    .unwrap();
    assert_eq!(old, exact);
}
