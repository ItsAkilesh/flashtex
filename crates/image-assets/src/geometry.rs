//! `\includegraphics`-style sizing and crop metadata.
//!
//! This module carries the placement intent a document author attaches to
//! an asset (explicit width/height, a uniform scale, and a crop rectangle)
//! without performing layout itself. The one thing it does enforce is that
//! a crop is bounded: it must describe a non-empty rectangle that actually
//! fits inside the asset's real decoded pixel dimensions.

use std::fmt;

use crate::Dimensions;

/// A `\includegraphics`-style length: either an absolute size in big points
/// (the LaTeX `bp` unit `\includegraphics[width=3in]` is normalized to), or
/// a scale factor relative to the asset's natural size.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Length {
    /// Absolute length in big points (1/72 inch), e.g. `width=216bp`.
    Points(f64),
    /// A multiple of the natural pixel size, e.g. `scale=0.5`.
    Scale(f64),
}

/// Sizing and crop metadata for one placement of an asset, mirroring the
/// options `\includegraphics` accepts.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct IncludeGraphicsSpec {
    pub width: Option<Length>,
    pub height: Option<Length>,
    pub crop: Option<Crop>,
}

impl IncludeGraphicsSpec {
    /// No explicit sizing or cropping: render the asset at its natural size.
    pub const NATURAL: Self = Self {
        width: None,
        height: None,
        crop: None,
    };

    /// Validates this spec against an asset's real decoded dimensions.
    /// Only the crop rectangle needs bounds checking; width/height/scale
    /// are left to the layout engine.
    pub fn validate(&self, dimensions: Dimensions) -> Result<(), GeometryError> {
        if let Some(crop) = self.crop {
            crop.validate(dimensions)?;
        }
        Ok(())
    }
}

/// A crop rectangle expressed the way `\includegraphics[trim=...,clip]`
/// expresses it: pixel margins to remove from each edge of the natural
/// image, left/bottom/right/top.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Crop {
    pub left: u32,
    pub bottom: u32,
    pub right: u32,
    pub top: u32,
}

impl Crop {
    /// The pixel dimensions remaining after this crop is applied to an
    /// image of `dimensions`, if the crop is bounded (fits, and leaves a
    /// non-empty result).
    pub fn validate(&self, dimensions: Dimensions) -> Result<Dimensions, GeometryError> {
        let width = self
            .left
            .checked_add(self.right)
            .and_then(|margins| dimensions.width.checked_sub(margins));
        let height = self
            .bottom
            .checked_add(self.top)
            .and_then(|margins| dimensions.height.checked_sub(margins));
        match (width, height) {
            (Some(w), Some(h)) if w > 0 && h > 0 => Ok(Dimensions {
                width: w,
                height: h,
            }),
            _ => Err(GeometryError::CropOutOfBounds {
                crop: *self,
                dimensions,
            }),
        }
    }
}

/// Why a sizing/crop spec was rejected.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GeometryError {
    /// The crop rectangle does not fit inside the asset's real dimensions,
    /// or leaves nothing behind.
    CropOutOfBounds {
        crop: Crop,
        dimensions: Dimensions,
    },
}

impl fmt::Display for GeometryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GeometryError::CropOutOfBounds { crop, dimensions } => write!(
                f,
                "crop {{left:{}, bottom:{}, right:{}, top:{}}} does not fit inside {}x{} image",
                crop.left, crop.bottom, crop.right, crop.top, dimensions.width, dimensions.height
            ),
        }
    }
}

impl std::error::Error for GeometryError {}

#[cfg(test)]
mod tests {
    use super::*;

    const DIMS: Dimensions = Dimensions {
        width: 100,
        height: 50,
    };

    #[test]
    fn crop_within_bounds_is_accepted() {
        let crop = Crop {
            left: 10,
            bottom: 5,
            right: 10,
            top: 5,
        };
        let result = crop.validate(DIMS).unwrap();
        assert_eq!(result, Dimensions {
            width: 80,
            height: 40
        });
    }

    #[test]
    fn crop_exactly_consuming_the_image_is_rejected() {
        let crop = Crop {
            left: 50,
            bottom: 25,
            right: 50,
            top: 25,
        };
        assert!(matches!(
            crop.validate(DIMS),
            Err(GeometryError::CropOutOfBounds { .. })
        ));
    }

    #[test]
    fn crop_wider_than_the_image_is_rejected_not_a_panic() {
        let crop = Crop {
            left: 90,
            bottom: 0,
            right: 90,
            top: 0,
        };
        assert!(matches!(
            crop.validate(DIMS),
            Err(GeometryError::CropOutOfBounds { .. })
        ));
    }

    #[test]
    fn crop_with_overflowing_margins_is_rejected_not_a_panic() {
        let crop = Crop {
            left: u32::MAX,
            bottom: 0,
            right: 1,
            top: 0,
        };
        assert!(matches!(
            crop.validate(DIMS),
            Err(GeometryError::CropOutOfBounds { .. })
        ));
    }

    #[test]
    fn spec_with_no_crop_always_validates() {
        assert!(IncludeGraphicsSpec::NATURAL.validate(DIMS).is_ok());
    }

    #[test]
    fn spec_validate_propagates_crop_error() {
        let spec = IncludeGraphicsSpec {
            width: Some(Length::Points(216.0)),
            height: None,
            crop: Some(Crop {
                left: 200,
                bottom: 0,
                right: 0,
                top: 0,
            }),
        };
        assert!(spec.validate(DIMS).is_err());
    }
}
