# Threshold

FFGL effect for Resolume: hard two-colour cut at a brightness level, with photocopy grit. Effect 5 of the
Video-22575 list.

The look is in [`src/effect.glsl`](src/effect.glsl); `src/lib.rs` declares
the parameters. Sizes are in pixels at 1080p and scale with the output.

## Parameters

| Group | Parameter | Range |
|---|---|---|
| Cut | Level | 0.0 to 1.0, default 0.5 |
| Cut | Softness | 0.0 to 0.25, default 0.0 |
| Cut | Invert | toggle, default off |
| Grit | Noise | 0.0 to 0.6, default 0.1 |
| Grit | Grain Size | 1.0 to 8.0, default 1.5 |
| Grit | Boil | 0.0 to 1.0, default 0.0 |
| Colours | Dark | colour |
| Colours | Light | colour |
| Output | Mix | 0.0 to 1.0, default 1.0 |
