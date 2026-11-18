// Bar Mosaic: rows of rounded, lit bars. Sizes are pixels at 1080p.
//
// Pass 0: blur the clip's luma across (Warp Blur).
// Pass 1: blur that down.
// Pass 2: the warp field d = u - a / barWidth (see below).
// Pass 3: running max of u along the bars, Streak Length doubling steps.
// Pass 4: draw the bars.
//
// Along each row, a bar is a run of pixels where floor(u) is the same, with
//   u(a) = lens(a) / barWidth + lumaWarp * blurredLuma(a) + phase
// Flat areas get even bars that line up into columns from row to row; the
// columns bend where brightness changes. Where the warp runs backwards (u
// falling along the row) pass 3's running max holds u flat, so that stretch
// becomes one long streak instead of a run of reversed bars. The field is
// stored minus a / barWidth so it stays small enough for 16-bit floats.
// Each pixel finds its bar's two ends by stepping out and bisecting.

#define BLUR_TAPS 12
#define MAX_STEPS 48
#define BISECT 7
#define COLOUR_TAPS 8

// Set in main() for the bar pass.
bool vert;
vec2 AB;        // frame size along / across the bars
float barW;
float rowH;
float row;
float phase;

// "a" runs along the bars, "b" across them.
vec2 toPx(float a, float b) { return vert ? vec2(b, a) : vec2(a, b); }

float waveAt(float a) {
    return rowWave * sizeScale() * sin(6.28318 * (a / AB.y * waveFreq + TIME * waveSpeed));
}

// Centre of the current row (in b) at position a.
float rowCentre(float a) {
    return (row + 0.5) * rowH - waveAt(a);
}

// Warp field (pass 2): u minus its plain a / barW part, at (a, b).
float warpField(float a, float b, float L) {
    vec2 n = (vec2(a, b) - AB * 0.5) / (AB.y * 0.5);
    return (a - AB.x * 0.5) * lens * dot(n, n) / barW + lumaWarp * L;
}

// u along the current row, from the scanned field (pass 4).
float barU(float a) {
    vec2 p = clamp(toPx(a, rowCentre(a)) / RENDERSIZE, 0.0, 1.0);
    return a / barW + texture(passIn, p).r + phase;
}

// End of the run of floor(u) == k from a, going in direction dir (-1 / +1).
float barEnd(float a, float k, float dir) {
    float st = barW * 0.5 * dir;
    float inside = a;
    float outside = a;
    bool hit = false;
    for (int i = 1; i <= MAX_STEPS; i++) {
        float t = a + st * float(i);
        if (t < 0.0 || t > AB.x) {
            return clamp(t, 0.0, AB.x);
        }
        if (floor(barU(t)) != k) {
            outside = t;
            hit = true;
            break;
        }
        inside = t;
    }
    if (!hit) {
        return inside;
    }
    for (int j = 0; j < BISECT; j++) {
        float m = 0.5 * (inside + outside);
        if (floor(barU(m)) == k) {
            inside = m;
        } else {
            outside = m;
        }
    }
    return 0.5 * (inside + outside);
}

float roundRect(vec2 p, vec2 halfSize, float r) {
    vec2 q = abs(p) - (halfSize - r);
    return length(max(q, 0.0)) + min(max(q.x, q.y), 0.0) - r;
}

