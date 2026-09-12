//! Rectangle model for a link's clickable area on an exported page.
//!
//! Units are whatever page-space unit the caller uses (points, unless it
//! says otherwise) — this crate does not convert or interpret them, only
//! checks that they describe a real, finite, non-negative rectangle.

use std::fmt;

/// A point in page space.
///
/// `x` and `y` are deliberately private. [`Point`] itself makes no
/// finiteness claim on its own — `Point::new` is a plain constructor — but
/// keeping the fields private closes off the only route by which a caller
/// could otherwise assemble a `Rect { origin: Point { x, y }, .. }` struct
/// literal directly and skip [`Rect::new`]'s validation entirely. With both
/// `Point` and `Rect` fields private, `Rect::new` is the sole way to obtain
/// a `Rect`, so every live `Rect` is guaranteed to satisfy its checks.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Point {
    x: f64,
    y: f64,
}

impl Point {
    pub const ORIGIN: Point = Point { x: 0.0, y: 0.0 };

    pub fn new(x: f64, y: f64) -> Point {
        Point { x, y }
    }

    pub fn x(&self) -> f64 {
        self.x
    }

    pub fn y(&self) -> f64 {
        self.y
    }
}

/// An axis-aligned rectangle with a lower-left origin, width, and height —
/// the clickable area of a link annotation.
///
/// `origin`, `width`, and `height` are deliberately private:
/// [`Rect::new`] is the only way to build one, so every live `Rect`
/// satisfies its checks (finite origin, finite non-negative extent). If the
/// fields were `pub`, a caller could assemble a
/// `Rect { origin: Point { x: f64::NAN, .. }, width: -5.0, height: f64::INFINITY }`
/// struct literal directly, skip `Rect::new` entirely, and hand `max_x()`/
/// `max_y()` a NaN-propagating rectangle that `is_empty()` also answers
/// wrongly for (a negative width is never "zero area", but the naive check
/// only ever compares against `0.0`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Rect {
    origin: Point,
    width: f64,
    height: f64,
}

impl Rect {
    /// Builds a rectangle, rejecting non-finite coordinates and negative or
    /// non-finite extents. This never clamps or silently repairs bad input.
    pub fn new(origin: Point, width: f64, height: f64) -> Result<Rect, RectError> {
        if !origin.x.is_finite() || !origin.y.is_finite() {
            return Err(RectError::NonFiniteOrigin {
                x: origin.x,
                y: origin.y,
            });
        }
        if !width.is_finite() || !height.is_finite() {
            return Err(RectError::NonFiniteExtent { width, height });
        }
        if width < 0.0 || height < 0.0 {
            return Err(RectError::NegativeExtent { width, height });
        }
        Ok(Rect {
            origin,
            width,
            height,
        })
    }

    pub fn origin(&self) -> Point {
        self.origin
    }

    pub fn width(&self) -> f64 {
        self.width
    }

    pub fn height(&self) -> f64 {
        self.height
    }

    pub fn max_x(&self) -> f64 {
        self.origin.x + self.width
    }

    pub fn max_y(&self) -> f64 {
        self.origin.y + self.height
    }

    /// True if this rectangle has zero area.
    pub fn is_empty(&self) -> bool {
        self.width == 0.0 || self.height == 0.0
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum RectError {
    NonFiniteOrigin { x: f64, y: f64 },
    NonFiniteExtent { width: f64, height: f64 },
    NegativeExtent { width: f64, height: f64 },
}

impl fmt::Display for RectError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RectError::NonFiniteOrigin { x, y } => {
                write!(f, "rectangle origin ({x}, {y}) is not finite")
            }
            RectError::NonFiniteExtent { width, height } => {
                write!(f, "rectangle extent {width}x{height} is not finite")
            }
            RectError::NegativeExtent { width, height } => {
                write!(f, "rectangle extent {width}x{height} is negative")
            }
        }
    }
}

impl std::error::Error for RectError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_negative_width() {
        let err = Rect::new(Point::ORIGIN, -1.0, 10.0).unwrap_err();
        assert_eq!(
            err,
            RectError::NegativeExtent {
                width: -1.0,
                height: 10.0
            }
        );
    }

    #[test]
    fn rejects_nan_origin() {
        let err = Rect::new(Point::new(f64::NAN, 0.0), 1.0, 1.0).unwrap_err();
        assert!(matches!(err, RectError::NonFiniteOrigin { .. }));
    }

    #[test]
    fn rejects_infinite_extent() {
        let err = Rect::new(Point::ORIGIN, f64::INFINITY, 1.0).unwrap_err();
        assert!(matches!(err, RectError::NonFiniteExtent { .. }));
    }

    #[test]
    fn accepts_and_computes_bounds() {
        let r = Rect::new(Point::new(10.0, 20.0), 30.0, 5.0).unwrap();
        assert_eq!(r.max_x(), 40.0);
        assert_eq!(r.max_y(), 25.0);
        assert!(!r.is_empty());
    }

    #[test]
    fn zero_area_is_empty() {
        let r = Rect::new(Point::ORIGIN, 0.0, 5.0).unwrap();
        assert!(r.is_empty());
    }

    /// Regression test for a validation-bypass defect: `Point` and `Rect`
    /// fields used to be `pub`, so a caller could assemble
    /// `Rect { origin: Point { x: f64::NAN, y: 0.0 }, width: -5.0, height:
    /// f64::INFINITY }` as a plain struct literal, skipping `Rect::new`
    /// entirely. That rectangle's `max_x()`/`max_y()` silently propagated
    /// NaN, and `is_empty()` answered `false` for a rectangle with negative
    /// width — never flagged as invalid anywhere.
    ///
    /// Now that the fields are private, the struct-literal bypass above is
    /// a compile error from outside this module (uncomment it locally to
    /// see `error[E0451]: field ... is private`), and `Rect::new` is the
    /// only way to build a `Rect`. The exact same NaN/negative-width/
    /// infinite-height input must now come back as a typed `RectError`.
    #[test]
    fn nan_origin_negative_width_and_infinite_height_is_a_typed_error_not_a_bypassed_rect() {
        // let bypassed = Rect {
        //     origin: Point { x: f64::NAN, y: 0.0 },
        //     width: -5.0,
        //     height: f64::INFINITY,
        // }; // <- no longer compiles: `origin`, `x`, and `width` are private.

        let err = Rect::new(Point::new(f64::NAN, 0.0), -5.0, f64::INFINITY).unwrap_err();
        // The origin check runs first, so a NaN origin is reported even
        // when width/height are also invalid — but whichever variant wins,
        // it must be a typed error, never a constructed `Rect`.
        assert!(matches!(err, RectError::NonFiniteOrigin { .. }));
    }
}
