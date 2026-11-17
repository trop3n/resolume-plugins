// Dither: ordered (Bayer) or noise dithering down to a few levels, at a
// chosen pixel size, in two colours or per colour channel.

void main() {
    vec4 src = IMG_INPUT(uv);
    float size = max(pixelSize * sizeScale(), 1.0);
    vec2 cell = floor(uv * RENDERSIZE / size);
    vec2 cellUv = (cell + 0.5) * size / RENDERSIZE;
    vec3 c = clamp((IMG_INPUT(cellUv).rgb - 0.5) * contrast + 0.5 + brightness, 0.0, 1.0);
    float t = pattern == 0 ? bayer2(cell)
            : pattern == 1 ? bayer4(cell)
            : pattern == 2 ? bayer8(cell)
            : hash12(cell);
    float n = float(levels) - 1.0;
    vec3 col;
    if (colourMode == 0) {
        float q = clamp(floor(luma(c) * n + t) / n, 0.0, 1.0);
        col = mix(darkColour, lightColour, q);
    } else {
        col = clamp(floor(c * n + t) / n, 0.0, 1.0);
    }
    fragColor = vec4(mix(src.rgb, col, mixAmount), src.a);
}
