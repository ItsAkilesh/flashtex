//! End-to-end coverage of `AssetLoader`: real files on disk, real SHA-256
//! identities, real decoded dimensions, and the three ways a path must be
//! rejected before it ever reaches the decoder.

use std::fs;
use std::io::Cursor;

use flashtex_image_assets::{AssetError, AssetFormat, AssetLoader, AssetRoot, RootError};
use image::{DynamicImage, ImageFormat};
use sha2::{Digest, Sha256};

fn png_bytes(width: u32, height: u32) -> Vec<u8> {
    let mut buf = Vec::new();
    DynamicImage::new_rgb8(width, height)
        .write_to(&mut Cursor::new(&mut buf), ImageFormat::Png)
        .unwrap();
    buf
}

fn jpeg_bytes(width: u32, height: u32) -> Vec<u8> {
    let mut buf = Vec::new();
    DynamicImage::new_rgb8(width, height)
        .write_to(&mut Cursor::new(&mut buf), ImageFormat::Jpeg)
        .unwrap();
    buf
}

fn sha256_hex(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    hasher
        .finalize()
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}

#[test]
fn loads_a_real_png_with_real_dimensions_and_real_sha_identity() {
    let dir = tempfile::tempdir().unwrap();
    let bytes = png_bytes(64, 32);
    fs::write(dir.path().join("fig.png"), &bytes).unwrap();

    let loader = AssetLoader::new(AssetRoot::new(dir.path()).unwrap());
    let asset = loader.load("fig.png").unwrap();

    assert_eq!(asset.format(), AssetFormat::Png);
    assert_eq!(asset.dimensions().width, 64);
    assert_eq!(asset.dimensions().height, 32);
    assert_eq!(asset.id().to_hex(), sha256_hex(&bytes));
}

#[test]
fn loads_a_real_jpeg_with_real_dimensions() {
    let dir = tempfile::tempdir().unwrap();
    let bytes = jpeg_bytes(48, 20);
    fs::write(dir.path().join("photo.jpg"), &bytes).unwrap();

    let loader = AssetLoader::new(AssetRoot::new(dir.path()).unwrap());
    let asset = loader.load("photo.jpg").unwrap();

    assert_eq!(asset.format(), AssetFormat::Jpeg);
    assert_eq!(asset.dimensions().width, 48);
    assert_eq!(asset.dimensions().height, 20);
}

#[test]
fn loads_an_asset_at_a_unicode_path() {
    let dir = tempfile::tempdir().unwrap();
    fs::create_dir_all(dir.path().join("図/📁directory")).unwrap();
    let bytes = png_bytes(10, 10);
    let rel = "図/📁directory/日本語ファイル名😀.png";
    fs::write(dir.path().join(rel), &bytes).unwrap();

    let loader = AssetLoader::new(AssetRoot::new(dir.path()).unwrap());
    let asset = loader.load(rel).unwrap();

    assert_eq!(asset.dimensions().width, 10);
    assert_eq!(asset.id().to_hex(), sha256_hex(&bytes));
}

#[test]
fn identical_bytes_at_different_paths_share_one_identity() {
    let dir = tempfile::tempdir().unwrap();
    let bytes = png_bytes(5, 5);
    fs::create_dir_all(dir.path().join("a/b")).unwrap();
    fs::write(dir.path().join("one.png"), &bytes).unwrap();
    fs::write(dir.path().join("a/b/two.png"), &bytes).unwrap();

    let loader = AssetLoader::new(AssetRoot::new(dir.path()).unwrap());
    let one = loader.load("one.png").unwrap();
    let two = loader.load("a/b/two.png").unwrap();

    assert_eq!(one.id(), two.id());
}

#[test]
fn a_single_changed_byte_in_the_stored_file_changes_the_identity() {
    // The identity is a hash of the raw on-disk bytes, not of the decoded
    // pixels, so flip one trailing byte after the PNG's IEND chunk (a
    // no-op for any PNG decoder, since decoding stops at IEND) and confirm
    // the loader still decodes it but assigns a different identity.
    let dir = tempfile::tempdir().unwrap();
    let mut bytes = png_bytes(5, 5);
    fs::write(dir.path().join("a.png"), &bytes).unwrap();
    bytes.push(0x00);
    let last = bytes.len() - 1;
    bytes[last] = 0xFF;
    fs::write(dir.path().join("b.png"), &bytes).unwrap();

    let loader = AssetLoader::new(AssetRoot::new(dir.path()).unwrap());
    let a = loader.load("a.png").unwrap();
    let b = loader.load("b.png").unwrap();

    assert_eq!(a.dimensions(), b.dimensions());
    assert_ne!(a.id(), b.id());
    assert_eq!(a.id().to_hex(), sha256_hex(&fs::read(dir.path().join("a.png")).unwrap()));
    assert_eq!(b.id().to_hex(), sha256_hex(&fs::read(dir.path().join("b.png")).unwrap()));
}

