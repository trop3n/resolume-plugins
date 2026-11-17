# Fluid Prism

FFGL effect for Resolume: smoke-like fluid distortion that swirls the video out of an emitter, with chromatic dispersion (in the style of the Hiss effect from Control).

Ported from the Prism ISF shader `FluidPrism.fs` in
[prism-ffgl](https://github.com/trop3n/prism-ffgl), which recreates effects
from the Prism Lab Blender add-on. The shader is in
[`src/effect.glsl`](src/effect.glsl), nearly unchanged; parameters keep
their ISF names as uniforms.

## Parameters

| Group | Parameter | Range |
|---|---|---|
| Fluid | Center | position (X and Y) |
| Fluid | Radius | 0.0 to 0.5, default 0.08 |
| Fluid | Emit | 0.0 to 1.0, default 0.6 |
| Fluid | Direction | 0.0 to 360.0, default 90.0 |
| Fluid | Buoyancy | 0.0 to 1.0, default 0.25 |
| Fluid | Turbulence | 0.0 to 2.0, default 0.6 |
| Fluid | Scale | 0.5 to 20.0, default 4.0 |
| Fluid | Speed | 0.0 to 4.0, default 1.0 |
| Fluid | Strength | 0.0 to 2.0, default 1.0 |
| Fluid | Fade | 0.0 to 4.0, default 0.6 |
| Fluid | Recover | 0.0 to 4.0, default 0.3 |
| Fluid | Dispersion | 0.0 to 1.0, default 0.35 |
| Fluid | Haze | 0.0 to 1.0, default 0.15 |
| Fluid | Mix | 0.0 to 1.0, default 1.0 |
