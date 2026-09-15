# Apple Live Photo resource model

A Live Photo is not merely two files with matching names. LiveForge models it as a linked resource pair:

```text
Still image
  embedded asset identifier: <UUID>

Paired MOV
  com.apple.quicktime.content.identifier: <same UUID>
  timed metadata: com.apple.quicktime.still-image-time
```

The still-image timestamp must fall inside the motion resource duration and correspond to the intended key photo.

## Validation layers

1. **Basic** — paths, readable files, expected extensions, non-empty resources.
2. **Structural** — identifiers exist and match; timed metadata exists and is in range.
3. **Decode** — image, video, audio, orientation, dimensions, and color metadata are readable.
4. **Apple-native** — `PHLivePhoto` loads and PhotoKit can create one asset from `.photo` plus `.pairedVideo`.

Windows and Linux can complete the first three layers once the writers and parsers are implemented. Apple-native validation requires an Apple runtime.

## Research rule

Implementation must construct and inspect the required metadata directly. Template resources may be used only as documented research fixtures, never as an unexplained production dependency.
