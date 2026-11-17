// Data Prism: ported from prism-ffgl's DataPrism.fs (ISF) to ffgl-fx.
// Parameters keep their ISF names as uniforms; see lib.rs.

// Port of Prism Lab's Blender "Data Points" overlay, frame layers only.
// The 3D target object becomes a box (Center, Width, Height); every layer is
// drawn around it in frame-height units, scaled by r = half the box diagonal
// (the add-on scales by half the bounding-box diagonal the same way).
// ffgl-isf can't import a font texture, so text uses a built-in 3x5 pixel font.

#define TAU 6.28318530718
#define ADVANCE 4.0

vec2 rotate2(vec2 v, float a) {
    float s = sin(a);
    float c = cos(a);
    return vec2(c * v.x - s * v.y, s * v.x + c * v.y);
}

float sdSegment(vec2 p, vec2 a, vec2 b) {
    vec2 pa = p - a;
    vec2 ba = b - a;
    float h = clamp(dot(pa, ba) / max(dot(ba, ba), 1e-12), 0.0, 1.0);
    return length(pa - ba * h);
}

// Antialiased line coverage for a distance d (frame heights).
float stroke(float d) {
    float px = 1.0 / RENDERSIZE.y;
    float w = max(LineWeight, 0.5) * px;
    return clamp((0.5 * w - d) / px + 0.5, 0.0, 1.0);
}

// ---------------------------------------------------------------------------
// 3x5 pixel font. Codes: 0-9 digits, 10 '-', 11 '.', 12 space, 13 P, 14 T,
// 15 W, 16 L, 17 O, 18 C, 19 K, 20 X, 21 Y. Bit (row * 3 + col), row 0 at the top.
// ---------------------------------------------------------------------------

float glyphBits(int c) {
    if (c == 0) return 31599.0;
    if (c == 1) return 29850.0;
    if (c == 2) return 29671.0;
    if (c == 3) return 31143.0;
    if (c == 4) return 18925.0;
    if (c == 5) return 31183.0;
    if (c == 6) return 31695.0;
    if (c == 7) return 9383.0;
    if (c == 8) return 31727.0;
    if (c == 9) return 31215.0;
    if (c == 10) return 448.0;
    if (c == 11) return 8192.0;
    if (c == 13) return 4843.0;
    if (c == 14) return 9367.0;
    if (c == 15) return 24429.0;
    if (c == 16) return 29257.0;
    if (c == 17) return 31599.0;
    if (c == 18) return 29263.0;
    if (c == 19) return 22109.0;
    if (c == 20) return 23213.0;
    if (c == 21) return 9389.0;
    return 0.0;
}

// Draw glyph c with its bottom-left at o (font pixel size s), then move o to the next cell.
float putChar(vec2 p, inout vec2 o, float s, int c) {
    vec2 g = (p - o) / s;
    o.x += ADVANCE * s;
    if (g.x < 0.0 || g.y < 0.0 || g.x >= 3.0 || g.y >= 5.0) return 0.0;
    float bit = floor(g.x) + (4.0 - floor(g.y)) * 3.0;
    float q = floor(glyphBits(c) / exp2(bit));
    return q - 2.0 * floor(q * 0.5);
}

float pow10(int n) {
    float r = 1.0;
    for (int i = 0; i < 8; i++) {
        if (i >= n) break;
        r *= 10.0;
    }
    return r;
}

// Digit k (0 = ones) of a non-negative whole number x stored as a float.
int digitAt(float x, int k) {
    float q = floor((x + 0.5) / pow10(k));
    return int(q - 10.0 * floor((q + 0.5) / 10.0));
}

int intDigits(float x) {
    int n = 1;
    for (int i = 1; i < 8; i++) {
        if (x + 0.5 >= pow10(i)) n = i + 1;
    }
    return n;
}

// Characters needed to print v (>= 0) with this many decimals.
float numLength(float v, int decimals) {
    float whole = floor(floor(v * pow10(decimals) + 0.5) / pow10(decimals) + 1e-4);
    return float(intDigits(whole) + (decimals > 0 ? decimals + 1 : 0));
}

float putNum(vec2 p, inout vec2 o, float s, float v, int decimals) {
    float scaled = floor(max(v, 0.0) * pow10(decimals) + 0.5);
    int n = intDigits(floor((scaled + 0.5) / pow10(decimals)));
    float cov = 0.0;
    for (int i = 0; i < 8; i++) {
        if (i >= n) break;
        cov += putChar(p, o, s, digitAt(scaled, decimals + n - 1 - i));
    }
    if (decimals > 0) {
        cov += putChar(p, o, s, 11);
        for (int i = 0; i < 4; i++) {
            if (i >= decimals) break;
            cov += putChar(p, o, s, digitAt(scaled, decimals - 1 - i));
        }
    }
    return cov;
}

