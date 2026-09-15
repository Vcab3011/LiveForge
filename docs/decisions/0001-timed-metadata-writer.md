# ADR 0001: Treat Live Photo timed metadata as a dedicated track

## Status

Accepted.

## Context

The paired MOV needs both top-level `com.apple.quicktime.content.identifier` metadata and a timed `mdta/com.apple.quicktime.still-image-time` sample. The timing comes from the metadata sample's presentation time; it is not equivalent to a normal file-level tag.

FFmpeg can remux compatible video and audio and can write ordinary QuickTime metadata. It does not reliably create or preserve the required boxed `mebx` timed metadata track. Treating a global float tag as a substitute would let commands appear successful without proving Apple compatibility.

Apple's AVFoundation model constructs a boxed Core Media metadata format, adds an `AVAssetWriterInput` of media type metadata, and appends an `AVTimedMetadataGroup` at the selected still-image time.

## Decision

- Model the still-image timestamp as a timed metadata sample.
- Keep pair orchestration independent from the eventual MOV backend.
- Require the still writer, motion writer, and verifier to receive the same immutable `MetadataPlan`.
- Publish generated resources only after verification succeeds.
- Do not advertise FFmpeg-only metadata stamping as a valid Live Photo writer.

The first implementation adds a transactional `liveforge-pair` crate and structural validation rules. A following backend will implement the actual QuickTime metadata track and provide binary fixtures from its output.

## References

- [Apple QuickTime File Format: Timed metadata media](https://developer.apple.com/documentation/quicktime-file-format/timed_metadata_media)
- [Apple: Capturing and saving Live Photos](https://developer.apple.com/documentation/avfoundation/capturing-and-saving-live-photos)
- [LimitPoint LivePhoto format notes](https://github.com/LimitPoint/LivePhoto#live-photo-format)
