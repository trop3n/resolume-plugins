# Night Vision

FFGL effect for Resolume: amplified green phosphor image with noise, bloom and vignette. Effect 3 of the
Video-22575 list.

The look is in [`src/effect.glsl`](src/effect.glsl); `src/lib.rs` declares
the parameters. Sizes are in pixels at 1080p and scale with the output.

## Parameters

| Group | Parameter | Range |
|---|---|---|
| Image | Gain | 0.5 to 6.0, default 2.0 |
| Image | Bloom | 0.0 to 1.0, default 0.4 |
| Texture | Noise | 0.0 to 1.0, default 0.3 |
| Texture | Scan Lines | 0.0 to 1.0, default 0.3 |
| Texture | Flicker | 0.0 to 1.0, default 0.2 |
| Texture | Vignette | 0.0 to 1.0, default 0.6 |
| Colour | Phosphor | colour |
| Output | Mix | 0.0 to 1.0, default 1.0 |