// ---------------------------------------------------------------------------
// Layers. c = box center, hs = box half size, r = half diagonal, t = animated time.
// ---------------------------------------------------------------------------

// Scan Brackets: corner brackets on the box, breathing outward by Gap.
float layerBrackets(vec2 p, vec2 c, vec2 hs, float t) {
    float grow = 1.0 + BracketGap * (1.0 + 0.15 * sin(t * 3.0));
    vec2 h = hs * grow;
    vec2 q = abs(p - c);
    vec2 arm = 2.0 * h * BracketLength;
    float d = min(sdSegment(q, vec2(h.x - arm.x, h.y), h), sdSegment(q, vec2(h.x, h.y - arm.y), h));
    return stroke(d);
}

// Object Lock Frame: pulsing square corners, center cross ticks and a LOCK X/Y readout.
float layerLock(vec2 p, vec2 c, float r, float t, float s) {
    float a = r * LockSize * (1.0 + 0.03 * sin(t * 4.0));
    vec2 q = abs(p - c);
    float k = a * 0.3;
    float d = min(sdSegment(q, vec2(a - k, a), vec2(a)), sdSegment(q, vec2(a, a - k), vec2(a)));
    d = min(d, min(sdSegment(q, vec2(a * 0.08, 0.0), vec2(a * 0.2, 0.0)),
                   sdSegment(q, vec2(0.0, a * 0.08), vec2(0.0, a * 0.2))));
    float cov = stroke(d);

    vec2 o = c + vec2(-a, a + r * 0.03);
    float txt = putChar(p, o, s, 16) + putChar(p, o, s, 17) + putChar(p, o, s, 18) + putChar(p, o, s, 19);
    o.x += ADVANCE * s;
    txt += putChar(p, o, s, 20);
    o.x += ADVANCE * s;
    txt += putNum(p, o, s, Center.x, 2);
    o.x += ADVANCE * s;
    txt += putChar(p, o, s, 21);
    o.x += ADVANCE * s;
    txt += putNum(p, o, s, Center.y, 2);
    return max(cov, min(txt, 1.0));
}

// Measure Arcs: rotating arc with ticks, a counter-rotating inner arc, and a width
// dimension line under the box labelled in output pixels.
float layerArcs(vec2 p, vec2 c, vec2 hs, float r, float t, float s) {
    vec2 q = p - c;
    float R = r * ArcRadius;
    float sweep = radians(ArcSweep);
    float ang = atan(q.y, q.x);
    float len = length(q);

    float start = t * 0.2;
    float rel = mod(ang - start, TAU);
    float d = rel <= sweep ? abs(len - R) : 1e3;

    float start2 = -t * 0.12;
    float d2 = mod(ang - start2, TAU) <= sweep * 0.45 ? abs(len - R * 0.93) : 1e3;
    d = min(d, d2);

    float n = floor(ArcTicks);
    if (n >= 2.0 && sweep > 0.0) {
        float k = clamp(floor(rel / sweep * (n - 1.0) + 0.5), 0.0, n - 1.0);
        float ta = start + k / (n - 1.0) * sweep;
        vec2 dir = vec2(cos(ta), sin(ta));
        float major = (k - 6.0 * floor((k + 0.5) / 6.0)) < 0.5 ? 2.2 : 1.0;
        d = min(d, sdSegment(q, dir * R, dir * (R + r * 0.05 * major)));
    }

    // Dimension line below the box, with end ticks.
    float y = c.y - hs.y - r * 0.25;
    float x0 = c.x - hs.x;
    float x1 = c.x + hs.x;
    float th = r * 0.05;
    d = min(d, sdSegment(p, vec2(x0, y), vec2(x1, y)));
    d = min(d, sdSegment(p, vec2(x0, y - th), vec2(x0, y + th)));
    d = min(d, sdSegment(p, vec2(x1, y - th), vec2(x1, y + th)));
    float cov = stroke(d);

    // "W <px> PX", centered over the line.
    float px = floor(Width * RENDERSIZE.x + 0.5);
    float chars = 2.0 + numLength(px, 0) + 3.0;
    vec2 o = vec2(c.x - 0.5 * (chars * ADVANCE - 1.0) * s, y + th);
    float txt = putChar(p, o, s, 15);
    o.x += ADVANCE * s;
    txt += putNum(p, o, s, px, 0);
    o.x += ADVANCE * s;
    txt += putChar(p, o, s, 13) + putChar(p, o, s, 20);
    return max(cov, min(txt, 1.0));
}

