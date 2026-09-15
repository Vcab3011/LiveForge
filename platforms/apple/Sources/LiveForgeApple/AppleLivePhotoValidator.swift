@preconcurrency import AVFoundation
import CoreMedia
import Foundation
import ImageIO

public struct AppleLivePhotoValidator: Sendable {
    private static let movieIdentifier = "mdta/com.apple.quicktime.content.identifier"
    private static let stillImageTimeKey = "com.apple.quicktime.still-image-time"

    public init() {}

    public func validate(
        photoURL: URL,
        pairedVideoURL: URL,
        expectedIdentifier: String? = nil,
        expectedStillImageTime: CMTime? = nil,
        tolerance: CMTime = CMTime(value: 1, timescale: 300)
    ) async throws -> AppleValidationReport {
        guard let imageIdentifier = readImageIdentifier(from: photoURL) else {
            throw LivePhotoAppleError.imageMetadataMissing
        }

        let asset = AVURLAsset(url: pairedVideoURL)
        let metadata = try await asset.load(.metadata)
        guard let movieIdentifierItem = metadata.first(where: {
            $0.identifier?.rawValue == Self.movieIdentifier
        }), let movieIdentifier = try await movieIdentifierItem.load(.stringValue) else {
            throw LivePhotoAppleError.movieIdentifierMissing
        }

        guard imageIdentifier == movieIdentifier,
              expectedIdentifier.map({ $0 == imageIdentifier }) ?? true
        else {
            throw LivePhotoAppleError.movieIdentifierMismatch
        }

        guard let actualStillImageTime = try await readStillImageTime(from: asset) else {
            throw LivePhotoAppleError.stillImageTimeMissing
        }

        if let expectedStillImageTime {
            let difference = abs(
                CMTimeGetSeconds(actualStillImageTime) - CMTimeGetSeconds(expectedStillImageTime)
            )
            guard difference <= CMTimeGetSeconds(tolerance) else {
                throw LivePhotoAppleError.stillImageTimeMismatch(
                    expected: CMTimeGetSeconds(expectedStillImageTime),
                    actual: CMTimeGetSeconds(actualStillImageTime)
                )
            }
        }

        return AppleValidationReport(
            assetIdentifier: imageIdentifier,
            stillImageTime: actualStillImageTime
        )
    }

    private func readImageIdentifier(from url: URL) -> String? {
        guard let source = CGImageSourceCreateWithURL(url as CFURL, nil),
              let properties = CGImageSourceCopyPropertiesAtIndex(source, 0, nil)
                as? [CFString: Any],
              let makerApple = properties[kCGImagePropertyMakerAppleDictionary]
                as? [String: Any]
        else {
            return nil
        }
        return makerApple["17"] as? String
    }

    private func readStillImageTime(from asset: AVAsset) async throws -> CMTime? {
        for track in try await asset.loadTracks(withMediaType: .metadata) {
            let reader = try AVAssetReader(asset: asset)
            let output = AVAssetReaderTrackOutput(track: track, outputSettings: nil)
            guard reader.canAdd(output) else { continue }
            reader.add(output)
            guard reader.startReading() else { throw readerError(reader) }

            while let sample = output.copyNextSampleBuffer() {
                guard let group = AVTimedMetadataGroup(sampleBuffer: sample) else { continue }
                let containsStillImageTime = group.items.contains { item in
                    item.keySpace == .quickTimeMetadata
                        && (item.key as? String) == Self.stillImageTimeKey
                        && item.dataType == kCMMetadataBaseDataType_SInt8 as String
                }
                if containsStillImageTime {
                    reader.cancelReading()
                    return group.timeRange.start
                }
            }

            if reader.status == .failed {
                throw readerError(reader)
            }
        }
        return nil
    }

    private func readerError(_ reader: AVAssetReader) -> LivePhotoAppleError {
        .readerFailed(reader.error?.localizedDescription ?? "unknown error")
    }
}
