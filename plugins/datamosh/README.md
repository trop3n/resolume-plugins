# Datamosh

FFGL effect for Resolume that fakes a video codec losing its keyframes.
Effect 16 of the Video-22575 list.

A real datamosh deletes a video's keyframes, so the decoder keeps applying
new motion vectors and corrections to an old picture. This does the same
live on the GPU:

1. [`motion.glsl`](src/motion.glsl) estimates one motion vector per block
   between the previous input frame and this one (Lucas-Kanade on a 4x4
   sample grid), like a codec's macroblocks.
2. [`mosh.glsl`](src/mosh.glsl) moves last frame's *output* by that motion
   and adds the residual (this frame minus the moved previous input). Old
   content rides along with new motion.
3. [`remember.glsl`](src/remember.glsl) keeps this frame's input for next time.
4. [`output.glsl`](src/output.glsl) shows the result.

The mosh, motion and previous input live in persistent buffers (see
`Pass::store` in `crates/ffgl-fx`).

It needs movement. A still clip just shows through, so drop it on moving
footage, then let the mosh build up and hit Keyframe to clear it.

## Parameters

| Group | Parameter | What it does |
|---|---|---|
| Keyframes | Keyframe | While on, the clip shows straight through (map it to a button to "reset") |
| Keyframes | Key Interval | Automatic keyframe every N seconds (0 = never) |
| Keyframes | Refresh | Rate of random blocks refreshing from the clip, like codec I-blocks |
| Motion | Block Size | Macroblock size in pixels at 1080p |
| Motion | Motion Gain | Multiplies the motion; above 1 smears faster |
| Motion | Residual | How much of each frame's correction is applied. 0 is pure smear of the old picture; 1 lets new detail through on top of it |
| Motion | Cut Threshold | Changes bigger than this carry no residual, as if their keyframe was deleted, so the old picture survives cuts and fast movement. Lower is more moshed |
| Motion | Freeze Motion | Keep reusing the current vectors: the "bloom" look of a repeated P-frame |
| Output | Show Motion | Shows the block vectors as colour (red = right, green = up) |
| Output | Mix | Blend with the original |
