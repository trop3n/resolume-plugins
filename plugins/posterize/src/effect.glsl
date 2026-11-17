// Posterize: fewer tone levels, either mapped onto a three-colour gradient
// or applied per colour channel. Ordered dither can soften the steps.

vec3 quantize(vec3 v, float n, float d) {
    return clamp(floor(v * (n - 1.0) + d) / (n - 1.0), 0.0, 1.0);
}

void main() {
    vec4 src = IMG_INPUT(uv);
    vec3 c = clamp((src.rgb - 0.5) * contrast + 0.5 + brightness, 0.0, 1.0);
    vec2 cell = floor(uv * RENDERSIZE / max(sizeScale(), 1.0));
    float d = mix(0.5, bayer4(cell), dither);
    float n = float(levels);
    vec3 col;
    if (mode == 0) {
        float q = quantize(vec3(luma(c)), n, d).x;
        col = q < 0.5 ? mix(shadowColour, midColour, q * 2.0) : mix(midColour, highlightColour, q * 2.0 - 1.0);
    } else {
        col = quantize(c, n, d);
    }
    fragColor = vec4(mix(src.rgb, col, mixAmount), src.a);
}
