# Risograph

FFGL effect for Resolume that turns the clip into a two-ink risograph
print. The image is separated into two plates, each plate is screened
(stochastic grain or halftone dots), the second plate prints out of
register, and the inks overprint on paper.

All the work is in [`src/risograph.glsl`](src/risograph.glsl); `src/lib.rs`
declares the parameters.

## Parameters

| Group | Parameter | What it does |
|---|---|---|
| Inks | Ink 1, Ink 2 | Ink colours (approximations of RISO swatches). Ink 1 is the base plate. |
| Inks | Paper | Natural, White, Newsprint, Kraft or Grey |
| Tone | Separation | **Duotone**: the same tone on both plates. **Best Fit**: per pixel, the mix of the two inks that best matches the colour. **Shadows + Mids**: shadows on plate 1, mids and shadows on plate 2. |
| Tone | Contrast, Brightness | Applied before separating |
| Tone | Clean Highlights | Tones lighter than this print as bare paper |
| Tone | Density | Ink amount on both plates |
| Tone | Ink 2 Strength | Extra control over the second plate |
| Screen | Screen | Grain (stochastic) or Halftone (Ink 1 at 45°, Ink 2 at 15°) |
| Screen | Dot Size | Grain or dot size, in pixels at 1080p |
| Screen | Ink Grain | Uneven ink: blotches and pinholes |
| Screen | Paper Texture | Paper fibres and tooth |
| Registration | Offset X, Offset Y | Plate 2's offset, in pixels at 1080p |
| Registration | Boil | Re-rolls the grain and jitters registration over time |

Sizes are relative to 1080p, so the look holds at any resolution.
