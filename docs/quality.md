# Quality policy

## Original quality

If the selected stream, container, trim boundary, and metadata muxing path are compatible, LiveForge remuxes the original compressed video stream. The output may use a new container, timestamps, and metadata while preserving the compressed video packets.

The UI may report **No video recompression** only when telemetry confirms that no video encoder ran.

## Maximum compatibility

An incompatible source is decoded and encoded once to an Apple-compatible format. The resulting file is not lossless even if its bitrate exceeds the source bitrate.

## Enhanced output

Super-resolution or other generated detail must be labeled **Enhanced**, never Original. A 4K video frame contains approximately 8.3 megapixels; upscaling cannot recover detail that was never present in the source.

## HDR

The analyzer records codec, pixel format, bit depth, color primaries, transfer characteristics, and matrix coefficients. Until a path is verified to preserve the complete required metadata and signal, LiveForge must not claim HDR preservation.

## Quality evidence

Every completed conversion should eventually expose:

- source and output stream properties;
- whether video/audio were copied or encoded;
- encoder and parameters when used;
- key-frame timestamp and clip bounds;
- validation level reached;
- warnings for unsupported or unverified properties.
