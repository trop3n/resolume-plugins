// Teletext: Ceefax-style block graphics. The screen is a grid of character
// cells, each split into 2x3 mosaic blocks, coloured from the eight
// teletext colours (the corners of the RGB cube).
//
// Free: every block picks its own colour.
// One Colour per Cell: like real teletext, each cell has one foreground
// colour; its blocks are either that colour or the background.

vec3 boosted(vec3 c) {
    c = clamp((c - 0.5) * contrast + 0.5, 0.0, 1.0);
    float l = luma(c);
    return clamp(mix(vec3(l), c, saturation), 0.0, 1.0);
}

void main() {
    vec4 src = IMG_INPUT(uv);
    float cw = RENDERSIZE.x / float(columns);
    vec2 cellSize = vec2(cw, cw * 1.2);
    vec2 blockSize = cellSize / vec2(2.0, 3.0);
    vec2 px = uv * RENDERSIZE;
    vec2 cell = floor(px / cellSize);
    vec2 block = floor(px / blockSize);
    vec2 bf = px / blockSize - block;

    vec3 c = boosted(IMG_INPUT((block + 0.5) * blockSize / RENDERSIZE).rgb);
    float t = mix(0.5, bayer2(block), dither);
    vec3 col;
    if (mode == 1) {
        // The cell's colour: its average, snapped to the nearest bright
        // teletext colour (never black, so it shows on the background).
        vec3 avg = boosted(IMG_INPUT((cell + 0.5) * cellSize / RENDERSIZE).rgb);
        vec3 fg = step(0.5, avg);
        if (dot(fg, vec3(1.0)) < 0.5) fg = vec3(1.0);
        bool lit = luma(c) > t;
        col = lit ? fg : backgroundColour;
    } else {
        col = step(vec3(t), c);
    }

    // Separated graphics: a gap around each block shows the background.
    if (separated) {
        vec2 gap = step(vec2(0.18), bf) * step(bf, vec2(0.82));
        col = mix(backgroundColour, col, gap.x * gap.y);
    }
    // Faint CRT lines.
    float line = step(0.5, fract(px.y / max(2.0 * sizeScale(), 1.0)));
    col *= 1.0 - scanLines * 0.25 * line;
    fragColor = vec4(mix(src.rgb, col, mixAmount), src.a);
}
