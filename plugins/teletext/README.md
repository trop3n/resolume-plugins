# Teletext

FFGL effect for Resolume: ceefax-style mosaic blocks in the eight teletext colours. Effect 18 of the
Video-22575 list.

The look is in [`src/effect.glsl`](src/effect.glsl); `src/lib.rs` declares
the parameters. Sizes are in pixels at 1080p and scale with the output.

## Parameters

| Group | Parameter | Range |
|---|---|---|
| Mosaic | Mode | Free, One Colour per Cell (default Free) |
| Mosaic | Columns | 20 to 120, default 40 |
| Mosaic | Separated | toggle, default off |
| Mosaic | Dither | 0.0 to 1.0, default 0.3 |
| Tone | Contrast | 0.25 to 3.0, default 1.2 |
| Tone | Saturation | 0.0 to 3.0, default 1.6 |
| Tone | Scan Lines | 0.0 to 1.0, default 0.3 |
| Colours | Background | colour |
| Output | Mix | 0.0 to 1.0, default 1.0 |
