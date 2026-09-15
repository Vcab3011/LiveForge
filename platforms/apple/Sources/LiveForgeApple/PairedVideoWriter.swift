@preconcurrency import AVFoundation
import CoreMedia
import Foundation

public struct PairedVideoWriter: Sendable {
    private static let stillImageTimeKey = "com.apple.quicktime.still-image-time"
    private static let stillImageTimeIdentifier = "mdta/\(stillImageTimeKey)"

    public init() {}

    public func write(
        sourceURL: URL,
        destinationURL: URL,
        assetIdentifier: String,
        stillImageTime: CMTime
    ) async throws {
        let asset = AVURLAsset(url: sourceURL)
        let duration = try await asset.load(.duration)
        guard stillImageTime >= .zero, stillImageTime < duration else {
            throw LivePhotoAppleError.invalidStillTime
        }

        let videoTracks = try await asset.loadTracks(withMediaType: .video)
        guard let videoTrack = videoTracks.first else {
            throw LivePhotoAppleError.missingVideoTrack
        }

        let reader = try AVAssetReader(asset: asset)
        let writer = try AVAssetWriter(outputURL: destinationURL, fileType: .mov)

        let videoPair = try await addPassthroughTrack(
            videoTrack,
            mediaType: .video,
            name: "video",
            reader: reader,
            writer: writer
        )

        var passthroughPairs = [videoPair]
        if let audioTrack = try await asset.loadTracks(withMediaType: .audio).first {
            passthroughPairs.append(
                try await addPassthroughTrack(
                    audioTrack,
                    mediaType: .audio,
                    name: "audio",
                    reader: reader,
                    writer: writer
                )
            )
        }

        let metadataAdaptor = try makeStillImageTimeAdaptor()
        guard writer.canAdd(metadataAdaptor.assetWriterInput) else {
            throw LivePhotoAppleError.cannotAddWriterInput("metadata")
        }
        writer.add(metadataAdaptor.assetWriterInput)
        writer.metadata = [makeAssetIdentifierItem(assetIdentifier)]

        guard writer.startWriting() else {
            throw writerError(writer)
        }
        writer.startSession(atSourceTime: .zero)

        let metadataGroup = AVTimedMetadataGroup(
            items: [makeStillImageTimeItem()],
            timeRange: CMTimeRange(
                start: stillImageTime,
                duration: CMTime(value: 1, timescale: 600)
            )
        )
        guard metadataAdaptor.append(metadataGroup) else {
            writer.cancelWriting()
            throw writerError(writer)
        }
        metadataAdaptor.assetWriterInput.markAsFinished()

        guard reader.startReading() else {
            writer.cancelWriting()
            throw readerError(reader)
        }

        do {
            try await withThrowingTaskGroup(of: Void.self) { group in
                for pair in passthroughPairs {
                    group.addTask {
                        try await copySamples(
                            from: pair.output,
                            to: pair.input,
                            reader: reader,
                            writer: writer
                        )
                    }
                }
                try await group.waitForAll()
            }
        } catch {
            reader.cancelReading()
            writer.cancelWriting()
            throw error
        }

        guard reader.status == .completed else {
            writer.cancelWriting()
            throw readerError(reader)
        }

        try await finish(writer)
    }

    private func addPassthroughTrack(
        _ track: AVAssetTrack,
        mediaType: AVMediaType,
        name: String,
        reader: AVAssetReader,
        writer: AVAssetWriter
    ) async throws -> (output: AVAssetReaderTrackOutput, input: AVAssetWriterInput) {
        let formatDescriptions = try await track.load(.formatDescriptions)
        let output = AVAssetReaderTrackOutput(track: track, outputSettings: nil)
        guard reader.canAdd(output) else {
            throw LivePhotoAppleError.cannotAddReaderOutput(name)
        }
        reader.add(output)

        let input = AVAssetWriterInput(
            mediaType: mediaType,
            outputSettings: nil,
            sourceFormatHint: formatDescriptions.first
        )
        input.expectsMediaDataInRealTime = false
        guard writer.canAdd(input) else {
            throw LivePhotoAppleError.cannotAddWriterInput(name)
        }
        writer.add(input)
        return (output, input)
    }

