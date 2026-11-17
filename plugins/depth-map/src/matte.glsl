// Pass 1 (stored in "matte"): how much each pixel differs from the learned
// background, as a soft 0..1 foreground matte. Blurred over a few pixels
// so noise doesn't speckle it, and smoothed over time: it rises quickly
// and falls slowly (Hold), so a subject doesn't flicker out.

float diffAt(vec2 p) {
    vec3 a = IMG_INPUT(p).rgb;
    vec3 b = texture(background, p).rgb;
    return length(a - b) + abs(luma(a) - luma(b));
}

void main() {
    vec2 e = 3.0 * sizeScale() / RENDERSIZE;
    float d = 0.0;
    for (int y = -1; y <= 1; y++) {
        for (int x = -1; x <= 1; x++) {
            float w = (x == 0 ? 2.0 : 1.0) * (y == 0 ? 2.0 : 1.0);
            d += w * diffAt(uv + vec2(x, y) * e);
        }
    }
    d /= 16.0;
    float raw = smoothstep(threshold, threshold + softness, d);
    float prev = texture(matte, uv).r;
    float m = raw > prev ? mix(prev, raw, 0.6) : mix(prev, raw, 1.0 - hold * 0.9);
    fragColor = vec4(m, 0.0, 0.0, 1.0);
}
