import Foundation
import UIKit

/// Validates and constrains capture images per runtime-v1 protocol requirements.
/// Rejects malformed and oversized images before transport.
enum ImageValidator {
    /// Maximum dimension (width or height) for captured images.
    static let maxDimension: CGFloat = 4096

    /// Maximum PNG data size (10 MB before base64 encoding).
    static let maxDataSize: Int = 10 * 1024 * 1024

    /// Accepted MIME types per runtime-v1.
    static let acceptedMIMETypes: Set<String> = ["image/png", "image/jpeg"]

    struct ValidationResult {
        let isValid: Bool
        let image: UIImage?
        let error: String?
    }

    // MARK: – Orientation normalization

    /// Returns a copy of `image` whose `imageOrientation` is `.up`.
    ///
    /// UIImagePickerController and PHAsset deliver images whose pixel buffer is
    /// correct but whose `imageOrientation` property tells renderers to rotate it.
    /// Left un-normalised, the base-64 JPEG or PNG sent in a capture_submit payload
    /// embeds the orientation in EXIF; many recipients (including the FlashTeX
    /// runtime) don't apply EXIF rotation and receive a mis-rotated image.
    /// Drawing through UIGraphicsImageRenderer bakes the transform into pixels and
    /// strips the EXIF flag so the output is always correctly oriented.
    static func normalizeOrientation(_ image: UIImage) -> UIImage {
        guard image.imageOrientation != .up else { return image }
        let size = image.size
        return UIGraphicsImageRenderer(size: size).image { context in
            image.draw(in: CGRect(origin: .zero, size: size))
        }
    }

    // MARK: – Validation pipeline

    /// Validate, orient-normalise, and optionally downscale an image for capture submission.
    static func validate(_ image: UIImage) -> ValidationResult {
        // 1. Normalise orientation first — must happen before downscaling so the
        //    scale math operates on the true width/height, not the rotated one.
        let oriented = normalizeOrientation(image)

        // 2. Downscale if either dimension exceeds the maximum.
        let processed: UIImage
        if oriented.size.width > maxDimension || oriented.size.height > maxDimension {
            let scale = min(maxDimension / oriented.size.width,
                           maxDimension / oriented.size.height)
            let newSize = CGSize(width: oriented.size.width * scale,
                                height: oriented.size.height * scale)
            processed = UIGraphicsImageRenderer(size: newSize).image { _ in
                oriented.draw(in: CGRect(origin: .zero, size: newSize))
            }
        } else {
            processed = oriented
        }

        // 3. Check encoded data size.
        guard let pngData = processed.pngData() else {
            return ValidationResult(isValid: false, image: nil,
                                   error: "Failed to generate PNG data")
        }

        if pngData.count > maxDataSize {
            // Try JPEG as fallback for large images.
            if let jpegData = processed.jpegData(compressionQuality: 0.85),
               jpegData.count <= maxDataSize {
                if let jpegImage = UIImage(data: jpegData) {
                    return ValidationResult(isValid: true, image: jpegImage, error: nil)
                }
            }
            return ValidationResult(isValid: false, image: nil,
                                   error: "Image exceeds \(maxDataSize / 1024 / 1024)MB limit")
        }

        return ValidationResult(isValid: true, image: processed, error: nil)
    }
}
