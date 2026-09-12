//! Hardening tests for phone-camera captures, as distinct from the clean
//! iPad-Pencil path: EXIF orientation, decompression-bomb-shaped inputs, and
//! honest rejection of formats we deliberately do not support (HEIC).
use base64::{engine::general_purpose::STANDARD, Engine};
use flashtex_bridge::{
    store::Store, Bridge, CaptureImage, CaptureSubmit, Document, MAX_IMAGE_BYTES,
};
use image::ImageDecoder;
use std::io::Cursor;

fn capture_with(mime_type: &str, data_base64: String) -> CaptureSubmit {
    CaptureSubmit {
        capture_id: "capture-1".into(),
        destination_id: "anchor-1".into(),
        base_revision: 1,
        image: CaptureImage {
            mime_type: mime_type.into(),
            data_base64,
        },
        instructions: "Keep the notation".into(),
    }
}

/// A minimal Exif APP1 segment (little-endian TIFF, one IFD0 entry: tag
/// 0x0112 Orientation) declaring the given orientation value (1-8), exactly
/// as a phone camera embeds it right after a JPEG's SOI marker.
fn exif_orientation_app1(value: u16) -> Vec<u8> {
    let mut tiff = Vec::new();
    tiff.extend_from_slice(b"II");
    tiff.extend_from_slice(&42u16.to_le_bytes());
    tiff.extend_from_slice(&8u32.to_le_bytes()); // offset of IFD0, relative to this header
    tiff.extend_from_slice(&1u16.to_le_bytes()); // one IFD0 entry
    tiff.extend_from_slice(&0x0112u16.to_le_bytes()); // tag: Orientation
    tiff.extend_from_slice(&3u16.to_le_bytes()); // type: SHORT
    tiff.extend_from_slice(&1u32.to_le_bytes()); // count: 1
    tiff.extend_from_slice(&value.to_le_bytes());
    tiff.extend_from_slice(&[0, 0]); // pad SHORT into the 4-byte value slot
    tiff.extend_from_slice(&0u32.to_le_bytes()); // next IFD offset: none

    let mut content = b"Exif\0\0".to_vec();
    content.extend_from_slice(&tiff);
    let seg_len = (content.len() + 2) as u16; // length field counts itself

    let mut app1 = vec![0xFF, 0xE1];
    app1.extend_from_slice(&seg_len.to_be_bytes());
    app1.extend_from_slice(&content);
    app1
}

/// Encodes `img` as JPEG (no metadata).
fn encode_jpeg(img: &image::RgbImage) -> Vec<u8> {
    let mut plain = Cursor::new(Vec::new());
    image::DynamicImage::ImageRgb8(img.clone())
        .write_to(&mut plain, image::ImageFormat::Jpeg)
        .unwrap();
    plain.into_inner()
}

/// Splices the given Exif orientation tag right after a JPEG's SOI marker,
/// exactly as phone camera firmware embeds it. The scan data is untouched,
/// so decoding `plain` and decoding the spliced result yield identical
/// pixels modulo the orientation metadata itself.
fn splice_orientation(plain: &[u8], orientation: u16) -> Vec<u8> {
    assert_eq!(&plain[0..2], &[0xFF, 0xD8], "expected a JPEG SOI marker");
    let mut out = plain[0..2].to_vec();
    out.extend_from_slice(&exif_orientation_app1(orientation));
    out.extend_from_slice(&plain[2..]);
    out
}

/// A 2x3 image with a distinct color in each corner so a rotation is
/// externally observable in the decoded pixels.
fn corner_marked_image() -> image::RgbImage {
    let mut img = image::RgbImage::new(2, 3);
    img.put_pixel(0, 0, image::Rgb([255, 0, 0])); // top-left: red
    img.put_pixel(1, 0, image::Rgb([0, 255, 0])); // top-right: green
    img.put_pixel(0, 2, image::Rgb([0, 0, 255])); // bottom-left: blue
    img.put_pixel(1, 2, image::Rgb([255, 255, 0])); // bottom-right: yellow
    img
}