#[test]
fn rejects_path_traversal_with_a_typed_error() {
    let dir = tempfile::tempdir().unwrap();
    let loader = AssetLoader::new(AssetRoot::new(dir.path()).unwrap());
    let err = loader.load("../../etc/passwd").unwrap_err();
    assert!(
        matches!(err, AssetError::Root(RootError::PathTraversal(_))),
        "{err:?}"
    );
}

#[test]
fn rejects_absolute_paths_with_a_typed_error() {
    let dir = tempfile::tempdir().unwrap();
    let loader = AssetLoader::new(AssetRoot::new(dir.path()).unwrap());
    let err = loader.load("/etc/passwd").unwrap_err();
    assert!(
        matches!(err, AssetError::Root(RootError::AbsolutePath(_))),
        "{err:?}"
    );
}

#[test]
#[cfg(unix)]
fn rejects_symlink_escape_with_a_typed_error() {
    let dir = tempfile::tempdir().unwrap();
    let outside = tempfile::tempdir().unwrap();
    fs::write(outside.path().join("secret.png"), png_bytes(1, 1)).unwrap();
    std::os::unix::fs::symlink(
        outside.path().join("secret.png"),
        dir.path().join("innocuous.png"),
    )
    .unwrap();

    let loader = AssetLoader::new(AssetRoot::new(dir.path()).unwrap());
    let err = loader.load("innocuous.png").unwrap_err();
    assert!(
        matches!(err, AssetError::Root(RootError::SymlinkEscape(_))),
        "{err:?}"
    );
}

#[test]
fn rejects_truncated_png_cleanly_never_panics() {
    let dir = tempfile::tempdir().unwrap();
    let full = png_bytes(20, 20);
    fs::write(dir.path().join("bad.png"), &full[..full.len() / 3]).unwrap();

    let loader = AssetLoader::new(AssetRoot::new(dir.path()).unwrap());
    let err = loader.load("bad.png").unwrap_err();
    assert!(matches!(err, AssetError::Decode(_)), "{err:?}");
}

#[test]
fn rejects_garbage_bytes_cleanly_never_panics() {
    let dir = tempfile::tempdir().unwrap();
    fs::write(dir.path().join("bad.png"), b"this is not an image, just noise").unwrap();

    let loader = AssetLoader::new(AssetRoot::new(dir.path()).unwrap());
    let err = loader.load("bad.png").unwrap_err();
    assert!(matches!(err, AssetError::UnsupportedFormat), "{err:?}");
}

#[test]
fn rejects_a_png_that_declares_itself_but_is_all_zero_body() {
    let dir = tempfile::tempdir().unwrap();
    // Correct 8-byte PNG signature, followed by zeroed-out chunk data:
    // sniffs as PNG, then must fail to decode instead of panicking.
    let mut bytes = vec![0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];
    bytes.extend(std::iter::repeat_n(0u8, 64));
    fs::write(dir.path().join("bad.png"), &bytes).unwrap();

    let loader = AssetLoader::new(AssetRoot::new(dir.path()).unwrap());
    let err = loader.load("bad.png").unwrap_err();
    assert!(matches!(err, AssetError::Decode(_)), "{err:?}");
}

#[test]
fn rejects_a_file_over_the_configured_byte_bound() {
    let dir = tempfile::tempdir().unwrap();
    fs::write(dir.path().join("big.png"), png_bytes(4, 4)).unwrap();

    let loader = AssetLoader::new(AssetRoot::new(dir.path()).unwrap()).with_max_file_bytes(4);
    let err = loader.load("big.png").unwrap_err();
    assert!(matches!(err, AssetError::TooLarge { .. }), "{err:?}");
}

#[test]
fn rejects_a_missing_file_without_panicking() {
    let dir = tempfile::tempdir().unwrap();
    let loader = AssetLoader::new(AssetRoot::new(dir.path()).unwrap());
    let err = loader.load("nope.png").unwrap_err();
    assert!(
        matches!(err, AssetError::Root(RootError::NotFound(_))),
        "{err:?}"
    );
}
