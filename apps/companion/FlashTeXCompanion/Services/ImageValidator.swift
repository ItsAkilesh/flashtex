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
        let encodedData: Data?
        let mimeType: String?
        let error: String?
    }

    /// Validate and optionally downscale an image for capture submission.
    static func validate(_ image: UIImage) -> ValidationResult {
        // Check dimensions and downscale if needed
        let processed: UIImage
        if image.size.width > maxDimension || image.size.height > maxDimension {
            let scale = min(maxDimension / image.size.width,
                           maxDimension / image.size.height)
            let newSize = CGSize(width: image.size.width * scale,
                                height: image.size.height * scale)
            processed = UIGraphicsImageRenderer(size: newSize).image { _ in
                image.draw(in: CGRect(origin: .zero, size: newSize))
            }
        } else {
            processed = image
        }

        // Check data size
        guard let pngData = processed.pngData() else {
            return ValidationResult(isValid: false, image: nil, encodedData: nil, mimeType: nil,
                                   error: "Failed to generate PNG data")
        }

        if pngData.count > maxDataSize {
            // Try JPEG as fallback for large images
            if let jpegData = processed.jpegData(compressionQuality: 0.85),
               jpegData.count <= maxDataSize {
                // Reconstruct as JPEG-backed UIImage
                if let jpegImage = UIImage(data: jpegData) {
                    return ValidationResult(
                        isValid: true,
                        image: jpegImage,
                        encodedData: jpegData,
                        mimeType: "image/jpeg",
                        error: nil
                    )
                }
            }
            return ValidationResult(isValid: false, image: nil, encodedData: nil, mimeType: nil,
                                   error: "Image exceeds \(maxDataSize / 1024 / 1024)MB limit")
        }

        return ValidationResult(
            isValid: true,
            image: processed,
            encodedData: pngData,
            mimeType: "image/png",
            error: nil
        )
    }
}
