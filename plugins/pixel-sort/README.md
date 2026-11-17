# Pixel Sort

FFGL effect for Resolume that sorts runs of pixels along each column or
row, on the GPU. A brightness mask picks which pixels take part; masked
pixels stay put and break each line into separate runs.

Three passes:

1. [`mask.glsl`](src/mask.glsl) marks sortable pixels.
2. [`sort.glsl`](src/sort.glsl) runs an odd-even transposition sort,
   `Iterations` steps in all, four steps per draw.
3. [`output.glsl`](src/output.glsl) mixes the result with the original.

A run of n pixels is fully sorted after n steps. Fewer steps leave the
streaky, half-sorted look, with streaks up to about `Iterations` pixels long.

## Parameters

| Group | Parameter | What it does |
|---|---|---|
| Sort | Direction | Down/Up sort columns, Right/Left sort rows; the brightest (or highest key) pixels move that way |
| Sort | Sort By | Brightness, Hue or Saturation |
| Sort | Iterations | Sort steps per frame (0 to 1024); sets the streak length and the GPU cost |
| Mask | Low, High | Pixels with brightness in this range get sorted |
| Mask | Invert Mask | Sort the pixels outside the range instead |
| Mask | Break Up | Random breaks that vary the streak length |
| Mask | Show Mask | Tints sorted pixels red, for setting Low and High |
| Output | Mix | Blend with the original |

## Cost

Linear in Iterations. On an RTX 3060 Ti at 1920x1080, about 0.07 ms per
step: roughly 9 ms at the default 128 steps and 18 ms at 256. For 60 fps at
1080p, keep Iterations below about 200 (less when other effects run).
