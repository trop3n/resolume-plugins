// Bar Mosaic: rows of rounded, lit bars. Sizes are pixels at 1080p.
//
// Pass 0: blur the clip's luma across (Warp Blur).
// Pass 1: blur that down.
// Pass 2: the warp field d = u - a / barWidth (see below).
// Pass 3: running max of u along the bars, Streak Length doubling steps.
// Pass 4: slope-limited running min, so u never climbs faster than
//         1 / (Min Bar * barWidth): no bar gets shorter than Min Bar.
// Pass 5: draw the bars.
//
// Along each row, a bar is a run of pixels where floor(u) is the same, with
//   u(a) = lens(a) / barWidth + lumaWarp * blurredLuma(a) + phase
// Flat areas get even bars that line up into columns from row to row; the
// columns bend where brightness changes. Where the warp runs backwards (u
// falling along the row) pass 3's running max holds u flat, so that stretch
// becomes one long streak instead of a run of reversed bars. The field is
// stored minus a / barWidth so it stays small enough for 16-bit floats.
// Each pixel finds its bar's two ends by stepping out and bisecting.
//
// Blocks: a block of Block Scale N is N bars merged along the row and N
// rows tall, in the same warped coordinate (u / N), so it needs no extra
// passes and lines up with the bars around it. A pixel uses the block when
// the block's average luma is inside the Block Low..High band.
//
// Key: each bar (or block) is kept or dropped as a whole, by its average
// luma against the Key Low..High band; dropped bars show Outside instead.

#define BLUR_TAPS 12
#define MAX_STEPS 48
#define BISECT 7
#define COLOUR_TAPS 8

// Set in main() for the bar pass; findBar() rescales them per layer.
bool vert;
vec2 AB;        // frame size along / across the bars
float baseW;    // bar width at scale 1, pixels
float baseRowH; // row height at scale 1, pixels
float curScale; // 1 for bars, Block Scale for blocks
float barW;     // baseW * curScale
float rowH;     // baseRowH * curScale
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

// u along the current row, from the scanned field, in units of the
// current layer's bars (blocks divide the bar coordinate by their scale).
float barU(float a) {
    vec2 p = clamp(toPx(a, rowCentre(a)) / RENDERSIZE, 0.0, 1.0);
    return (a / baseW + texture(passIn, p).r + phase) / curScale;
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

// 1 inside lo..hi, eased over soft. An edge at 0 or 1 is open, so the
// full range keeps pure black and pure white.
float band(float x, float lo, float hi, float soft) {
    float sf = max(soft, 1e-4);
    float l = lo <= 0.0 ? 1.0 : smoothstep(lo - sf, lo + sf, x);
    float h = hi >= 1.0 ? 1.0 : 1.0 - smoothstep(hi - sf, hi + sf, x);
    return l * h;
}

struct Bar {
    float x0;   // ends along the row, pixels
    float x1;
    float fy;   // 0..1 across the row
    vec3 raw;   // the clip's average colour under the bar
};

// The bar (scale 1) or block (scale N) under position a, where bw is the
// across-row coordinate with the row wave added. Leaves the globals set
// for that layer, so shading uses the right row height.
Bar findBar(float a, float bw, float scale) {
    curScale = scale;
    barW = baseW * scale;
    rowH = baseRowH * scale;
    row = floor(bw / rowH);
    phase = TIME * scroll + rowJitter * hash12(vec2(row, 17.3 + scale)) * 4.0;

    Bar r;
    r.fy = bw / rowH - row;
    float k = floor(barU(a));
    r.x0 = barEnd(a, k, -1.0);
    r.x1 = barEnd(a, k, 1.0);

    // Average along the bar, two lines per row.
    vec3 c = vec3(0.0);
    for (int i = 0; i < COLOUR_TAPS; i++) {
        float ai = mix(r.x0, r.x1, (float(i) + 0.5) / float(COLOUR_TAPS));
        float bi = rowCentre(ai);
        c += IMG_INPUT(clamp(toPx(ai, bi - rowH * 0.25) / RENDERSIZE, 0.0, 1.0)).rgb;
        c += IMG_INPUT(clamp(toPx(ai, bi + rowH * 0.25) / RENDERSIZE, 0.0, 1.0)).rgb;
    }
    r.raw = c / float(2 * COLOUR_TAPS);
    return r;
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
    baseW = max(barWidth * s, 1.0);
    baseRowH = max(rowHeight * s, 2.0);
    curScale = 1.0;
    barW = baseW;
    rowH = baseRowH;
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
    if (PASSINDEX == 4) {
        // w(a) = min(w(a), w(a - st) + st * (1 / minBar - 1) / barW): the
        // largest u below the field whose slope stays under 1 / (minBar *
        // barW). Without it, steep brightness climbs squeeze bars into
        // slivers that flicker.
        float st = max(1.0, floor(exp2(float(ITERATION)) * s));
        float w = IMG_PASS(uv).r;
        if (minBar > 0.0 && a - st >= 0.0) {
            float rise = st * (1.0 / minBar - 1.0) / barW;
            w = min(w, IMG_PASS(uv - toPx(st, 0.0) / RENDERSIZE).r + rise);
        }
        fragColor = vec4(w, 0.0, 0.0, 1.0);
        return;
    }

    vec4 src = IMG_INPUT(uv);
    float g = gap * s;
    float bw = (vert ? px.x : px.y) + waveAt(a);

    // Block if this spot's block falls in the block band, else a bar.
    Bar br;
    bool isBlock = false;
    if (blockScale > 1) {
        br = findBar(a, bw, float(blockScale));
        isBlock = band(luma(br.raw), blockLow, blockHigh, 0.0) > 0.5;
    }
    if (!isBlock) {
        br = findBar(a, bw, 1.0);
    }
    float x0 = br.x0;
    float x1 = br.x1;
    float fy = br.fy;
    float len = max(x1 - x0, 1.0);
    vec3 c = pow(max(br.raw, 0.0), vec3(gamma)) * tint;

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
    vec3 col = mix(gapColour, mix(body, capCol, capM), mask);

    // Key: keep the whole bar or show Outside in its place.
    float kv = band(luma(br.raw), keyLow, keyHigh, keySoftness);
    if (keyInvert) {
        kv = 1.0 - kv;
    }
    vec3 outside = src.rgb;
    if (keyOutside == 1) {
        outside = mix(gapColour, capCol, capM * mask);   // dashes only
    } else if (keyOutside == 2) {
        outside = gapColour;
    }
    col = mix(outside, col, kv);
    fragColor = vec4(mix(src.rgb, col, mixAmount), src.a);
}
