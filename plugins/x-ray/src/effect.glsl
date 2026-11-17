// X-Ray: inverted, high-contrast luminance in a cool tint, with glowing
// edges, film grain and faint scan lines.

float lumaAt(vec2 p) {
    return luma(IMG_INPUT(p).rgb);
}

void main() {
    vec4 src = IMG_INPUT(uv);
    float s = sizeScale();
    vec2 e = 1.5 * s / RENDERSIZE;

    // Sobel edges.
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
    float edge = clamp(length(vec2(gx, gy)), 0.0, 1.0);

    float v = 1.0 - luma(src.rgb);
    v = (v - 0.5) * contrast + 0.5 + brightness;
    v = clamp(v + edge * edgeGlow, 0.0, 1.0);
    v += (hash12(uv * RENDERSIZE + floor(TIME * 24.0) * 17.0) - 0.5) * grain * 0.3;
    float line = 0.5 + 0.5 * sin(uv.y * RENDERSIZE.y * 3.14159265 / max(2.0 * s, 1.0));
    v *= 1.0 - scanLines * 0.25 * line;
    v = clamp(v, 0.0, 1.0);

    vec3 col = v * tint;
    col = mix(col, vec3(1.0), smoothstep(0.85, 1.0, v) * 0.6);
    fragColor = vec4(mix(src.rgb, col, mixAmount), src.a);
}
