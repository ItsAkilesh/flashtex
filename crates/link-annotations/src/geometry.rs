//! Rectangle model for a link's clickable area on an exported page.
//!
//! Units are whatever page-space unit the caller uses (points, unless it
//! says otherwise) — this crate does not convert or interpret them, only
//! checks that they describe a real, finite, non-negative rectangle.

use std::fmt;

/// A point in page space.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

impl Point {
    pub const ORIGIN: Point = Point { x: 0.0, y: 0.0 };

    pub fn new(x: f64, y: f64) -> Point {
        Point { x, y }
    }
}

/// An axis-aligned rectangle with a lower-left origin, width, and height —
/// the clickable area of a link annotation.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Rect {
    pub origin: Point,
    pub width: f64,
    pub height: f64,
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
}
