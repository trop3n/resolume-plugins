# Edge Detect

FFGL effect for Resolume: sobel edges as outlines, scan-line stripes or neon. Effect 13 of the
Video-22575 list.

The look is in [`src/effect.glsl`](src/effect.glsl); `src/lib.rs` declares
the parameters. Sizes are in pixels at 1080p and scale with the output.

## Parameters

| Group | Parameter | Range |
|---|---|---|
| Edges | Mode | Lines, Scan Lines, Neon (default Lines) |
| Edges | Strength | 0.5 to 8.0, default 2.5 |
| Edges | Threshold | 0.0 to 1.0, default 0.15 |
| Edges | Line Width | 0.5 to 4.0, default 1.0 |
| Scan Lines | Line Spacing | 2.0 to 20.0, default 6.0 |
| Colours | Ink | colour |
| Colours | Paper | colour |
| Output | Mix | 0.0 to 1.0, default 1.0 |
