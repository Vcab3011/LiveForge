@preconcurrency import AVFoundation
import CoreVideo
import Foundation

enum SyntheticVideoFactory {
    static func makeVideo(at url: URL, frameCount: Int = 90, framesPerSecond: Int32 = 30) async throws {
        let width = 96
        let height = 64
        let writer = try AVAssetWriter(outputURL: url, fileType: .mov)
        let input = AVAssetWriterInput(
            mediaType: .video,
            outputSettings: [
                AVVideoCodecKey: AVVideoCodecType.h264,
                AVVideoWidthKey: width,
                AVVideoHeightKey: height,
            ]
        )
        input.expectsMediaDataInRealTime = false

        let adaptor = AVAssetWriterInputPixelBufferAdaptor(
            assetWriterInput: input,
            sourcePixelBufferAttributes: [
                kCVPixelBufferPixelFormatTypeKey as String: kCVPixelFormatType_32BGRA,
                kCVPixelBufferWidthKey as String: width,
                kCVPixelBufferHeightKey as String: height,
            ]
        )
        guard writer.canAdd(input) else {
            throw LivePhotoAppleError.cannotAddWriterInput("synthetic video")
        }
        writer.add(input)
        guard writer.startWriting() else {
            throw LivePhotoAppleError.writerFailed(
                writer.error?.localizedDescription ?? "synthetic video start failed"
            )
        }
        writer.startSession(atSourceTime: .zero)

        for frame in 0..<frameCount {
            while !input.isReadyForMoreMediaData {
                try await Task.sleep(nanoseconds: 1_000_000)
            }
            guard let pool = adaptor.pixelBufferPool else {
                throw LivePhotoAppleError.writerFailed("Synthetic pixel-buffer pool is unavailable.")
            }
            var optionalBuffer: CVPixelBuffer?
            let status = CVPixelBufferPoolCreatePixelBuffer(nil, pool, &optionalBuffer)
            guard status == kCVReturnSuccess, let buffer = optionalBuffer else {
                throw LivePhotoAppleError.writerFailed("Could not allocate a synthetic frame.")
            }

            fill(buffer, frame: frame)
            let presentationTime = CMTime(value: Int64(frame), timescale: framesPerSecond)
            guard adaptor.append(buffer, withPresentationTime: presentationTime) else {
                throw LivePhotoAppleError.writerFailed(
                    writer.error?.localizedDescription ?? "synthetic frame append failed"
                )
            }
        }

        input.markAsFinished()
        try await withCheckedThrowingContinuation {
            (continuation: CheckedContinuation<Void, Error>) in
            writer.finishWriting {
                if writer.status == .completed {
                    continuation.resume()
                } else {
                    continuation.resume(
                        throwing: LivePhotoAppleError.writerFailed(
                            writer.error?.localizedDescription ?? "synthetic video finish failed"
                        )
                    )
                }
            }
        }
    }

    private static func fill(_ buffer: CVPixelBuffer, frame: Int) {
        CVPixelBufferLockBaseAddress(buffer, [])
        defer { CVPixelBufferUnlockBaseAddress(buffer, []) }
        guard let baseAddress = CVPixelBufferGetBaseAddress(buffer) else { return }

        let byteCount = CVPixelBufferGetBytesPerRow(buffer) * CVPixelBufferGetHeight(buffer)
        let color = UInt8((frame * 7) % 255)
        memset(baseAddress, Int32(color), byteCount)
    }
}
