// Edge Detect: Sobel edges drawn three ways.
// Lines: dark outlines on paper, like a technical drawing.
// Scan Lines: horizontal stripes that thicken in dark areas and at edges.
// Neon: glowing coloured edges on black.

float lumaAt(vec2 p) {
    return luma(IMG_INPUT(p).rgb);
}

void main() {
    vec4 src = IMG_INPUT(uv);
    float s = sizeScale();
    vec2 e = lineWidth * s / RENDERSIZE;

    float tl = lumaAt(uv + e * vec2(-1, 1));
    float tc = lumaAt(uv + e * vec2(0, 1));
    float tr = lumaAt(uv + e * vec2(1, 1));
    float ml = lumaAt(uv + e * vec2(-1, 0));
    float mr = lumaAt(uv + e * vec2(1, 0));
    float bl = lumaAt(uv + e * vec2(-1, -1));
    float bc = lumaAt(uv + e * vec2(0, -1));
    float br = lumaAt(uv + e * vec2(1, -1));
    float gx = (tr + 2.0 * mr + br) - (tl + 2.0 * ml + bl);
    float gy = (tl + 2.0 * tc + tr) - (bl + 2.0 * bc + br);
    float edge = clamp((length(vec2(gx, gy)) - threshold) * strength, 0.0, 1.0);

    vec3 col;
    if (mode == 1) {
        float dark = 1.0 - luma(src.rgb);
        float d = clamp(dark * dark * 1.2 + edge, 0.0, 1.0);
        float spacing = max(lineSpacing * s, 2.0);
        float pattern = abs(fract(uv.y * RENDERSIZE.y / spacing) - 0.5) * 2.0;   // 0 at stripe centre
        float aa = 2.0 / spacing;
        float on = 1.0 - smoothstep(d - aa, d + aa, pattern);
        col = mix(paperColour, inkColour, on);
    } else if (mode == 2) {
        col = inkColour * edge * 1.5 + inkColour * edge * edge;
    } else {
        col = mix(paperColour, inkColour, edge);
    }
    fragColor = vec4(mix(src.rgb, clamp(col, 0.0, 1.0), mixAmount), src.a);
}
