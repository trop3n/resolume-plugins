// Pass 0 (stored in "motion"): one motion vector per block, in pixels, from
// the previous input frame to this one. Lucas-Kanade over a 4x4 grid of
// samples in the block, so every pixel of a block gets the same vector,
// like a codec's macroblocks. Freeze Motion keeps last frame's vectors.

float lumIn(vec2 p) { return luma(IMG_INPUT(p).rgb); }
float lumPrev(vec2 p) { return luma(texture(prevInput, p).rgb); }

void main() {
    if (freezeMotion) {
        fragColor = texture(motion, uv);
        return;
    }
    float bs = max(blockSize * sizeScale(), 4.0);
    vec2 inv = 1.0 / RENDERSIZE;
    vec2 centre = (floor(uv * RENDERSIZE / bs) + 0.5) * bs;
    // A wide gradient step catches moves bigger than a pixel or two.
    float d = max(bs / 8.0, 1.0);

    mat2 A = mat2(0.0);
    vec2 b = vec2(0.0);
    for (int y = 0; y < 4; y++) {
        for (int x = 0; x < 4; x++) {
            vec2 p = (centre + (vec2(x, y) - 1.5) * bs / 4.0) * inv;
            float gx = (lumIn(p + vec2(d, 0.0) * inv) - lumIn(p - vec2(d, 0.0) * inv)) / (2.0 * d);
            float gy = (lumIn(p + vec2(0.0, d) * inv) - lumIn(p - vec2(0.0, d) * inv)) / (2.0 * d);
            float it = lumIn(p) - lumPrev(p);
            A += mat2(gx * gx, gx * gy, gx * gy, gy * gy);
            b += vec2(gx, gy) * it;
        }
    }
    // Flat blocks have no usable gradient: call them still.
    float det = A[0][0] * A[1][1] - A[0][1] * A[1][0];
    vec2 v = det > 1e-7 ? -(inverse(A) * b) : vec2(0.0);
    if (length(v) > bs) v = normalize(v) * bs;
    fragColor = vec4(v, 0.0, 1.0);
}
