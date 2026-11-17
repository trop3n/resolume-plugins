# ASCII

FFGL effect for Resolume: redraws the clip as text characters or shade blocks. Effect 6 of the
Video-22575 list.

The look is in [`src/effect.glsl`](src/effect.glsl); `src/lib.rs` declares
the parameters. Sizes are in pixels at 1080p and scale with the output.

## Parameters

| Group | Parameter | Range |
|---|---|---|
| Text | Charset | Ramp, Blocks (default Ramp) |
| Text | Char Size | 6.0 to 64.0, default 16.0 |
| Text | Invert | toggle, default off |
| Tone | Contrast | 0.25 to 3.0, default 1.3 |
| Tone | Brightness | -0.5 to 0.5, default 0.0 |
| Colours | Image Colour | toggle, default off |
| Colours | Text | colour |
| Colours | Background | colour |
| Output | Mix | 0.0 to 1.0, default 1.0 |
