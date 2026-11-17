# Retro LCD

FFGL effect for Resolume: low-res two-tone phone LCD with dither and a pixel grid. Effect 9 of the
Video-22575 list.

The look is in [`src/effect.glsl`](src/effect.glsl); `src/lib.rs` declares
the parameters. Sizes are in pixels at 1080p and scale with the output.

## Parameters

| Group | Parameter | Range |
|---|---|---|
| Screen | Pixel Size | 2.0 to 24.0, default 6.0 |
| Screen | Levels | 2 to 4, default 2 |
| Screen | Dither | toggle, default on |
| Screen | Grid | 0.0 to 1.0, default 0.5 |
| Tone | Contrast | 0.25 to 3.0, default 1.3 |
| Tone | Brightness | -0.5 to 0.5, default 0.0 |
| Colours | Ink | colour |
| Colours | Screen | colour |
| Output | Mix | 0.0 to 1.0, default 1.0 |
