# CCTV

FFGL effect for Resolume: cheap security camera: soft, tinted, noisy, with jitter, scan lines and a REC dot. Effect 8 of the
Video-22575 list.

The look is in [`src/effect.glsl`](src/effect.glsl); `src/lib.rs` declares
the parameters. Sizes are in pixels at 1080p and scale with the output.

## Parameters

| Group | Parameter | Range |
|---|---|---|
| Image | Softness | 0.0 to 3.0, default 1.0 |
| Image | Chroma Shift | 0.0 to 10.0, default 2.0 |
| Grade | Saturation | 0.0 to 1.5, default 0.6 |
| Grade | Contrast | 0.5 to 2.0, default 1.15 |
| Grade | Cast | colour |
| Signal | Noise | 0.0 to 1.0, default 0.25 |
| Signal | Scan Lines | 0.0 to 1.0, default 0.35 |
| Signal | Jitter | 0.0 to 1.0, default 0.2 |
| Signal | Vignette | 0.0 to 1.0, default 0.5 |
| Signal | REC Dot | toggle, default on |
| Output | Mix | 0.0 to 1.0, default 1.0 |
