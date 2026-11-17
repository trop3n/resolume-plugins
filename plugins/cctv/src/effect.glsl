// CCTV: a cheap security camera. Soft low-res image with a colour cast and
// washed-out saturation, chroma bleed, line jitter, noise, scan lines, a
// vignette and a blinking REC dot.

void main() {
    vec4 src = IMG_INPUT(uv);
    float s = sizeScale();
    float frame = floor(TIME * 30.0);

    // Tracking jitter: rows of a few pixels shift sideways a little each frame.
    float row = floor(uv.y * RENDERSIZE.y / max(2.0 * s, 1.0));
    vec2 p = uv + vec2((hash12(vec2(row, frame)) - 0.5) * jitter * 6.0 * s / RENDERSIZE.x, 0.0);

    // Soft, low-res look: a 3x3 box blur, plus a chroma offset.
    vec2 r = softness * 3.0 * s / RENDERSIZE;
    vec2 shift = vec2(chromaShift * s / RENDERSIZE.x, 0.0);
    vec3 c = vec3(0.0);
    for (int y = -1; y <= 1; y++) {
        for (int x = -1; x <= 1; x++) {
            vec2 q = p + vec2(x, y) * r;
            c += vec3(IMG_INPUT(q + shift).r, IMG_INPUT(q).g, IMG_INPUT(q - shift).b);
        }
    }
    c /= 9.0;

    // Grade: wash out, tint, crush.
    c = mix(vec3(luma(c)), c, saturation) * castColour;
    c = clamp((c - 0.5) * contrast + 0.5, 0.0, 1.0);
    c += (hash12(uv * RENDERSIZE / max(s, 1.0) + frame * 7.3) - 0.5) * noise * 0.25;
    c *= 1.0 - scanLines * 0.3 * step(0.5, fract(uv.y * RENDERSIZE.y / max(3.0 * s, 1.0)));
    vec2 v = (uv - 0.5) * vec2(RENDERSIZE.x / RENDERSIZE.y, 1.0);
    c *= mix(1.0, smoothstep(0.85, 0.25, length(v)), vignette);

    // REC dot in the top-right corner, blinking once a second.
    if (recDot && fract(TIME) < 0.5) {
        vec2 dp = (uv * RENDERSIZE - vec2(RENDERSIZE.x - 60.0 * s, RENDERSIZE.y - 60.0 * s)) / s;
        float dotMask = 1.0 - smoothstep(13.0, 15.0, length(dp));
        c = mix(c, vec3(0.95, 0.08, 0.06), dotMask);
    }
    fragColor = vec4(mix(src.rgb, clamp(c, 0.0, 1.0), mixAmount), src.a);
}
