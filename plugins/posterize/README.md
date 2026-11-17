# Posterize

FFGL effect for Resolume: few tone levels, on a three-colour gradient or per channel. Effect 10 of the
Video-22575 list.

The look is in [`src/effect.glsl`](src/effect.glsl); `src/lib.rs` declares
the parameters. Sizes are in pixels at 1080p and scale with the output.

## Parameters

| Group | Parameter | Range |
|---|---|---|
| Tone | Mode | Gradient, Per Channel (default Gradient) |
| Tone | Levels | 2 to 16, default 4 |
| Tone | Dither | 0.0 to 1.0, default 0.0 |
| Tone | Contrast | 0.25 to 3.0, default 1.2 |
| Tone | Brightness | -0.5 to 0.5, default 0.0 |
| Gradient | Shadows | colour |
| Gradient | Midtones | colour |
| Gradient | Highlights | colour |
| Output | Mix | 0.0 to 1.0, default 1.0 |
