# Data Prism

FFGL effect for Resolume: fUI-style HUD overlay (brackets, lock frame, measure arcs, labels, circle chain, anchors) around a box you place on screen.

Ported from the Prism ISF shader `DataPrism.fs` in
[prism-ffgl](https://github.com/trop3n/prism-ffgl), which recreates effects
from the Prism Lab Blender add-on. The shader is in
[`src/effect.glsl`](src/effect.glsl), nearly unchanged; parameters keep
their ISF names as uniforms.

## Parameters

| Group | Parameter | Range |
|---|---|---|
| Box | Center | position (X and Y) |
| Box | Width | 0.02 to 1.0, default 0.3 |
| Box | Height | 0.02 to 1.0, default 0.45 |
| Box | Color | colour |
| Box | Accent | colour |
| Box | Line Weight | 0.5 to 8.0, default 2.0 |
| Box | Text Size | 0.0 to 3.0, default 1.0 |
| Box | Speed | -4.0 to 4.0, default 1.0 |
| Box | Seed | 0.0 to 100.0, default 0.0 |
| Box | Opacity | 0.0 to 1.0, default 1.0 |
| Brackets | Brackets | toggle, default on |
| Brackets | Brackets Opacity | 0.0 to 1.0, default 1.0 |
| Brackets | Bracket Length | 0.0 to 0.5, default 0.18 |
| Brackets | Bracket Gap | 0.0 to 0.5, default 0.08 |
| Labels | Labels | toggle, default on |
| Labels | Labels Opacity | 0.0 to 1.0, default 1.0 |
| Labels | Label Count | 0.0 to 8.0, default 4.0 |
| Labels | Label Offset | 0.0 to 1.0, default 0.35 |
| Anchors | Anchors | toggle, default off |
| Anchors | Anchors Opacity | 0.0 to 1.0, default 1.0 |
| Anchors | Anchor Count | 0.0 to 32.0, default 10.0 |
| Anchors | Anchor Size | 0.0 to 4.0, default 1.0 |
| Arcs | Arcs | toggle, default off |
| Arcs | Arcs Opacity | 0.0 to 1.0, default 1.0 |
| Arcs | Arc Radius | 0.0 to 3.0, default 1.4 |
| Arcs | Arc Sweep | 0.0 to 360.0, default 240.0 |
| Arcs | Arc Ticks | 0.0 to 200.0, default 48.0 |
| Lock | Lock | toggle, default off |
| Lock | Lock Opacity | 0.0 to 1.0, default 1.0 |
| Lock | Lock Size | 0.0 to 2.0, default 0.75 |
| Chain | Chain | toggle, default off |
| Chain | Chain Opacity | 0.0 to 1.0, default 1.0 |
| Chain | Chain Count | 3.0 to 120.0, default 28.0 |
| Chain | Chain Radius | 0.0 to 3.0, default 1.25 |
