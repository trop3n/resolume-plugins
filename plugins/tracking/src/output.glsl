// Pass 3: desaturated picture with the tracking HUD on top: body box (or
// corner brackets), a head box guessed from the top of the body box,
// crosshairs through the centre, a "TRK 01" tag and a live X/Y readout.

// Small 5x7 font: digits 0-9, then T R K X Y '.' and space. Seven rows per
// glyph, top row first; bit 4 is the leftmost pixel.
const int G_T = 10, G_R = 11, G_K = 12, G_X = 13, G_Y = 14, G_DOT = 15, G_SPACE = 16;
const int FONT[119] = int[119](
    14, 17, 19, 21, 25, 17, 14,   // 0
     4, 12,  4,  4,  4,  4, 14,   // 1
    14, 17,  1,  2,  4,  8, 31,   // 2
    31,  2,  4,  2,  1, 17, 14,   // 3
     2,  6, 10, 18, 31,  2,  2,   // 4
    31, 16, 30,  1,  1, 17, 14,   // 5
     6,  8, 16, 30, 17, 17, 14,   // 6
    31,  1,  2,  4,  8,  8,  8,   // 7
    14, 17, 17, 14, 17, 17, 14,   // 8
    14, 17, 17, 15,  1,  2, 12,   // 9
    31,  4,  4,  4,  4,  4,  4,   // T
    30, 17, 17, 30, 20, 18, 17,   // R
    17, 18, 20, 24, 20, 18, 17,   // K
    17, 17, 10,  4, 10, 17, 17,   // X
    17, 17, 10,  4,  4,  4,  4,   // Y
     0,  0,  0,  0,  0, 12, 12,   // .
     0,  0,  0,  0,  0,  0,  0    // space
);

// 1 where glyph g has a lit pixel at cell-local font coords f (0..6 x 0..8,
// y up; the bottom row and right column are spacing).
float glyph(int g, vec2 f) {
    int gx = int(f.x);
    int gy = 7 - int(f.y);   // font rows go down
    if (gx < 0 || gx > 4 || gy < 0 || gy > 6) return 0.0;
    return float((FONT[g * 7 + gy] >> (4 - gx)) & 1);
}

// The tag "TRK 01".
int tagChar(int i) {
    return i == 0 ? G_T : i == 1 ? G_R : i == 2 ? G_K : i == 3 ? G_SPACE : i == 4 ? 0 : 1;
}

// The readout "X.123 Y.456" for a position in 0..1.
int readoutChar(int i, vec2 v) {
    ivec2 n = ivec2(clamp(v, 0.0, 0.999) * 1000.0);
    if (i == 0) return G_X;
    if (i == 1 || i == 7) return G_DOT;
    if (i == 5) return G_SPACE;
    if (i == 6) return G_Y;
    int value = i < 5 ? n.x : n.y;
    int k = i < 5 ? i - 2 : i - 8;   // digit 0..2, most significant first
    int div = k == 0 ? 100 : k == 1 ? 10 : 1;
    return (value / div) % 10;
}

// Text mask for a run of `len` characters starting at `origin` (bottom-left).
float textMask(vec2 px, vec2 origin, float fontPx, int len, bool readout, vec2 v) {
    vec2 f = (px - origin) / fontPx;
    if (f.x < 0.0 || f.y < 0.0 || f.y >= 8.0) return 0.0;
    int i = int(f.x / 6.0);
    if (i >= len) return 0.0;
    int g = readout ? readoutChar(i, v) : tagChar(i);
    return glyph(g, vec2(mod(f.x, 6.0), f.y));
}

// Outline of the rectangle lo..hi, lw pixels thick. Corners style keeps
// only brackets at each corner.
float rectOutline(vec2 px, vec2 lo, vec2 hi, float lw, bool corners) {
    vec2 a = px - lo;
    vec2 b = hi - px;
    float inside = step(0.0, min(min(a.x, a.y), min(b.x, b.y)));
    float edge = step(min(min(a.x, a.y), min(b.x, b.y)), lw);
    float m = inside * edge;
    if (corners) {
        float len = 0.22 * min(hi.x - lo.x, hi.y - lo.y);
        vec2 dc = min(a, b);   // distance to the nearest vertical / horizontal side
        m *= step(dc.x, len) + step(dc.y, len) > 1.5 ? 1.0 : 0.0;
    }
    return m;
}

void main() {
    vec4 src = IMG_INPUT(uv);
    float s = sizeScale();
    vec2 px = uv * RENDERSIZE;

    vec3 col = mix(src.rgb, vec3(luma(src.rgb)), desaturate);
    if (showMotion) col = mix(col, vec3(1.0, 0.0, 0.0), texture(energy, uv).r * 0.7);

    vec4 box = texelFetch(track, ivec2(0, 0), 0);
    vec4 info = texelFetch(track, ivec2(1, 0), 0);
    float conf = info.y > 0.5 ? info.x : 0.0;

    if (conf > 0.0) {
        float lw = max(lineWidth * s, 1.0);
        vec2 lo = (box.xy - box.zw) * RENDERSIZE;
        vec2 hi = (box.xy + box.zw) * RENDERSIZE;
        vec2 centre = box.xy * RENDERSIZE;
        float hud = rectOutline(px, lo, hi, lw, boxStyle == 1);

        if (headBox) {
            // Just under the top of the strong motion, inside the body box.
            float h = min(hi.x - lo.x, hi.y - lo.y) * 0.18;
            float topPx = min(info.z * RENDERSIZE.y, hi.y);
            vec2 hc = vec2(centre.x, max(topPx - h, lo.y + h));
            hud = max(hud, rectOutline(px, hc - h, hc + h, lw, false));
        }
        if (crosshair) {
            float thin = max(s, 1.0);
            float lines = step(abs(px.x - centre.x), thin * 0.5) + step(abs(px.y - centre.y), thin * 0.5);
            hud = max(hud, min(lines, 1.0) * 0.45);
            hud = max(hud, step(abs(length(px - centre) - 10.0 * s), lw * 0.5));
        }

        vec3 hudCol = boxColour;
        if (labels) {
            float fontPx = max(2.0 * s, 1.0);
            // Tag on a filled block above the box's top-left corner, and the
            // readout under it; both kept inside the frame.
            vec2 tagSize = vec2(6.0 * 6.0, 8.0) * fontPx;
            vec2 readSize = vec2(11.0 * 6.0, 8.0) * fontPx;
            vec2 tagOrigin = clamp(vec2(lo.x, hi.y + 4.0 * fontPx), vec2(3.0 * fontPx), RENDERSIZE - tagSize - 3.0 * fontPx);
            vec2 tagLo = tagOrigin - fontPx * 2.0;
            vec2 tagHi = tagOrigin + tagSize + fontPx;
            float tagBg = step(tagLo.x, px.x) * step(px.x, tagHi.x) * step(tagLo.y, px.y) * step(px.y, tagHi.y);
            float tagText = textMask(px, tagOrigin, fontPx, 6, false, vec2(0.0));
            col = mix(col, boxColour, tagBg * conf);
            col = mix(col, vec3(1.0), tagText * conf);
            // Live centre position under the box.
            vec2 readOrigin = clamp(vec2(lo.x, lo.y - 12.0 * fontPx), vec2(3.0 * fontPx), RENDERSIZE - readSize - 3.0 * fontPx);
            hud = max(hud, textMask(px, readOrigin, fontPx, 11, true, box.xy));
        }
        col = mix(col, hudCol, hud * conf);
    }
    fragColor = vec4(mix(src.rgb, col, mixAmount), src.a);
}
