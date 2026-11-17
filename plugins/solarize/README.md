# Solarize

FFGL effect for Resolume: tones past a threshold fold back, in rainbow, colour or mono. Effect 11 of the
Video-22575 list.

The look is in [`src/effect.glsl`](src/effect.glsl); `src/lib.rs` declares
the parameters. Sizes are in pixels at 1080p and scale with the output.

## Parameters

| Group | Parameter | Range |
|---|---|---|
| Solarize | Palette | Rainbow, Colour, Mono (default Rainbow) |
| Solarize | Threshold | 0.05 to 0.95, default 0.5 |
| Solarize | Amount | 0.0 to 1.0, default 1.0 |
| Rainbow | Cycles | 0.25 to 4.0, default 1.0 |
| Rainbow | Hue Shift | 0.0 to 1.0, default 0.0 |
| Rainbow | Speed | -1.0 to 1.0, default 0.0 |
| Rainbow | Saturation | 0.0 to 1.0, default 0.9 |
| Output | Mix | 0.0 to 1.0, default 1.0 |
