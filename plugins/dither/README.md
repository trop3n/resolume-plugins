# Dither

FFGL effect for Resolume: bayer or noise dithering to a few levels at a chosen pixel size. Effect 15 of the
Video-22575 list.

The look is in [`src/effect.glsl`](src/effect.glsl); `src/lib.rs` declares
the parameters. Sizes are in pixels at 1080p and scale with the output.

## Parameters

| Group | Parameter | Range |
|---|---|---|
| Dither | Pattern | Bayer 2x2, Bayer 4x4, Bayer 8x8, Noise (default Bayer 4x4) |
| Dither | Pixel Size | 1.0 to 16.0, default 2.0 |
| Dither | Levels | 2 to 8, default 2 |
| Dither | Colour | Two Colour, Per Channel (default Two Colour) |
| Tone | Contrast | 0.25 to 3.0, default 1.2 |
| Tone | Brightness | -0.5 to 0.5, default 0.0 |
| Colours | Dark | colour |
| Colours | Light | colour |
| Output | Mix | 0.0 to 1.0, default 1.0 |
