# Contributing to LiveForge

LiveForge is quality-first software that handles personal media. Changes should be small, testable, and explicit about quality or compatibility tradeoffs.

## Local checks

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features
```

Media analysis additionally requires `ffprobe` on `PATH`.

## Pull requests

- Explain the user-visible behavior and failure modes.
- Add unit tests for deterministic logic.
- Add only redistributable, privacy-safe media fixtures.
- Document any codec, HDR, platform, or metadata limitation.
- Never claim lossless output when a stream was decoded and re-encoded.
- Do not introduce network uploads into the conversion path.

## Live Photo compatibility work

Metadata changes should include evidence from generated files and, when possible, a real Apple Photos import test. A command completing successfully is not sufficient evidence of compatibility.
