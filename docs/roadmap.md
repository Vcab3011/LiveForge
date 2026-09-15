# Roadmap

## Phase 0 — Foundation

- [x] Cross-platform workspace and CI
- [x] Normalized media information model
- [x] `ffprobe` analyzer
- [x] Deterministic conversion planner
- [x] Metadata contracts and layered validation model
- [x] Transactional resource-pair builder
- [x] CLI commands for analyze, plan, and basic verify

## Phase 1 — First valid pair

- [ ] Frame-accurate still extraction
- [ ] HEIC/JPEG asset identifier writer
- [ ] MOV content identifier writer
- [ ] Timed `still-image-time` metadata writer
- [ ] Three-second Apple-like preset
- [ ] Structural validator
- [ ] Golden, synthetic compatibility fixtures

## Phase 2 — Quality pipeline

- [ ] Packet-copy/remux path with verifiable telemetry
- [ ] One-time HEVC/H.264 compatibility transcode
- [ ] Audio preservation and validation
- [ ] Rotation/orientation normalization policy
- [ ] HDR10/HLG/Dolby Vision detection matrix
- [ ] Batch CLI

## Phase 3 — User applications

- [ ] Windows desktop editor
- [ ] Frame-level timeline and key-photo picker
- [ ] Processing evidence and validation report UI
- [ ] iOS PhotoKit importer
- [ ] macOS native importer/validator

## Phase 4 — Smart selection

- [ ] Local best-frame scoring
- [ ] Face sharpness, eyes-open, exposure, and motion-blur signals
- [ ] Optional local enhancement with honest labeling
