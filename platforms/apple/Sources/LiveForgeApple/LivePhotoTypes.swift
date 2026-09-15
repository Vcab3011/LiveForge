import CoreMedia
import Foundation

public enum LivePhotoAppleError: Error, LocalizedError {
    case invalidStillTime
    case missingVideoTrack
    case cannotAddReaderOutput(String)
    case cannotAddWriterInput(String)
    case readerFailed(String)
    case writerFailed(String)
    case imageEncodingFailed
    case imageMetadataMissing
    case movieIdentifierMissing
    case movieIdentifierMismatch
    case stillImageTimeMissing
    case stillImageTimeMismatch(expected: Double, actual: Double)

    public var errorDescription: String? {
        switch self {
        case .invalidStillTime:
            return "The still-image time must be inside the movie duration."
        case .missingVideoTrack:
            return "The source asset does not contain a video track."
        case let .cannotAddReaderOutput(name):
            return "AVAssetReader cannot add the \(name) output."
        case let .cannotAddWriterInput(name):
            return "AVAssetWriter cannot add the \(name) input."
        case let .readerFailed(message):
            return "AVAssetReader failed: \(message)"
        case let .writerFailed(message):
            return "AVAssetWriter failed: \(message)"
        case .imageEncodingFailed:
            return "ImageIO could not write the key photo."
        case .imageMetadataMissing:
            return "The key photo does not contain MakerApple asset identifier 17."
        case .movieIdentifierMissing:
            return "The paired movie does not contain a QuickTime content identifier."
        case .movieIdentifierMismatch:
            return "The image and movie asset identifiers do not match."
        case .stillImageTimeMissing:
            return "The paired movie does not contain a still-image-time metadata sample."
        case let .stillImageTimeMismatch(expected, actual):
            return "The still-image-time is \(actual)s; expected \(expected)s."
        }
    }
}

public struct LivePhotoResources: Sendable, Equatable {
    public let assetIdentifier: String
    public let photoURL: URL
    public let pairedVideoURL: URL
    public let stillImageTime: CMTime

    public init(
        assetIdentifier: String,
        photoURL: URL,
        pairedVideoURL: URL,
        stillImageTime: CMTime
    ) {
        self.assetIdentifier = assetIdentifier
        self.photoURL = photoURL
        self.pairedVideoURL = pairedVideoURL
        self.stillImageTime = stillImageTime
    }
}

public struct AppleValidationReport: Sendable, Equatable {
    public let assetIdentifier: String
    public let stillImageTime: CMTime

    public init(assetIdentifier: String, stillImageTime: CMTime) {
        self.assetIdentifier = assetIdentifier
        self.stillImageTime = stillImageTime
    }
}
