// Night Vision: an amplified, noisy green phosphor image with bloom, scan
// lines, flicker and a tube vignette.

void main() {
    vec4 src = IMG_INPUT(uv);
    float s = sizeScale();
    float l = luma(src.rgb);

    // Bloom: average a ring of samples and keep what's bright.
    float b = 0.0;
    for (int i = 0; i < 12; i++) {
        float a = float(i) * 0.5235988;
        b += luma(IMG_INPUT(uv + vec2(cos(a), sin(a)) * 10.0 * s / RENDERSIZE).rgb);
    }
    b /= 12.0;

    float frame = floor(TIME * 30.0);
    float v = pow(clamp(l * gain, 0.0, 4.0), 0.85);
    v += max(b * gain - 0.7, 0.0) * bloom * 1.5;
    v += (hash12(uv * RENDERSIZE / max(s, 1.0) + frame * 13.1) - 0.5) * noise;
    v *= 1.0 + (hash12(vec2(frame, 3.0)) - 0.5) * flicker * 0.3;
    v *= 1.0 - scanLines * 0.3 * step(0.5, fract(uv.y * RENDERSIZE.y / max(3.0 * s, 1.0)));
    vec2 q = (uv - 0.5) * vec2(RENDERSIZE.x / RENDERSIZE.y, 1.0);
    v *= mix(1.0, smoothstep(0.75, 0.2, length(q)), vignette);
    v = max(v, 0.0);

    // Overdriven highlights wash out toward white.
    vec3 col = phosphor * v;
    col = mix(col, vec3(0.85, 1.0, 0.85), clamp(v - 1.0, 0.0, 1.0) * 0.6);
    fragColor = vec4(mix(src.rgb, clamp(col, 0.0, 1.0), mixAmount), src.a);
}