void main() {
    float blurPx = warpBlur * sizeScale();

    if (PASSINDEX == 0) {
        vec2 st = vec2(blurPx / RENDERSIZE.x, 0.0) / float(BLUR_TAPS);
        float sum = 0.0;
        float norm = 0.0;
        for (int i = -BLUR_TAPS; i <= BLUR_TAPS; i++) {
            float x = float(i) / float(BLUR_TAPS);
            float w = exp(-x * x * 4.5);
            sum += luma(IMG_INPUT(clamp(uv + st * float(i), 0.0, 1.0)).rgb) * w;
            norm += w;
        }
        fragColor = vec4(sum / norm, 0.0, 0.0, 1.0);
        return;
    }
    if (PASSINDEX == 1) {
        vec2 st = vec2(0.0, blurPx / RENDERSIZE.y) / float(BLUR_TAPS);
        float sum = 0.0;
        float norm = 0.0;
        for (int i = -BLUR_TAPS; i <= BLUR_TAPS; i++) {
            float x = float(i) / float(BLUR_TAPS);
            float w = exp(-x * x * 4.5);
            sum += IMG_PASS(clamp(uv + st * float(i), 0.0, 1.0)).r * w;
            norm += w;
        }
        fragColor = vec4(sum / norm, 0.0, 0.0, 1.0);
        return;
    }

    float s = sizeScale();
    vert = orientation == 1;
    AB = vert ? RENDERSIZE.yx : RENDERSIZE;
    barW = max(barWidth * s, 1.0);
    rowH = max(rowHeight * s, 2.0);
    vec2 px = uv * RENDERSIZE;
    float a = vert ? px.y : px.x;

    if (PASSINDEX == 2) {
        float L = IMG_PASS(uv).r;
        fragColor = vec4(warpField(a, vert ? px.x : px.y, L), 0.0, 0.0, 1.0);
        return;
    }
    if (PASSINDEX == 3) {
        // m(a) = max(m(a), m(a - st) - st / barW): after n steps, the max of
        // u over the 2^n * scale pixels before a, relative to a / barW.
        float st = max(1.0, floor(exp2(float(ITERATION)) * s));
        float m = IMG_PASS(uv).r;
        if (a - st >= 0.0) {
            m = max(m, IMG_PASS(uv - toPx(st, 0.0) / RENDERSIZE).r - st / barW);
        }
        fragColor = vec4(m, 0.0, 0.0, 1.0);
        return;
    }

    vec4 src = IMG_INPUT(uv);
    float g = gap * s;
    float b = (vert ? px.x : px.y) + waveAt(a);
    row = floor(b / rowH);
    float fy = b / rowH - row;
    phase = TIME * scroll + rowJitter * hash12(vec2(row, 17.3)) * 4.0;

    float k = floor(barU(a));
    float x0 = barEnd(a, k, -1.0);
    float x1 = barEnd(a, k, 1.0);
    float len = max(x1 - x0, 1.0);

    // Bar colour: the clip's average along the bar, two lines per row.
    vec3 c = vec3(0.0);
    for (int i = 0; i < COLOUR_TAPS; i++) {
        float ai = mix(x0, x1, (float(i) + 0.5) / float(COLOUR_TAPS));
        float bi = rowCentre(ai);
        c += IMG_INPUT(clamp(toPx(ai, bi - rowH * 0.25) / RENDERSIZE, 0.0, 1.0)).rgb;
        c += IMG_INPUT(clamp(toPx(ai, bi + rowH * 0.25) / RENDERSIZE, 0.0, 1.0)).rgb;
    }
    c = pow(max(c / float(2 * COLOUR_TAPS), 0.0), vec3(gamma)) * tint;

    // Rounded bar with a gap all round.
    vec2 halfSize = max(vec2(len, rowH) * 0.5 - g * 0.5, vec2(0.5));
    vec2 p = vec2(a - x0 - len * 0.5, (fy - 0.5) * rowH);
    float r = roundness * min(halfSize.x, halfSize.y);
    float mask = clamp(0.5 - roundRect(p, halfSize, r), 0.0, 1.0);

    // Brighter toward the far end, with a lit cap on it. Cap Floor keeps
    // the cap visible (in the bar's hue) where the clip is dark, which
    // leaves a field of thin dashes over black.
    float t = clamp((a - x0) / len, 0.0, 1.0);
    vec3 body = c * (1.0 - ramp + ramp * t);
    float capW = max(1.0, 1.5 * s);
    float capM = clamp(1.0 - (x1 - g * 0.5 - a) / capW, 0.0, 1.0);
    float peak = max(max(c.r, c.g), c.b);
    vec3 hue = peak > 1e-4 ? c / peak : tint;
    vec3 capCol = max(c * (1.0 + capGlow) + capGlow * 0.1, hue * capFloor);
    vec3 col = mix(body, capCol, capM);

    col = mix(gapColour, col, mask);
    fragColor = vec4(mix(src.rgb, col, mixAmount), src.a);
}