    private func copySamples(
        from output: AVAssetReaderOutput,
        to input: AVAssetWriterInput,
        reader: AVAssetReader,
        writer: AVAssetWriter
    ) async throws {
        try await withCheckedThrowingContinuation {
            (continuation: CheckedContinuation<Void, Error>) in
            let queue = DispatchQueue(label: "dev.liveforge.apple.copy.\(UUID().uuidString)")
            var completed = false

            input.requestMediaDataWhenReady(on: queue) {
                guard !completed else { return }

                while input.isReadyForMoreMediaData {
                    if let sample = output.copyNextSampleBuffer() {
                        guard input.append(sample) else {
                            completed = true
                            input.markAsFinished()
                            continuation.resume(throwing: writerError(writer))
                            return
                        }
                    } else {
                        completed = true
                        input.markAsFinished()
                        if reader.status == .failed {
                            continuation.resume(throwing: readerError(reader))
                        } else {
                            continuation.resume()
                        }
                        return
                    }
                }
            }
        }
    }

    private func finish(_ writer: AVAssetWriter) async throws {
        try await withCheckedThrowingContinuation {
            (continuation: CheckedContinuation<Void, Error>) in
            writer.finishWriting {
                if writer.status == .completed {
                    continuation.resume()
                } else {
                    continuation.resume(throwing: writerError(writer))
                }
            }
        }
    }

    private func makeAssetIdentifierItem(_ identifier: String) -> AVMetadataItem {
        let item = AVMutableMetadataItem()
        item.key = AVMetadataKey.quickTimeMetadataKeyContentIdentifier as NSString
        item.keySpace = .quickTimeMetadata
        item.value = identifier as NSString
        item.dataType = kCMMetadataBaseDataType_UTF8 as String
        return item
    }

    private func makeStillImageTimeAdaptor() throws -> AVAssetWriterInputMetadataAdaptor {
        let specification: NSDictionary = [
            kCMMetadataFormatDescriptionMetadataSpecificationKey_Identifier as NSString:
                Self.stillImageTimeIdentifier,
            kCMMetadataFormatDescriptionMetadataSpecificationKey_DataType as NSString:
                kCMMetadataBaseDataType_SInt8,
        ]
        var description: CMMetadataFormatDescription?
        let status = CMMetadataFormatDescriptionCreateWithMetadataSpecifications(
            allocator: kCFAllocatorDefault,
            metadataType: kCMMetadataFormatType_Boxed,
            metadataSpecifications: [specification] as CFArray,
            formatDescriptionOut: &description
        )
        guard status == noErr, let description else {
            throw LivePhotoAppleError.writerFailed("Could not create the metadata format description (\(status)).")
        }

        let input = AVAssetWriterInput(
            mediaType: .metadata,
            outputSettings: nil,
            sourceFormatHint: description
        )
        return AVAssetWriterInputMetadataAdaptor(assetWriterInput: input)
    }

    private func makeStillImageTimeItem() -> AVMetadataItem {
        let item = AVMutableMetadataItem()
        item.key = Self.stillImageTimeKey as NSString
        item.keySpace = .quickTimeMetadata
        item.value = NSNumber(value: Int8(-1))
        item.dataType = kCMMetadataBaseDataType_SInt8 as String
        return item
    }

    private func readerError(_ reader: AVAssetReader) -> LivePhotoAppleError {
        .readerFailed(reader.error?.localizedDescription ?? "unknown error")
    }

    private func writerError(_ writer: AVAssetWriter) -> LivePhotoAppleError {
        .writerFailed(writer.error?.localizedDescription ?? "unknown error")
    }
}
