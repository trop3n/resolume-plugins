# Tracking

FFGL effect for Resolume that draws a surveillance-style tracking HUD.
Effect 7 of the Video-22575 list.

There's no face or body detection here; a shader can't recognise people.
The box follows **motion**: whatever moves gets boxed, and a subject that
stops moving stays boxed for a while (Memory) before the box fades out.
On a fixed camera with one person moving, that reads as tracking them.

1. [`energy.glsl`](src/energy.glsl) measures motion against the previous
   frame and holds it as a decaying "energy".
2. [`remember.glsl`](src/remember.glsl) keeps the frame for next time.
3. [`tracker.glsl`](src/tracker.glsl) finds the energy's weighted centre and
   spread on a 48x48 grid and smooths the box over time. It runs in just two
   texels of a persistent buffer.
4. [`output.glsl`](src/output.glsl) desaturates the picture and draws the
   HUD: box or corner brackets, a head box guessed from the top of the body
   box, crosshairs, a "TRK 01" tag and a live X/Y readout from a built-in
   bitmap font.

Several people moving at once get one box around all of them.

## Parameters

| Group | Parameter | What it does |
|---|---|---|
| Tracker | Sensitivity | How small a movement counts |
| Tracker | Memory | How long motion is remembered (higher keeps a paused subject boxed) |
| Tracker | Smoothing | How steadily the box follows |
| Tracker | Padding | Extra margin around the box |
| HUD | Box Style | Full box or corner brackets |
| HUD | Head Box | Small box at the top of the body box |
| HUD | Crosshair | Lines through the centre and a ring |
| HUD | Labels | "TRK 01" tag and X/Y readout |
| HUD | Line Width | In pixels at 1080p |
| HUD | Colour | HUD colour |
| Picture | Desaturate | Greys the picture under the HUD |
| Picture | Show Motion | Tints the motion energy red, for tuning |
| Output | Mix | Blend with the original |
