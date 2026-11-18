# Bar Mosaic

FFGL effect for Resolume: redraws the clip as rows of rounded, lit bars. Each
bar takes the clip's average colour under it. The bar edges sit on a grid in a
warped coordinate, so they line up into columns that bend with the image's
brightness (and an optional lens curve) and stretch into long streaks where the
brightness falls away.

The look is in [`src/effect.glsl`](src/effect.glsl); `src/lib.rs` declares the
parameters. Sizes are in pixels at 1080p and scale with the output.

The Key and Blocks controls work per bar, by the average brightness of the
clip under it, so a bar is never cut in half at a key edge. With their
defaults the output is the same as without them.

## Parameters

| Group | Parameter | Range |
|---|---|---|
| Bars | Bar Width | 2 to 80, default 12 (bar length in flat areas) |
| Bars | Row Height | 2 to 60, default 13 |
| Bars | Gap | 0 to 8, default 2 |
| Bars | Roundness | 0.0 to 1.0, default 0.5 |
| Bars | Orientation | Horizontal, Vertical |
| Warp | Luma Warp | -60 to 60, default 20 (in bars; how far brightness shifts the bar edges; negative flips which side streaks on) |
| Warp | Warp Blur | 0 to 80, default 14 (smoothness of the warp field) |
| Warp | Streak Length | 0 to 10, default 8 (doubling steps: where the warp folds back, one bar stretches up to 2^n px at 1080p) |
| Warp | Min Bar | 0.0 to 1.0, default 0.4 (shortest bar as a fraction of Bar Width; stops steep brightness squeezing bars into flickering slivers; 0 = off) |
| Warp | Lens | -1.0 to 1.0, default 0 (barrel / pincushion curve of the bar columns) |
| Warp | Row Wave | 0 to 40, default 0 |
| Warp | Wave Freq | 0 to 20, default 3 |
| Warp | Wave Speed | -5 to 5, default 0.3 |
| Warp | Scroll | -5 to 5, default 0 (bars per second) |
| Warp | Row Jitter | 0.0 to 1.0, default 0 |
| Blocks | Block Scale | 1 to 8, default 1 (off). Blocks are this many bars long and rows tall |
| Blocks | Block Low | 0.0 to 1.0, default 0.2 |
| Blocks | Block High | 0.0 to 1.0, default 0.55 (bars whose block averages inside Low..High become one block) |
| Key | Key Low | 0.0 to 1.0, default 0.0 |
| Key | Key High | 0.0 to 1.0, default 1.0 (bars whose average luma is inside Low..High get the effect) |
| Key | Key Softness | 0.0 to 0.5, default 0.05 |
| Key | Key Invert | Off / On |
| Key | Outside | Original, Dashes, Gap Colour (what replaces bars outside the key) |
| Look | Ramp | 0.0 to 1.0, default 0.5 |
| Look | Cap Glow | 0.0 to 2.0, default 0.8 |
| Look | Cap Floor | 0.0 to 1.0, default 0.3 (keeps caps visible over black) |
| Look | Gamma | 0.3 to 3.0, default 1.0 |
| Look | Tint | colour |
| Look | Gap Colour | colour |
| Output | Mix | 0.0 to 1.0, default 1.0 |

## Starting points

- Face with streaks (Video-30846): defaults, raise Luma Warp until the face edges start to streak (about 25–45).
- Lens arcs over a dark background (Video-33330): Lens -0.25, Luma Warp 1, Cap Floor 0.4.
- Mixed bars and blocks (Video-68826): Block Scale 3, Block Low 0.15, Block High 0.45, Key Low 0.08, Outside Dashes.
- Only the bright parts: Key Low 0.45, Outside Original.
- Fine tiles (Video-28874): Bar Width 6, Row Height 7, Gap 1.5, Luma Warp 4.
