# Blur Prism

FFGL effect for Resolume: directional motion-style blur inside a box/ellipse area or a luma/alpha/color key.

Ported from the Prism ISF shader `BlurPrism.fs` in
[prism-ffgl](https://github.com/trop3n/prism-ffgl), which recreates effects
from the Prism Lab Blender add-on. The shader is in
[`src/effect.glsl`](src/effect.glsl), nearly unchanged; parameters keep
their ISF names as uniforms.

## Parameters

| Group | Parameter | Range |
|---|---|---|
| Blur | Mode | Area, Key |
| Blur | Shape | Box, Ellipse |
| Blur | Center | position (X and Y) |
| Blur | Width | 0.0 to 1.5, default 0.4 |
| Blur | Height | 0.0 to 1.5, default 0.3 |
| Blur | Rotation | 0.0 to 360.0, default 0.0 |
| Blur | Key | Luma, Alpha, Color |
| Blur | Key Color | colour |
| Blur | Threshold | 0.0 to 1.0, default 0.5 |
| Blur | Invert | toggle, default off |
| Blur | Softness | 0.0 to 0.5, default 0.03 |
| Blur | Amount | 0.0 to 1.0, default 1.0 |
| Blur | Length | 0.0 to 6.0, default 1.0 |
| Blur | Direction | 0.0 to 360.0, default 0.0 |
| Blur | Quality | 2.0 to 64.0, default 16.0 |
| Blur | Trail | toggle, default on |
| Blur | Animate | toggle, default off |
| Blur | Pulse Speed | 0.0 to 10.0, default 1.0 |
| Blur | Pulse Depth | 0.0 to 1.0, default 0.6 |
| Blur | Spin | -5.0 to 5.0, default 0.0 |
| Blur | Mix | 0.0 to 1.0, default 1.0 |
