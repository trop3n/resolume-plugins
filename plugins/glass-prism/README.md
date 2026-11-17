# Glass Prism

FFGL effect for Resolume: view the video through patterned glass (reeded, fluted, frosted, liquid, bubble, ripple, hammered, lenticular).

Ported from the Prism ISF shader `GlassPrism.fs` in
[prism-ffgl](https://github.com/trop3n/prism-ffgl), which recreates effects
from the Prism Lab Blender add-on. The shader is in
[`src/effect.glsl`](src/effect.glsl), nearly unchanged; parameters keep
their ISF names as uniforms.

## Parameters

| Group | Parameter | Range |
|---|---|---|
| Glass | Pattern | Reeded, Fluted, Frost, Liquid, Bubble, Ripple, Hammered, Lenticular |
| Glass | Scale | 2.0 to 120.0, default 24.0 |
| Glass | Depth | 0.0 to 1.0, default 0.35 |
| Glass | Rotation | 0.0 to 360.0, default 0.0 |
| Glass | Dispersion | 0.0 to 1.0, default 0.0 |
| Glass | Frost | 0.0 to 1.0, default 0.0 |
| Glass | Sheen | 0.0 to 1.0, default 0.15 |
| Glass | Tint | colour |
| Glass | Animate | Off, Ripples, Waves |
| Glass | Anim Amount | 0.0 to 2.0, default 0.5 |
| Glass | Anim Speed | -4.0 to 4.0, default 1.0 |
| Glass | Anim Freq | 0.5 to 30.0, default 4.0 |
| Glass | Anim Dir | 0.0 to 360.0, default 0.0 |
| Glass | Center | position (X and Y) |
| Glass | Mix | 0.0 to 1.0, default 1.0 |