// Labels: leader lines from points in the box out to "PT-n  dist" readouts.
float layerLabels(vec2 p, vec2 c, vec2 hs, float r, float seed, float s) {
    float cov = 0.0;
    int count = int(floor(LabelCount));
    for (int i = 0; i < 8; i++) {
        if (i >= count) break;
        vec2 anchor = c + (hash22(vec2(float(i) + 0.5, seed + 2.0)) * 2.0 - 1.0) * hs * 0.8;
        vec2 rel = anchor - c;
        vec2 n = length(rel) > 1e-5 ? normalize(rel) : vec2(1.0, 0.0);
        vec2 elbow = anchor + n * LabelOffset * r;
        float side = n.x >= 0.0 ? 1.0 : -1.0;
        vec2 end = elbow + vec2(side * r * 0.35, 0.0);
        float d = min(sdSegment(p, anchor, elbow), sdSegment(p, elbow, end));
        d = min(d, abs(length(p - anchor) - r * 0.012));
        cov = max(cov, stroke(d));

        // Text sits just above the end of the leader, reading away from the box.
        float dist = length(rel);
        float chars = 3.0 + float(intDigits(float(i + 1))) + 2.0 + numLength(dist, 3);
        float w = (chars * ADVANCE - 1.0) * s;
        vec2 o = vec2(side > 0.0 ? end.x : end.x - w, end.y + r * 0.02);
        float txt = putChar(p, o, s, 13) + putChar(p, o, s, 14) + putChar(p, o, s, 10);
        txt += putNum(p, o, s, float(i + 1), 0);
        o.x += 2.0 * ADVANCE * s;
        txt += putNum(p, o, s, dist, 3);
        cov = max(cov, min(txt, 1.0));
    }
    return cov;
}

// Circle Chain: a slowly turning ring of small circles (every 4th one larger).
float layerChain(vec2 p, vec2 c, float r, float t) {
    vec2 q = p - c;
    float R = r * ChainRadius;
    float n = max(floor(ChainCount), 3.0);
    float spin = t * 0.15;
    float stepA = TAU / n;
    float k = floor((atan(q.y, q.x) - spin) / stepA + 0.5);
    float ca = spin + k * stepA;
    float km = k - 4.0 * floor((k + 0.5) / 4.0);
    float linkR = R * 3.14159265 / n * 0.9 * (km < 0.5 ? 1.0 : 0.55);
    float d = abs(length(q - R * vec2(cos(ca), sin(ca))) - linkR);
    d = min(d, abs(length(q) - R));
    return stroke(d);
}

// Anchor Motifs: small diamonds with a ring inside, scattered over the box.
float layerAnchors(vec2 p, vec2 c, vec2 hs, float r, float seed) {
    float sz = AnchorSize * r * 0.06;
    float d = 1e3;
    int count = int(floor(AnchorCount));
    for (int i = 0; i < 32; i++) {
        if (i >= count) break;
        vec2 pt = c + (hash22(vec2(float(i) + 0.5, seed + 3.0)) * 2.0 - 1.0) * hs * 0.9;
        vec2 q = abs(p - pt);
        d = min(d, abs(q.x + q.y - sz * 0.70710678) * 0.70710678);
        d = min(d, abs(length(p - pt) - sz * 0.18));
    }
    return stroke(d);
}

void main() {
    vec4 original = IMG_INPUT(uv);
    float aspect = RENDERSIZE.x / RENDERSIZE.y;
    vec2 p = (uv - 0.5) * vec2(aspect, 1.0);

    vec2 c = (Center - 0.5) * vec2(aspect, 1.0);
    vec2 hs = vec2(Width * aspect, Height) * 0.5;
    float r = max(length(hs), 1e-3);
    float t = TIME * Speed;
    float seed = floor(Seed);
    // Font pixel size: text cap height = 7% of r at TextSize 1, like the add-on.
    float fs = TextSize * r * 0.07 / 5.0;

    vec3 col = original.rgb;
    if (Chain) col = mix(col, Color.rgb, layerChain(p, c, r, t) * ChainOpacity);
    if (Arcs) col = mix(col, Color.rgb, layerArcs(p, c, hs, r, t, fs) * ArcsOpacity);
    if (Anchors) col = mix(col, Accent.rgb, layerAnchors(p, c, hs, r, seed) * AnchorsOpacity);
    if (Labels) col = mix(col, Accent.rgb, layerLabels(p, c, hs, r, seed, fs) * LabelsOpacity);
    if (Brackets) col = mix(col, Accent.rgb, layerBrackets(p, c, hs, t) * BracketsOpacity);
    if (Lock) col = mix(col, Accent.rgb, layerLock(p, c, r, t, fs) * LockOpacity);

    fragColor = vec4(mix(original.rgb, col, Opacity), original.a);
}
