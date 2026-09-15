# LiveForge

**The quality-first, local-first Apple Live Photo converter.**

LiveForge is an open-source toolkit for turning ordinary videos into Apple Live Photo resources while preserving source quality whenever technically possible.

> Local processing. No uploads. No subscriptions. No unnecessary transcoding.

## Project status

LiveForge is in the **foundation phase**. The repository currently provides a cross-platform Rust workspace, an `ffprobe`-based media analyzer, a deterministic conversion planner, metadata contracts, a basic pair validator, and a first-class CLI shell. It does **not yet write production Live Photo metadata or import assets into Apple Photos**.

| Capability | Status |
|---|---|
| Inspect video/container/HDR metadata | Foundation available |
| Decide copy vs. one-time transcode | Foundation available |
| Generate a shared Live Photo asset identifier | Foundation available |
| Basic HEIC/MOV pair checks | Foundation available |
| Write image asset identifier | Planned |
| Write QuickTime content identifier | Planned |
| Write timed `still-image-time` metadata | Planned |
| Structural metadata validation | Planned |
| Windows desktop GUI | Planned |
| iOS Photos importer | Planned |

## Design principles

1. **Copy first** — remux compatible video streams; encode only when required.
2. **Verify everything** — a successful export is not proof of a valid Live Photo.
3. **Local by default** — user media never needs to leave the computer.
4. **Windows-first development** — the core, CLI, and desktop app remain cross-platform; Apple-native import is an optional integration.
5. **Honest quality reporting** — distinguish preserved source detail from generated or enhanced output.

## Architecture

```text
Video
  -> liveforge-media      analyze streams with ffprobe
  -> liveforge-core       validate options and choose a processing strategy
  -> liveforge-metadata   construct the shared Live Photo metadata contract
  -> media pipeline       remux or transcode once (planned)
  -> liveforge-validator  validate the generated pair
  -> Apple Photos         import through an iOS/macOS helper (planned)
```

See [Architecture](docs/architecture.md), [Live Photo format](docs/live-photo-format.md), [Quality policy](docs/quality.md), and the [Roadmap](docs/roadmap.md).

## CLI foundation

Requirements:

- Rust 1.78 or newer
- FFmpeg/`ffprobe` available on `PATH` for media analysis

```bash
cargo run -p liveforge-cli -- analyze input.mov

cargo run -p liveforge-cli -- plan input.mov \
  --key-time 7.8 \
  --duration 3 \
  --quality original

cargo run -p liveforge-cli -- verify output/IMG_0001.HEIC output/IMG_0001.MOV
```

The `verify` command currently performs basic resource checks only and reports that structural Apple metadata validation is not implemented yet.

## Repository layout

```text
crates/
  liveforge-core/       domain models and conversion planning
  liveforge-media/      ffprobe-backed media inspection
  liveforge-metadata/   Live Photo identifiers and metadata contracts
  liveforge-validator/  layered validation reports
  liveforge-cli/        command-line interface
docs/                   format, architecture, quality, and roadmap
.github/workflows/      cross-platform checks
```

## Contributing

The format writer and validator must be built from reproducible tests and documented fixtures. Do not add a bundled “magic template” MOV as the primary metadata strategy. Read [CONTRIBUTING.md](CONTRIBUTING.md) before opening a pull request.

## License

LiveForge is licensed under the [MIT License](LICENSE).
