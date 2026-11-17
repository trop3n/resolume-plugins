# Thermal

FFGL effect for Resolume: thermal-camera false colour: iron, rainbow, white hot or black hot. Effect 4 of the
Video-22575 list.

The look is in [`src/effect.glsl`](src/effect.glsl); `src/lib.rs` declares
the parameters. Sizes are in pixels at 1080p and scale with the output.

## Parameters

| Group | Parameter | Range |
|---|---|---|
| Palette | Palette | Iron, Rainbow, White Hot, Black Hot (default Iron) |
| Palette | Source | Brightness, Warmth (default Brightness) |
| Range | Low | 0.0 to 1.0, default 0.05 |
| Range | High | 0.0 to 1.0, default 0.95 |
| Range | Contrast | 0.5 to 3.0, default 1.2 |
| Camera | Blur | 0.0 to 1.0, default 0.3 |
| Camera | Noise | 0.0 to 1.0, default 0.3 |
| Output | Mix | 0.0 to 1.0, default 1.0 |
