# Ripple Prism

FFGL effect for Resolume: ripple distortion spreading from a movable center (circular, linear waves, droplet, standing wave).

Ported from the Prism ISF shader `RipplePrism.fs` in
[prism-ffgl](https://github.com/trop3n/prism-ffgl), which recreates effects
from the Prism Lab Blender add-on. The shader is in
[`src/effect.glsl`](src/effect.glsl), nearly unchanged; parameters keep
their ISF names as uniforms.

## Parameters

| Group | Parameter | Range |
|---|---|---|
| Ripple | Pattern | Circular, Linear Waves, Droplet, Standing Wave |
| Ripple | Amplitude | 0.0 to 0.2, default 0.02 |
| Ripple | Wavelength | 0.01 to 1.0, default 0.25 |
| Ripple | Speed | -2.0 to 2.0, default 0.3 |
| Ripple | Phase | 0.0 to 360.0, default 0.0 |
| Ripple | Falloff | 0.0 to 10.0, default 0.5 |
| Ripple | Radius | 0.05 to 3.0, default 1.0 |
| Ripple | Direction | 0.0 to 360.0, default 0.0 |
| Ripple | Noise | 0.0 to 2.0, default 0.0 |
| Ripple | Center | position (X and Y) |
| Ripple | Shading | 0.0 to 1.0, default 0.3 |
| Ripple | Mix | 0.0 to 1.0, default 1.0 |
