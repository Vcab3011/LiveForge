import AVFoundation
import XCTest
@testable import LiveForgeApple

final class LivePhotoPairBuilderTests: XCTestCase {
    func testBuildsAndReadsBackARealMetadataPair() async throws {
        let root = FileManager.default.temporaryDirectory
            .appendingPathComponent("liveforge-test-\(UUID().uuidString)", isDirectory: true)
        try FileManager.default.createDirectory(at: root, withIntermediateDirectories: true)
        defer { try? FileManager.default.removeItem(at: root) }

        let source = root.appendingPathComponent("source.mov")
        try await SyntheticVideoFactory.makeVideo(at: source)

        let identifier = "6E9727DA-D1A5-4E86-AE90-77D389C6302A"
        let stillTime = CMTime(seconds: 1, preferredTimescale: 600)
        let resources = try await LivePhotoPairBuilder().build(
            sourceVideoURL: source,
            destinationDirectory: root.appendingPathComponent("pair", isDirectory: true),
            stillImageTime: stillTime,
            assetIdentifier: identifier
        )

        XCTAssertTrue(FileManager.default.fileExists(atPath: resources.photoURL.path))
        XCTAssertTrue(FileManager.default.fileExists(atPath: resources.pairedVideoURL.path))

        let report = try await AppleLivePhotoValidator().validate(
            photoURL: resources.photoURL,
            pairedVideoURL: resources.pairedVideoURL,
            expectedIdentifier: identifier,
            expectedStillImageTime: stillTime
        )
        XCTAssertEqual(report.assetIdentifier, identifier)
        XCTAssertEqual(CMTimeGetSeconds(report.stillImageTime), 1, accuracy: 1.0 / 300.0)
    }

    func testRejectsStillTimeOutsideMovie() async throws {
        let root = FileManager.default.temporaryDirectory
            .appendingPathComponent("liveforge-test-\(UUID().uuidString)", isDirectory: true)
        try FileManager.default.createDirectory(at: root, withIntermediateDirectories: true)
        defer { try? FileManager.default.removeItem(at: root) }

        let source = root.appendingPathComponent("source.mov")
        try await SyntheticVideoFactory.makeVideo(at: source, frameCount: 30)

        do {
            _ = try await LivePhotoPairBuilder().build(
                sourceVideoURL: source,
                destinationDirectory: root.appendingPathComponent("pair", isDirectory: true),
                stillImageTime: CMTime(seconds: 5, preferredTimescale: 600)
            )
            XCTFail("Expected invalidStillTime")
        } catch LivePhotoAppleError.invalidStillTime {
            XCTAssertFalse(FileManager.default.fileExists(atPath: root.appendingPathComponent("pair").path))
        }
    }
}
