// Pass 3: soften the depth and show it in the chosen style.

// Turbo colour map, polynomial approximation (Google, Apache-2.0).
vec3 turbo(float x) {
    const vec4 kRed = vec4(0.13572138, 4.61539260, -42.66032258, 132.13108234);
    const vec4 kGreen = vec4(0.09140261, 2.19418839, 4.84296658, -14.18503333);
    const vec4 kBlue = vec4(0.10667330, 12.64194608, -60.58204836, 110.36276771);
    const vec2 kRed2 = vec2(-152.94239396, 59.28637943);
    const vec2 kGreen2 = vec2(4.27729857, 2.82956604);
    const vec2 kBlue2 = vec2(-89.90310912, 27.34824973);
    x = clamp(x, 0.0, 1.0);
    vec4 v4 = vec4(1.0, x, x * x, x * x * x);
    vec2 v2 = v4.zw * v4.z;
    return clamp(vec3(dot(v4, kRed) + dot(v2, kRed2),
                      dot(v4, kGreen) + dot(v2, kGreen2),
                      dot(v4, kBlue) + dot(v2, kBlue2)), 0.0, 1.0);
}

void main() {
    vec4 src = IMG_INPUT(uv);
    // 3x3 tent blur, radius up to 12 px at 1080p.
    vec2 e = smoothing * 12.0 * sizeScale() / RENDERSIZE;
    float d = 0.0;
    for (int y = -1; y <= 1; y++) {
        for (int x = -1; x <= 1; x++) {
            float w = (x == 0 ? 2.0 : 1.0) * (y == 0 ? 2.0 : 1.0);
            d += w * IMG_PASS(uv + vec2(x, y) * e).r;
        }
    }
    d /= 16.0;

    vec3 col;
    if (style == 1) {
        col = vec3(1.0 - d);
    } else if (style == 2) {
        col = turbo(d);
    } else if (style == 3) {
        // Contours: grey depth with a line at every band boundary.
        float f = fract(d * float(bands));
        float lineW = fwidth(d * float(bands)) * 1.5;
        float line = 1.0 - smoothstep(0.0, lineW, min(f, 1.0 - f));
        col = mix(vec3(d * 0.6), vec3(1.0), line);
    } else if (style == 4) {
        col = vec3(texture(matte, uv).r);
    } else {
        col = vec3(d);
    }
    fragColor = vec4(mix(src.rgb, col, mixAmount), src.a);
}