#[test]
fn exif_orientation_is_applied_and_image_is_normalized_to_png() {
    let img = corner_marked_image();
    let plain = encode_jpeg(&img);
    let jpeg = splice_orientation(&plain, 6); // Exif 6 == Orientation::Rotate90 (90 deg CW)
    let mut cap = capture_with("image/jpeg", STANDARD.encode(&jpeg));

    cap.validate()
        .expect("a rotated-but-otherwise-valid capture must validate");

    // The corrected bytes (and an honest mime type for them) are what a
    // caller sees afterward -- including, in production, what actually gets
    // embedded in the Grok request body.
    assert_eq!(cap.image.mime_type, "image/png");

    let corrected_bytes = STANDARD.decode(&cap.image.data_base64).unwrap();
    let corrected = image::load_from_memory_with_format(&corrected_bytes, image::ImageFormat::Png)
        .unwrap()
        .to_rgb8();

    // Rotating 90 degrees clockwise swaps width and height.
    assert_eq!((corrected.width(), corrected.height()), (3, 2));

    // Compare pixel-for-pixel against independently rotating the pixels
    // decoded from the *same, metadata-free* JPEG bytes via `image`'s own
    // transform for this orientation value. Using the metadata-free bytes
    // (rather than the original lossless RgbImage) keeps JPEG's lossy
    // compression noise identical on both sides, so this isolates whether
    // our code read the tag we embedded and applied the matching transform,
    // rather than producing merely *some* 3x2 image.
    let raw_decoded =
        image::load_from_memory_with_format(&plain, image::ImageFormat::Jpeg).unwrap();
    let expected = raw_decoded.rotate90().to_rgb8();
    assert_eq!(corrected, expected);

    // The re-encoded PNG must not carry the orientation tag forward, or a
    // second Exif-aware reader downstream would rotate it a second time.
    let mut decoder =
        image::ImageReader::with_format(Cursor::new(corrected_bytes), image::ImageFormat::Png)
            .into_decoder()
            .unwrap();
    assert_eq!(
        decoder.orientation().unwrap(),
        image::metadata::Orientation::NoTransforms
    );
}

#[test]
fn upright_jpeg_without_exif_orientation_is_left_untouched() {
    let img = corner_marked_image();
    let mut plain = Cursor::new(Vec::new());
    image::DynamicImage::ImageRgb8(img)
        .write_to(&mut plain, image::ImageFormat::Jpeg)
        .unwrap();
    let original_b64 = STANDARD.encode(plain.into_inner());
    let mut cap = capture_with("image/jpeg", original_b64.clone());

    cap.validate()
        .expect("a plain upright capture must validate");

    // No orientation tag means nothing to correct: avoid a lossy re-encode
    // and preserve the caller's original bytes and declared format.
    assert_eq!(cap.image.mime_type, "image/jpeg");
    assert_eq!(cap.image.data_base64, original_b64);
}

#[test]
fn heic_is_rejected_with_an_honest_error_not_a_crash() {
    // iPhones default to HEIC, not JPEG. We deliberately do not decode it
    // (that would require adding a new dependency); verify instead that the
    // rejection is a clean, explicit, operator-legible error.
    let mut cap = capture_with(
        "image/heic",
        STANDARD.encode(b"not a real HEIC payload either way"),
    );
    let err = cap.validate().unwrap_err();
    assert_eq!(err.code, "unsupported_image");
    assert!(
        err.message.contains("HEIC"),
        "error should name the photo format as the problem, got: {}",
        err.message
    );
}

