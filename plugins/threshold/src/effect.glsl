// Threshold: a hard 1-bit cut between two colours. Noise jitters the cut
// level per grain cell, which roughens edges the way a photocopy does.

void main() {
    vec4 src = IMG_INPUT(uv);
    float l = luma(src.rgb);
    vec2 px = uv * RENDERSIZE;
    float seed = boil > 0.0 ? floor(TIME * mix(2.0, 24.0, boil)) : 0.0;
    vec2 cell = floor(px / max(grainSize * sizeScale(), 1.0));
    float t = level + (hash12(cell + seed * 37.0) - 0.5) * noise;
    float v = softness > 0.0 ? smoothstep(t - softness, t + softness, l) : step(t, l);
    if (invert) v = 1.0 - v;
    vec3 col = mix(darkColour, lightColour, v);
    fragColor = vec4(mix(src.rgb, col, mixAmount), src.a);
}
