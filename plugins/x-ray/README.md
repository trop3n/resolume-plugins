# X-Ray

FFGL effect for Resolume: inverted, high-contrast x-ray with glowing edges, grain and scan lines. Effect 2 of the
Video-22575 list.

The look is in [`src/effect.glsl`](src/effect.glsl); `src/lib.rs` declares
the parameters. Sizes are in pixels at 1080p and scale with the output.

## Parameters

| Group | Parameter | Range |
|---|---|---|
| Tone | Contrast | 0.5 to 4.0, default 1.6 |
| Tone | Brightness | -0.5 to 0.5, default -0.05 |
| Tone | Edge Glow | 0.0 to 2.0, default 0.6 |
| Texture | Grain | 0.0 to 1.0, default 0.2 |
| Texture | Scan Lines | 0.0 to 1.0, default 0.3 |
| Colour | Tint | colour |
| Output | Mix | 0.0 to 1.0, default 1.0 |
