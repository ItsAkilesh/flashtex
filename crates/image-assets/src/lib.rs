//! Rooted, bounded document asset loader for PNG and JPEG.
//!
//! [`AssetLoader`] is the single entry point: it resolves a caller-supplied
//! relative path through an [`AssetRoot`] (rejecting traversal, absolute
//! paths and symlink escapes — see [`root`]), reads and bounds the raw
//! bytes, and decodes them with the `image` crate (already a dependency of
//! `flashtex-bridge` / `flashtex-conversion-jobs` in this workspace; no
//! decoder is hand-rolled here).
//!
//! Every successfully loaded [`ImageAsset`] carries:
//! - an [`AssetId`]: a SHA-256 digest of the raw encoded bytes, so identical
//!   content always has the same identity regardless of where it lives, and
//!   a single changed byte always changes it;
//! - its real decoded pixel [`Dimensions`];
//! - and, via [`geometry::IncludeGraphicsSpec`], `\includegraphics`-style
//!   sizing and crop metadata that is validated against those real
//!   dimensions rather than trusted blindly.

pub mod geometry;
pub mod root;

use std::fmt;
use std::io::Cursor;
use std::path::{Path, PathBuf};

use image::{GenericImageView, ImageFormat, ImageReader, Limits};
use sha2::{Digest, Sha256};

pub use geometry::{Crop, GeometryError, IncludeGraphicsSpec, Length};
pub use root::{AssetRoot, RootError};

/// Content-derived, stable identity for an asset's raw encoded bytes.
///
/// Two assets with byte-identical content always share an [`AssetId`],
/// regardless of path; changing even one byte changes it.
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct AssetId([u8; 32]);

impl AssetId {
    fn of(bytes: &[u8]) -> Self {
        let mut hasher = Sha256::new();
        hasher.update(bytes);
        let mut out = [0u8; 32];
        out.copy_from_slice(&hasher.finalize());
        AssetId(out)
    }

    /// The raw 32-byte SHA-256 digest.
    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }

    /// Lowercase hex encoding of the digest, e.g. for use as a cache key or
    /// filename.
    pub fn to_hex(&self) -> String {
        let mut s = String::with_capacity(64);
        for b in self.0 {
            s.push_str(&format!("{b:02x}"));
        }
        s
    }
}

impl fmt::Display for AssetId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.to_hex())
    }
}

impl fmt::Debug for AssetId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "AssetId({})", self.to_hex())
    }
}

/// A raster format this crate can decode. Deliberately just the two formats
/// `\includegraphics` document assets need; anything else is rejected.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AssetFormat {
    Png,
    Jpeg,
}

/// Real, decoded pixel dimensions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Dimensions {
    pub width: u32,
    pub height: u32,
}

/// A successfully loaded and decoded document image asset.
#[derive(Clone)]
pub struct ImageAsset {
    id: AssetId,
    format: AssetFormat,
    dimensions: Dimensions,
    bytes: Vec<u8>,
}

impl fmt::Debug for ImageAsset {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ImageAsset")
            .field("id", &self.id)
            .field("format", &self.format)
            .field("dimensions", &self.dimensions)
            .field("bytes_len", &self.bytes.len())
            .finish()
    }
}

impl ImageAsset {
    /// Content-derived identity of the raw encoded bytes.
    pub fn id(&self) -> AssetId {
        self.id
    }

    pub fn format(&self) -> AssetFormat {
        self.format
    }

    /// Real decoded pixel dimensions.
    pub fn dimensions(&self) -> Dimensions {
        self.dimensions
    }

    /// The raw encoded bytes as read from disk (not the decoded pixels).
    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }

    /// Validates an `\includegraphics`-style sizing/crop spec against this
    /// asset's real dimensions.
    pub fn validate_spec(&self, spec: &IncludeGraphicsSpec) -> Result<(), GeometryError> {
        spec.validate(self.dimensions)
    }
}