#[test]
fn truncated_png_with_huge_declared_dimensions_is_rejected_before_full_decode() {
    // Classic decompression-bomb shape: a few dozen header bytes claiming an
    // enormous image, with no pixel data at all. If dimensions were only
    // checked after a full decode, this would instead fail for a different
    // reason (missing IDAT) -- asserting `invalid_image` here pins down that
    // the width/height cap is what actually fires, and that it fires without
    // needing (or allocating) any bomb payload.
    let mut png = vec![137, 80, 78, 71, 13, 10, 26, 10]; // PNG signature
                                                         // IHDR declaring 20000x20000, 8-bit RGB, no interlace; no other chunks follow.
    png.extend_from_slice(&[
        0, 0, 0, 13, 73, 72, 68, 82, 0, 0, 78, 32, 0, 0, 78, 32, 8, 2, 0, 0, 0, 108, 18, 209, 110,
    ]);
    let mut cap = capture_with("image/png", STANDARD.encode(&png));
    assert_eq!(cap.validate().unwrap_err().code, "invalid_image");
}

#[test]
fn dimensions_within_cap_but_over_alloc_budget_are_rejected_before_pixel_decode() {
    // 8192x8192 is individually within the width/height cap, but at 3 bytes
    // per pixel that is ~192 MiB of pixel data: far past the 64 MiB
    // allocation budget. This specifically exercises the `Limits::reserve`
    // guard that `ImageReader::into_decoder()` does not apply on its own
    // (unlike `ImageReader::decode()`, which the un-hardened code used).
    let mut png = vec![137, 80, 78, 71, 13, 10, 26, 10]; // PNG signature
                                                         // IHDR declaring 8192x8192, 8-bit RGB, no interlace; no other chunks follow.
    png.extend_from_slice(&[
        0, 0, 0, 13, 73, 72, 68, 82, 0, 0, 32, 0, 0, 0, 32, 0, 8, 2, 0, 0, 0, 253, 200, 93, 14,
    ]);
    let mut cap = capture_with("image/png", STANDARD.encode(&png));
    assert_eq!(cap.validate().unwrap_err().code, "invalid_image");
}

#[test]
fn oversized_encoded_image_is_a_clean_error_not_unbounded_decode() {
    // A modern phone photo can run several megabytes; make sure well past
    // the 8 MiB cap is a bounded, honest rejection rather than an attempt to
    // base64-decode and hold an unbounded buffer.
    let oversized_b64 = "A".repeat(MAX_IMAGE_BYTES.div_ceil(3) * 4 + 4);
    let mut cap = capture_with("image/jpeg", oversized_b64);
    assert_eq!(cap.validate().unwrap_err().code, "image_too_large");
}

#[test]
fn rotated_photo_is_normalized_before_it_ever_reaches_the_stored_capture_record() {
    // End-to-end: prove the corrected bytes, not the original sideways ones,
    // are what `Bridge::receive` durably stores -- and therefore what the
    // Grok request body (which reads `record.capture.image` directly) would
    // actually send.
    let dir = tempfile::tempdir().unwrap();
    let mut bridge = Bridge::new(Store::open(dir.path()).unwrap());
    bridge
        .open_document(Document {
            project_id: "project".into(),
            path: "main.tex".into(),
            revision: 1,
            text: "hello world".into(),
        })
        .unwrap();
    bridge
        .pin("anchor-1", "project", "main.tex", 1, 0, 0)
        .unwrap();

    let img = corner_marked_image();
    let plain = encode_jpeg(&img);
    let jpeg = splice_orientation(&plain, 8); // Exif 8 == Orientation::Rotate270
    let cap = capture_with("image/jpeg", STANDARD.encode(&jpeg));

    let record = bridge.receive(cap).unwrap();
    assert_eq!(record.capture.image.mime_type, "image/png");

    let stored_bytes = STANDARD.decode(&record.capture.image.data_base64).unwrap();
    let stored = image::load_from_memory_with_format(&stored_bytes, image::ImageFormat::Png)
        .unwrap()
        .to_rgb8();
    let raw_decoded =
        image::load_from_memory_with_format(&plain, image::ImageFormat::Jpeg).unwrap();
    let expected = raw_decoded.rotate270().to_rgb8();
    assert_eq!(stored, expected);
}
