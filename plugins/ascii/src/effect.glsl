// ASCII: the clip redrawn as text. Each character cell samples the image
// and picks a glyph by tone from a 10-step ramp, drawn from a built-in 5x7
// bitmap font (no texture needed). Blocks mode uses shade patterns instead.

// Ramp " -.:+=%*#@", sparse to dense (by lit pixel count). Seven rows per
// glyph, top row first; bit 4 is the leftmost pixel.
const int GLYPHS[70] = int[70](
     0,  0,  0,  0,  0,  0,  0,   // ' '
     0,  0,  0, 14,  0,  0,  0,   // '-'
     0,  0,  0,  0,  0, 12, 12,   // '.'
     0, 12, 12,  0, 12, 12,  0,   // ':'
     0,  4,  4, 31,  4,  4,  0,   // '+'
     0,  0, 31,  0, 31,  0,  0,   // '='
    24, 25,  2,  4,  8, 19,  3,   // '%'
     0, 21, 14, 31, 14, 21,  0,   // '*'
    10, 10, 31, 10, 31, 10, 10,   // '#'
    14, 17, 23, 21, 23, 16, 14    // '@'
);

void main() {
    vec4 src = IMG_INPUT(uv);
    // Cells are 6x8 font pixels: a 5x7 glyph plus one pixel of spacing.
    float h = max(charSize * sizeScale(), 4.0);
    vec2 cellSize = vec2(h * 0.75, h);
    vec2 px = uv * RENDERSIZE;
    vec2 cell = floor(px / cellSize);
    vec2 f = px / cellSize - cell;
    vec3 c = IMG_INPUT((cell + 0.5) * cellSize / RENDERSIZE).rgb;
    float l = clamp((luma(c) - 0.5) * contrast + 0.5 + brightness, 0.0, 1.0);
    // Dense glyphs go where the image is dark, unless inverted (for light
    // text on a dark background).
    float amount = invert ? l : 1.0 - l;

    float on;
    if (charset == 1) {
        // Blocks: 5 shade levels as ordered-dither fills of the cell.
        float level = floor(amount * 4.999) / 4.0;
        vec2 sub = floor(f * vec2(6.0, 8.0));
        on = step(bayer4(sub) + 0.001, level);
    } else {
        int index = int(clamp(floor(amount * 10.0), 0.0, 9.0));
        int gx = int(f.x * 6.0);
        int gy = int((1.0 - f.y) * 8.0);   // uv y points up; glyph rows go down
        on = 0.0;
        if (gx < 5 && gy < 7) {
            on = float((GLYPHS[index * 7 + gy] >> (4 - gx)) & 1);
        }
    }
    vec3 ink = imageColour ? c : textColour;
    vec3 col = mix(backgroundColour, ink, on);
    fragColor = vec4(mix(src.rgb, col, mixAmount), src.a);
}
