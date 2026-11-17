# Depth Map

FFGL effect for Resolume that fakes a depth camera. Effect 1 of the
Video-22575 list.

One camera image has no depth in it; real depth needs a depth camera or a
machine-learning model. This effect makes a convincing stand-in for a
**fixed camera**, such as a locked-off stage shot:

1. [`background.glsl`](src/background.glsl) learns the empty scene.
2. [`matte.glsl`](src/matte.glsl) marks whatever differs from it (people,
   objects that arrive) as a soft foreground matte, steadied over time.
3. [`depth.glsl`](src/depth.glsl) gives the scene a simple fake depth and
   pushes the foreground near.
4. [`output.glsl`](src/output.glsl) smooths it and draws it.

## Setting it up

1. Point the camera at the empty stage and turn **Learn Background** on for
   a moment, then off. (With nothing learned, the first frame becomes the
   background, so anyone in it at that moment won't stand out.)
2. Use Style **Matte** while adjusting **Threshold** and **Softness**, until
   people are white and the empty stage is black.
3. Switch Style back to Grayscale, Turbo or Contours.

If the camera moves or the shot cuts, most of the frame suddenly reads as
foreground; **Auto Relearn** notices that and re-learns the background
within a few frames. Anyone in shot at that moment gets learned too, until
they move. For a clean result, learn the background again by hand.

## Parameters

| Group | Parameter | What it does |
|---|---|---|
| Background | Learn Background | While on, copies the clip in as the empty scene |
| Background | Adapt | How fast the background follows slow lighting changes (only where nobody is) |
| Background | Auto Relearn | Re-learn quickly when most of the frame reads as foreground (cut or camera move) |
| Subject | Threshold, Softness | How different from the background a pixel must be to count as foreground |
| Subject | Hold | How slowly the foreground fades when someone stops differing |
| Depth | Scene | Fake depth for the background: Floor (lower in frame is nearer), Brightness, or Flat |
| Depth | Scene Amount | How much of the near/far range the scene uses |
| Depth | Near, Far | Depth values for the foreground and the farthest scene |
| Depth | Relief | Shading detail on the foreground |
| Output | Smoothing | Blurs the depth |
| Output | Style | Grayscale (white = near), Inverted, Turbo colour map, Contours, or the raw Matte |
| Output | Bands | Contour count |
| Output | Mix | Blend with the original |
