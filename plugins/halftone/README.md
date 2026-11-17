# Halftone

FFGL effect for Resolume: rotated halftone screen of dots, lines or squares, mono or CMYK. Effect 12 of the
Video-22575 list.

The look is in [`src/effect.glsl`](src/effect.glsl); `src/lib.rs` declares
the parameters. Sizes are in pixels at 1080p and scale with the output.

## Parameters

| Group | Parameter | Range |
|---|---|---|
| Screen | Mode | Mono, CMYK (default Mono) |
| Screen | Shape | Circle, Line, Square (default Circle) |
| Screen | Cell Size | 3.0 to 60.0, default 10.0 |
| Screen | Angle | 0.0 to 90.0, default 45.0 |
| Tone | Contrast | 0.25 to 3.0, default 1.1 |
| Tone | Brightness | -0.5 to 0.5, default 0.0 |
| Colours | Ink | colour |
| Colours | Paper | colour |
| Output | Mix | 0.0 to 1.0, default 1.0 |
