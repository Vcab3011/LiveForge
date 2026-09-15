import CoreGraphics
import Foundation
import ImageIO
import UniformTypeIdentifiers

public struct StillImageWriter: Sendable {
    public init() {}

    public func writeJPEG(_ image: CGImage, assetIdentifier: String, to destination: URL) throws {
        guard let writer = CGImageDestinationCreateWithURL(
            destination as CFURL,
            UTType.jpeg.identifier as CFString,
            1,
            nil
        ) else {
            throw LivePhotoAppleError.imageEncodingFailed
        }

        let properties: [CFString: Any] = [
            kCGImagePropertyMakerAppleDictionary: ["17": assetIdentifier],
            kCGImageDestinationLossyCompressionQuality: 1.0,
        ]
        CGImageDestinationAddImage(writer, image, properties as CFDictionary)

        guard CGImageDestinationFinalize(writer) else {
            throw LivePhotoAppleError.imageEncodingFailed
        }
    }
}
