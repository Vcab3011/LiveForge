@preconcurrency import AVFoundation
import CoreMedia
import Foundation

public struct LivePhotoPairBuilder: Sendable {
    private let imageWriter: StillImageWriter
    private let videoWriter: PairedVideoWriter
    private let validator: AppleLivePhotoValidator

    public init(
        imageWriter: StillImageWriter = StillImageWriter(),
        videoWriter: PairedVideoWriter = PairedVideoWriter(),
        validator: AppleLivePhotoValidator = AppleLivePhotoValidator()
    ) {
        self.imageWriter = imageWriter
        self.videoWriter = videoWriter
        self.validator = validator
    }

    public func build(
        sourceVideoURL: URL,
        destinationDirectory: URL,
        stillImageTime: CMTime,
        assetIdentifier: String = UUID().uuidString.uppercased()
    ) async throws -> LivePhotoResources {
        let fileManager = FileManager.default
        let stagingDirectory = destinationDirectory
            .deletingLastPathComponent()
            .appendingPathComponent(".liveforge-\(UUID().uuidString)", isDirectory: true)

        try fileManager.createDirectory(
            at: stagingDirectory,
            withIntermediateDirectories: true
        )

        do {
            let stagedPhoto = stagingDirectory.appendingPathComponent("IMG_LIVEFORGE.JPG")
            let stagedVideo = stagingDirectory.appendingPathComponent("IMG_LIVEFORGE.MOV")
            let image = try extractFrame(from: sourceVideoURL, at: stillImageTime)

            try imageWriter.writeJPEG(
                image,
                assetIdentifier: assetIdentifier,
                to: stagedPhoto
            )
            try await videoWriter.write(
                sourceURL: sourceVideoURL,
                destinationURL: stagedVideo,
                assetIdentifier: assetIdentifier,
                stillImageTime: stillImageTime
            )
            _ = try await validator.validate(
                photoURL: stagedPhoto,
                pairedVideoURL: stagedVideo,
                expectedIdentifier: assetIdentifier,
                expectedStillImageTime: stillImageTime
            )

            guard !fileManager.fileExists(atPath: destinationDirectory.path) else {
                throw LivePhotoAppleError.writerFailed(
                    "Destination already exists: \(destinationDirectory.path)"
                )
            }
            try fileManager.moveItem(at: stagingDirectory, to: destinationDirectory)

            return LivePhotoResources(
                assetIdentifier: assetIdentifier,
                photoURL: destinationDirectory.appendingPathComponent("IMG_LIVEFORGE.JPG"),
                pairedVideoURL: destinationDirectory.appendingPathComponent("IMG_LIVEFORGE.MOV"),
                stillImageTime: stillImageTime
            )
        } catch {
            try? fileManager.removeItem(at: stagingDirectory)
            throw error
        }
    }

    private func extractFrame(from sourceURL: URL, at time: CMTime) throws -> CGImage {
        let generator = AVAssetImageGenerator(asset: AVURLAsset(url: sourceURL))
        generator.appliesPreferredTrackTransform = true
        generator.requestedTimeToleranceBefore = .zero
        generator.requestedTimeToleranceAfter = .zero
        return try generator.copyCGImage(at: time, actualTime: nil)
    }
}
