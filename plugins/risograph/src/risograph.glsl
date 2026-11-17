// Risograph: a two-ink stencil print. The image is separated into two
// plates, each plate is screened (grain or halftone), the second plate is
// printed slightly out of register, and both inks overprint on paper.

// Approximate sRGB of RISO ink swatches. Order matches INKS in lib.rs.
const vec3 INKS[12] = vec3[12](
    vec3(0.196, 0.333, 0.643),  // Medium Blue   #3255A4
    vec3(0.945, 0.314, 0.376),  // Bright Red    #F15060
    vec3(1.000, 0.282, 0.690),  // Fluo Pink     #FF48B0
    vec3(1.000, 0.455, 0.467),  // Fluo Orange   #FF7477
    vec3(1.000, 0.910, 0.000),  // Yellow        #FFE800
    vec3(0.000, 0.663, 0.361),  // Green         #00A95C
    vec3(0.000, 0.514, 0.541),  // Teal          #00838A
    vec3(0.000, 0.471, 0.749),  // Blue          #0078BF
    vec3(0.239, 0.333, 0.533),  // Federal Blue  #3D5588
    vec3(0.463, 0.357, 0.655),  // Purple        #765BA7
    vec3(0.569, 0.306, 0.447),  // Burgundy      #914E72
    vec3(0.090, 0.090, 0.090)   // Black
);

// Order matches PAPERS in lib.rs.
const vec3 PAPERS[5] = vec3[5](
    vec3(0.949, 0.929, 0.882),  // Natural
    vec3(0.985, 0.983, 0.975),  // White
    vec3(0.890, 0.867, 0.808),  // Newsprint
    vec3(0.776, 0.651, 0.494),  // Kraft
    vec3(0.835, 0.835, 0.820)   // Grey
);

vec3 graded(vec2 p) {
    vec3 c = IMG_INPUT(p).rgb;
    c = (c - 0.5) * contrast + 0.5 + brightness;
    return clamp(c, 0.0, 1.0);
}

// Ink amounts (0..1) for plate 1 (x) and plate 2 (y) at one colour.
vec2 separateRaw(vec3 c, vec3 ink1, vec3 ink2) {
    float dark = 1.0 - dot(c, vec3(0.299, 0.587, 0.114));
    if (separation == 0) {
        // Duotone: the same tone on both plates.
        return vec2(dark);
    }
    if (separation == 2) {
        // Shadows on plate 1, everything from the midtones down on plate 2.
        return vec2(smoothstep(0.35, 0.95, dark), smoothstep(0.08, 0.55, dark));
    }
    // Best fit: the ink amounts whose overprint best matches the colour,
    // solved by least squares in optical density, where overprints add.
    vec3 d = -log(max(c, vec3(0.004)));
    vec3 d1 = -log(max(ink1, vec3(0.02)));
    vec3 d2 = -log(max(ink2, vec3(0.02)));
    float a = dot(d1, d1);
    float b = dot(d1, d2);
    float e = dot(d2, d2);
    float r1 = dot(d1, d);
    float r2 = dot(d2, d);
    float det = a * e - b * b;
    if (det < 1e-4) {
        return vec2(dark);  // inks too alike to tell apart
    }
    vec2 x = vec2(e * r1 - b * r2, a * r2 - b * r1) / det;
    // If one ink would be negative, fit with the other ink alone.
    if (x.x < 0.0) x = vec2(0.0, r2 / e);
    if (x.y < 0.0) x = vec2(r1 / a, 0.0);
    return clamp(x, 0.0, 1.0);
}

// Light tones below the Clean Highlights level print as bare paper.
vec2 separate(vec3 c, vec3 ink1, vec3 ink2) {
    vec2 x = separateRaw(c, ink1, ink2);
    return clamp((x - highlights) / (1.0 - highlights), 0.0, 1.0);
}

// Antialiased coverage (0..1) of one screened plate at pixel p.
float screenPlate(float amount, vec2 p, float angle, float seed, float cellSize) {
    if (screenType == 1) {
        // Halftone: rotated grid of round dots whose area is the ink amount.
        float pitch = max(cellSize * 3.0, 2.0);
        float ca = cos(angle);
        float sa = sin(angle);
        vec2 q = mat2(ca, -sa, sa, ca) * p / pitch;
        vec2 f = fract(q) - 0.5;
        float r = sqrt(amount / 3.14159265);
        float aa = 0.75 / pitch;
        return 1.0 - smoothstep(r - aa, r + aa, length(f));
    }
    // Grain: a stochastic screen, like a riso master's grain-touch mode.
    vec2 cell = floor(p / cellSize);
    float t = hash12(cell + seed * 113.0);
    return step(t, amount * 1.02 - 0.01);
}

// Uneven ink lay-down: broad blotches plus pinholes where ink skipped.
float inkTexture(vec2 p, float seed, float sc) {
    float blotch = fbm(p / (70.0 * sc) + seed * 3.1);
    float pin = step(hash12(floor(p / max(sc, 1.0)) + seed * 91.0), 0.05);
    return clamp(1.0 - grain * (0.5 * blotch + 0.6 * pin), 0.0, 1.0);
}

void main() {
    float sc = RENDERSIZE.y / 1080.0;   // sizes are in pixels at 1080p
    vec2 px = uv * RENDERSIZE;
    vec3 c1 = INKS[clamp(ink1, 0, 11)];
    vec3 c2 = INKS[clamp(ink2, 0, 11)];

    // Boil re-rolls the grain and nudges the registration, a few times a second.
    float frame = boil > 0.0 ? floor(TIME * mix(2.0, 24.0, boil)) : 0.0;
    float seed = fract(frame * 0.6180339);
    vec2 jitter = boil > 0.0 ? (hash22(vec2(frame, 7.0)) - 0.5) * boil * 4.0 * sc : vec2(0.0);
    // Plate 2's offset in pixels (+Y moves it down the screen).
    vec2 shift = vec2(offsetX, -offsetY) * sc + jitter;

    float k1 = separate(graded(uv), c1, c2).x * density;
    float k2 = separate(graded(uv - shift / RENDERSIZE), c1, c2).y * density * ink2Strength;

    // Each plate is screened in its own coordinates, so its grain moves with it.
    float cell = max(dotSize * sc, 1.0);
    vec2 px2 = px - shift;
    float cov1 = screenPlate(k1, px, radians(45.0), seed, cell) * inkTexture(px, seed, sc);
    float cov2 = screenPlate(k2, px2, radians(15.0), seed + 0.37, cell) * inkTexture(px2, seed + 0.5, sc);

    vec3 paperCol = PAPERS[clamp(paper, 0, 4)];
    float fibre = fbm(px / (5.0 * sc) * vec2(1.0, 0.3)) - 0.5;
    float tooth = hash12(floor(px)) - 0.5;
    paperCol *= 1.0 + paperTexture * (0.10 * fibre + 0.06 * tooth);

    // Riso inks are translucent: each multiplies what's under it.
    vec3 col = paperCol * mix(vec3(1.0), c1, cov1 * 0.95) * mix(vec3(1.0), c2, cov2 * 0.92);
    fragColor = vec4(clamp(col, 0.0, 1.0), 1.0);
}
