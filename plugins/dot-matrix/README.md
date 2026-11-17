# Dot Matrix

FFGL effect for Resolume: grid of dots sized by brightness, printer or LED style. Effect 20 of the
Video-22575 list.

The look is in [`src/effect.glsl`](src/effect.glsl); `src/lib.rs` declares
the parameters. Sizes are in pixels at 1080p and scale with the output.

## Parameters

| Group | Parameter | Range |
|---|---|---|
| Dots | Style | Print, LED (default Print) |
| Dots | Shape | Round, Square (default Round) |
| Dots | Cell Size | 3.0 to 48.0, default 12.0 |
| Dots | Min Size | 0.0 to 1.0, default 0.15 |
| Dots | Glow | 0.0 to 1.0, default 0.5 |
| Tone | Contrast | 0.25 to 3.0, default 1.2 |
| Tone | Brightness | -0.5 to 0.5, default 0.0 |
| Colours | Image Colour | toggle, default off |
| Colours | Dot | colour |
| Colours | Background | colour |
| Output | Mix | 0.0 to 1.0, default 1.0 |
