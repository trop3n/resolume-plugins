// Halftone: a rotated grid of dots (or lines, or squares) sized by tone,
// in one ink (Mono) or four (CMYK, at the classic screen angles).

mat2 rot(float a) {
    float c = cos(a);
    float s = sin(a);
    return mat2(c, -s, s, c);
}

vec3 graded(vec2 p) {
    return clamp((IMG_INPUT(p).rgb - 0.5) * contrast + 0.5 + brightness, 0.0, 1.0);
}

// Ink amount 0..1: darkness for mono (channel < 0), else C, M, Y or K.
float inkAmount(vec3 c, int channel) {
    if (channel < 0) return 1.0 - luma(c);
    float k = 1.0 - max(c.r, max(c.g, c.b));
    if (channel == 3) return k;
    float v = channel == 0 ? c.r : channel == 1 ? c.g : c.b;
    return (1.0 - v - k) / max(1.0 - k, 1e-3);
}

// Coverage 0..1 of one screen at pixel px. Each cell's dot is sized by
// the image at the cell centre, so dots stay round.
float screenCoverage(vec2 px, float angle, int channel) {
    float pitch = max(cellSize * sizeScale(), 2.0);
    mat2 r = rot(radians(angle));
    vec2 q = r * px / pitch;
    vec2 cell = floor(q) + 0.5;
    vec2 f = q - cell;
    vec2 centrePx = transpose(r) * cell * pitch;
    float amount = clamp(inkAmount(graded(centrePx / RENDERSIZE), channel), 0.0, 1.0);
    float aa = 1.0 / pitch;
    if (shape == 1) {
        return 1.0 - smoothstep(amount * 0.5 - aa, amount * 0.5 + aa, abs(f.y));
    }
    if (shape == 2) {
        float h = sqrt(amount) * 0.5;
        return 1.0 - smoothstep(h - aa, h + aa, max(abs(f.x), abs(f.y)));
    }
    // Circle with area = amount until dots touch, then grow to fill the corners.
    float r0 = amount <= 0.785 ? sqrt(amount / 3.14159265) : mix(0.5, 0.7072, (amount - 0.785) / 0.215);
    return 1.0 - smoothstep(r0 - aa, r0 + aa, length(f));
}

void main() {
    vec4 src = IMG_INPUT(uv);
    vec2 px = uv * RENDERSIZE;
    vec3 col;
    if (mode == 0) {
        col = mix(paperColour, inkColour, screenCoverage(px, angle, -1));
    } else {
        // Inks multiply the paper. With Angle at 45: C 15, M 75, Y 0, K 45.
        col = paperColour;
        col *= mix(vec3(1.0), vec3(0.0, 0.68, 0.94), screenCoverage(px, angle - 30.0, 0));
        col *= mix(vec3(1.0), vec3(0.93, 0.0, 0.55), screenCoverage(px, angle + 30.0, 1));
        col *= mix(vec3(1.0), vec3(1.0, 0.94, 0.0), screenCoverage(px, angle - 45.0, 2));
        col *= mix(vec3(1.0), inkColour, screenCoverage(px, angle, 3));
    }
    fragColor = vec4(mix(src.rgb, col, mixAmount), src.a);
}
