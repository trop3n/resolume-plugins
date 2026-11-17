# Glow Prism

FFGL effect for Resolume: bright parts of the video bloom into a soft glow.

Ported from the Prism ISF shader `GlowPrism.fs` in
[prism-ffgl](https://github.com/trop3n/prism-ffgl), which recreates effects
from the Prism Lab Blender add-on. The shader is in
[`src/effect.glsl`](src/effect.glsl), nearly unchanged; parameters keep
their ISF names as uniforms.

## Parameters

| Group | Parameter | Range |
|---|---|---|
| Glow | Threshold | 0.0 to 1.0, default 0.7 |
| Glow | Softness | 0.0 to 0.5, default 0.1 |
| Glow | Size | 0.0 to 1.0, default 0.6 |
| Glow | Strength | 0.0 to 5.0, default 1.0 |
| Glow | Tint | colour |
| Glow | Mix | 0.0 to 1.0, default 1.0 |
