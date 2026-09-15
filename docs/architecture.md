# Architecture

## Goal

LiveForge converts a source video into an Apple-compatible still-image and paired-video resource set without avoidable generation loss.

## Component boundaries

| Crate | Responsibility | Must not own |
|---|---|---|
| `liveforge-core` | Domain types, option validation, conversion planning | Process execution or Apple APIs |
| `liveforge-media` | Probe and eventually process media through replaceable backends | Product policy |
| `liveforge-metadata` | Shared identifiers and metadata-writing contracts | UI or Photos import |
| `liveforge-validator` | Layered validation with evidence | Mutation of outputs |
| `liveforge-cli` | User input, orchestration, machine-readable output | Codec/format rules |

## Planned data flow

1. Probe the source and normalize media metadata.
2. Validate the requested key time and clip duration.
3. Select remux/copy or a single compatibility transcode.
4. Extract the key frame at native dimensions with explicit HDR handling.
5. Encode the still image and embed the shared asset identifier.
6. Remux the motion resource and write the QuickTime content identifier plus timed `still-image-time` metadata.
7. Run basic, structural, decode, and Apple-native validation where supported.
8. Import through PhotoKit on an Apple device.

## Platform strategy

- **Windows** is the primary development and desktop target.
- **Linux** supports the core and CLI for automation and batch use.
- **macOS** adds native validation and Photos import when available.
- **iOS companion** is planned as a narrow importer for Windows-generated pairs.

Platform-specific code must sit behind explicit interfaces. The core format model cannot depend on AVFoundation or PhotoKit.

## Processing invariants

- Both resources use one generated asset identifier.
- Key time is inside the selected clip.
- Passthrough means no video decode/encode; container remuxing may still occur.
- Any required transcode happens once.
- Output claims are derived from measured processing decisions, not presets or target bitrates.