/// Why loading an asset failed.
#[derive(Debug)]
pub enum AssetError {
    /// The path could not be resolved inside the document root.
    Root(RootError),
    /// The file exists and is in-root but reading it failed.
    Io(std::io::Error),
    /// The file is larger than the configured bound.
    TooLarge { limit: usize, actual: u64 },
    /// The file is zero bytes.
    Empty,
    /// The bytes were not recognized as PNG or JPEG.
    UnsupportedFormat,
    /// The bytes were recognized as PNG or JPEG but failed to decode
    /// (truncated, corrupt, or exceeding decode limits).
    Decode(String),
}

impl fmt::Display for AssetError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AssetError::Root(e) => write!(f, "{e}"),
            AssetError::Io(e) => write!(f, "asset I/O error: {e}"),
            AssetError::TooLarge { limit, actual } => write!(
                f,
                "asset is {actual} bytes, exceeding the {limit}-byte bound"
            ),
            AssetError::Empty => write!(f, "asset file is empty"),
            AssetError::UnsupportedFormat => {
                write!(f, "asset is not a recognized PNG or JPEG")
            }
            AssetError::Decode(msg) => write!(f, "asset failed to decode: {msg}"),
        }
    }
}

impl std::error::Error for AssetError {}

impl From<RootError> for AssetError {
    fn from(e: RootError) -> Self {
        AssetError::Root(e)
    }
}

/// Loads bounded, rooted PNG/JPEG assets and assigns them a content-derived
/// identity.
#[derive(Debug, Clone)]
pub struct AssetLoader {
    root: AssetRoot,
    max_file_bytes: u64,
}

impl AssetLoader {
    /// Default bound on raw encoded file size: 32 MiB.
    pub const DEFAULT_MAX_FILE_BYTES: u64 = 32 * 1024 * 1024;
    /// Default bound on decoded pixel dimensions in either axis.
    pub const DEFAULT_MAX_PIXELS_PER_AXIS: u32 = 16384;
    /// Default bound on the allocation the decoder may make.
    pub const DEFAULT_MAX_DECODE_ALLOC_BYTES: u64 = 256 * 1024 * 1024;

    /// Creates a loader bounded to `root`, with the default byte/pixel
    /// limits.
    pub fn new(root: AssetRoot) -> Self {
        Self {
            root,
            max_file_bytes: Self::DEFAULT_MAX_FILE_BYTES,
        }
    }

    /// Overrides the raw encoded file size bound.
    pub fn with_max_file_bytes(mut self, max_file_bytes: u64) -> Self {
        self.max_file_bytes = max_file_bytes;
        self
    }

    pub fn root(&self) -> &AssetRoot {
        &self.root
    }

    /// Loads and decodes the asset at `relative`, a path taken to be
    /// relative to the document root.
    ///
    /// Fails with [`AssetError::Root`] if `relative` escapes the root by
    /// traversal, is absolute, or resolves through a symlink to outside the
    /// root. Fails with [`AssetError::TooLarge`] / [`AssetError::Decode`] /
    /// [`AssetError::UnsupportedFormat`] for oversized, malformed, or
    /// non-PNG/JPEG content — this call never panics on untrusted input.
    pub fn load(&self, relative: impl AsRef<Path>) -> Result<ImageAsset, AssetError> {
        let resolved = self.root.resolve(relative)?;
        self.load_resolved(&resolved)
    }

    fn load_resolved(&self, resolved: &PathBuf) -> Result<ImageAsset, AssetError> {
        let metadata = std::fs::metadata(resolved).map_err(AssetError::Io)?;
        let len = metadata.len();
        if len > self.max_file_bytes {
            return Err(AssetError::TooLarge {
                limit: self.max_file_bytes as usize,
                actual: len,
            });
        }
        let bytes = std::fs::read(resolved).map_err(AssetError::Io)?;
        if bytes.is_empty() {
            return Err(AssetError::Empty);
        }
        let (format, dimensions) = decode_bounded(&bytes)?;
        Ok(ImageAsset {
            id: AssetId::of(&bytes),
            format,
            dimensions,
            bytes,
        })
    }
}

