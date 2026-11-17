// Retro LCD: a low-res, two-tone phone screen with ordered dither and the
// faint grid between LCD pixels.

void main() {
    vec4 src = IMG_INPUT(uv);
    float size = max(pixelSize * sizeScale(), 2.0);
    vec2 px = uv * RENDERSIZE;
    vec2 cell = floor(px / size);
    vec3 c = IMG_INPUT((cell + 0.5) * size / RENDERSIZE).rgb;
    float l = clamp((luma(c) - 0.5) * contrast + 0.5 + brightness, 0.0, 1.0);
    float n = float(levels) - 1.0;
    float t = dither ? bayer4(cell) : 0.5;
    float q = clamp(floor(l * n + t) / n, 0.0, 1.0);
    vec3 col = mix(inkColour, screenColour, q);

    // Gaps between pixels show the bare screen.
    vec2 f = fract(px / size);
    float gap = 0.14;
    float inside = smoothstep(0.0, gap, f.x) * smoothstep(0.0, gap, f.y);
    col = mix(col, screenColour * 1.06, (1.0 - inside) * grid);
    fragColor = vec4(mix(src.rgb, clamp(col, 0.0, 1.0), mixAmount), src.a);
}
