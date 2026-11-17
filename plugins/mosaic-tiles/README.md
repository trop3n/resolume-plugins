# Mosaic Tiles

FFGL effect for Resolume: grid of tiles flipping between the image and a solid colour. Effect 19 of the
Video-22575 list.

The look is in [`src/effect.glsl`](src/effect.glsl); `src/lib.rs` declares
the parameters. Sizes are in pixels at 1080p and scale with the output.

## Parameters

| Group | Parameter | Range |
|---|---|---|
| Tiles | Columns | 2 to 40, default 8 |
| Tiles | Solid Amount | 0.0 to 1.0, default 0.4 |
| Tiles | Flip Speed | 0.0 to 10.0, default 1.0 |
| Tiles | Seed | 0.0 to 100.0, default 0.0 |
| Image | Image | Original, Threshold, Mono (default Threshold) |
| Image | Level | 0.0 to 1.0, default 0.5 |
| Colours | Tile | colour |
| Colours | Grout | 0.0 to 0.3, default 0.04 |
| Colours | Grout Colour | colour |
| Output | Mix | 0.0 to 1.0, default 1.0 |
