// Thermal: maps "heat" (brightness, or warmth of colour) through a
// false-colour palette, with the blur and noise of a thermal camera.

const vec3 IRON[6] = vec3[6](
    vec3(0.00, 0.00, 0.06),
    vec3(0.22, 0.00, 0.55),
    vec3(0.75, 0.05, 0.45),
    vec3(0.95, 0.32, 0.05),
    vec3(1.00, 0.72, 0.00),
    vec3(1.00, 1.00, 0.85)
);

vec3 iron(float t) {
    float x = clamp(t, 0.0, 1.0) * 5.0;
    int i = int(min(floor(x), 4.0));
    return mix(IRON[i], IRON[i + 1], x - float(i));
}

vec3 paletteColour(float t) {
    if (palette == 1) return hsv2rgb(vec3(0.7 * (1.0 - t), 1.0, 1.0));
    if (palette == 2) return vec3(t);
    if (palette == 3) return vec3(1.0 - t);
    return iron(t);
}

float heat(vec3 c) {
    float l = luma(c);
    if (source == 1) return clamp(l * 0.6 + (c.r - c.b) * 0.6 + 0.15, 0.0, 1.0);
    return l;
}

void main() {
    vec4 src = IMG_INPUT(uv);
    // 3x3 tent blur; radius up to 8 px at 1080p.
    float r = blur * 8.0 * sizeScale();
    float h = 0.0;
    for (int y = -1; y <= 1; y++) {
        for (int x = -1; x <= 1; x++) {
            float w = (x == 0 ? 2.0 : 1.0) * (y == 0 ? 2.0 : 1.0);
            h += w * heat(IMG_INPUT(uv + vec2(x, y) * r / RENDERSIZE).rgb);
        }
    }
    h /= 16.0;
    h = (h - low) / max(high - low, 1e-3);
    h = (h - 0.5) * contrast + 0.5;
    h += (hash12(uv * RENDERSIZE + floor(TIME * 24.0) * 7.0) - 0.5) * noise * 0.15;
    vec3 col = paletteColour(clamp(h, 0.0, 1.0));
    fragColor = vec4(mix(src.rgb, col, mixAmount), src.a);
}