/// Decodes `bytes` as PNG or JPEG under bounded resource limits. Never
/// panics: truncated, garbage, or oversized-when-decoded input all produce
/// a clean [`AssetError`].
fn decode_bounded(bytes: &[u8]) -> Result<(AssetFormat, Dimensions), AssetError> {
    let reader = ImageReader::new(Cursor::new(bytes))
        .with_guessed_format()
        .map_err(AssetError::Io)?;
    let format = match reader.format() {
        Some(ImageFormat::Png) => AssetFormat::Png,
        Some(ImageFormat::Jpeg) => AssetFormat::Jpeg,
        _ => return Err(AssetError::UnsupportedFormat),
    };

    let mut limits = Limits::default();
    limits.max_image_width = Some(AssetLoader::DEFAULT_MAX_PIXELS_PER_AXIS);
    limits.max_image_height = Some(AssetLoader::DEFAULT_MAX_PIXELS_PER_AXIS);
    limits.max_alloc = Some(AssetLoader::DEFAULT_MAX_DECODE_ALLOC_BYTES);

    let mut reader = ImageReader::with_format(Cursor::new(bytes), format.to_image_format());
    reader.limits(limits);
    let decoded = reader
        .decode()
        .map_err(|e| AssetError::Decode(e.to_string()))?;
    let (width, height) = decoded.dimensions();
    Ok((format, Dimensions { width, height }))
}

impl AssetFormat {
    fn to_image_format(self) -> ImageFormat {
        match self {
            AssetFormat::Png => ImageFormat::Png,
            AssetFormat::Jpeg => ImageFormat::Jpeg,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn one_pixel_png() -> Vec<u8> {
        let mut buf = Vec::new();
        image::DynamicImage::new_rgb8(1, 1)
            .write_to(&mut Cursor::new(&mut buf), ImageFormat::Png)
            .unwrap();
        buf
    }

    #[test]
    fn asset_id_is_stable_for_identical_bytes() {
        let bytes = one_pixel_png();
        assert_eq!(AssetId::of(&bytes), AssetId::of(&bytes.clone()));
    }

    #[test]
    fn asset_id_changes_with_a_single_byte() {
        let mut bytes = one_pixel_png();
        let original = AssetId::of(&bytes);
        let last = bytes.len() - 1;
        bytes[last] ^= 0x01;
        assert_ne!(original, AssetId::of(&bytes));
    }

    #[test]
    fn decode_bounded_rejects_empty_input_cleanly() {
        let err = decode_bounded(&[]);
        assert!(err.is_err());
    }

    #[test]
    fn decode_bounded_rejects_garbage_cleanly() {
        let err = decode_bounded(&[0u8, 1, 2, 3, 4, 5, 6, 7, 8, 9]);
        assert!(matches!(err, Err(AssetError::UnsupportedFormat)));
    }

    #[test]
    fn decode_bounded_rejects_truncated_png_cleanly() {
        let full = one_pixel_png();
        let truncated = &full[..full.len() / 2];
        let err = decode_bounded(truncated);
        assert!(err.is_err());
    }

    #[test]
    fn decode_bounded_reports_real_dimensions() {
        let mut buf = Vec::new();
        image::DynamicImage::new_rgb8(37, 19)
            .write_to(&mut Cursor::new(&mut buf), ImageFormat::Png)
            .unwrap();
        let (format, dims) = decode_bounded(&buf).unwrap();
        assert_eq!(format, AssetFormat::Png);
        assert_eq!(
            dims,
            Dimensions {
                width: 37,
                height: 19
            }
        );
    }

    #[test]
    fn asset_id_hex_round_trips_length() {
        let id = AssetId::of(&one_pixel_png());
        assert_eq!(id.to_hex().len(), 64);
        assert_eq!(format!("{id}"), id.to_hex());
    }
}
