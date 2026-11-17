// Glass Prism: ported from prism-ffgl's GlassPrism.fs (ISF) to ffgl-fx.
// Parameters keep their ISF names as uniforms; see lib.rs.

// Port of Prism Lab's Blender "Patterned Glass". Each Pattern is a height field;
// its slope bends where the video is sampled, like light through textured glass.

#define TAU 6.28318530718

float vnoise(vec2 p) {
    vec2 i = floor(p);
    vec2 f = fract(p);
    vec2 u = f * f * (3.0 - 2.0 * f);
    return mix(mix(hash12(i), hash12(i + vec2(1.0, 0.0)), u.x),
               mix(hash12(i + vec2(0.0, 1.0)), hash12(i + vec2(1.0, 1.0)), u.x), u.y);
}

float fbm(vec2 p, float octaves) {
    float sum = 0.0;
    float amp = 0.5;
    float norm = 0.0;
    for (int i = 0; i < 6; i++) {
        if (float(i) >= octaves) break;
        sum += amp * vnoise(p);
        norm += amp;
        p = p * 2.03 + vec2(17.1, 9.7);
        amp *= 0.55;
    }
    return sum / norm;
}

// Distance to the nearest random feature point (Voronoi F1).
float voronoi(vec2 q, float randomness) {
    vec2 cell = floor(q);
    float d = 8.0;
    for (int j = -1; j <= 1; j++) {
        for (int i = -1; i <= 1; i++) {
            vec2 c = cell + vec2(float(i), float(j));
            vec2 pt = c + mix(vec2(0.5), hash22(c), randomness);
            d = min(d, length(q - pt));
        }
    }
    return d;
}

// Smooth Voronoi: soft cell borders, for hammered glass.
float voronoiSmooth(vec2 q) {
    vec2 cell = floor(q);
    float res = 0.0;
    for (int j = -1; j <= 1; j++) {
        for (int i = -1; i <= 1; i++) {
            vec2 c = cell + vec2(float(i), float(j));
            vec2 pt = c + hash22(c);
            res += exp(-6.0 * length(q - pt));
        }
    }
    return -log(res) / 6.0;
}

// Rounded rib profile: 1 at the middle of each cell, 0 at its edges.
float semicircle(float t) {
    float c = fract(t) * 2.0 - 1.0;
    return sqrt(max(1.0 - c * c, 0.0));
}

vec2 rotate2(vec2 v, float a) {
    float s = sin(a);
    float c = cos(a);
    return vec2(c * v.x - s * v.y, s * v.x + c * v.y);
}

// Pattern height at p (frame-height units, centered, aspect-corrected).
float patternHeight(vec2 p) {
    vec2 q = rotate2(p, radians(Rotation)) * Scale;
    if (Pattern == 0) return semicircle(q.x);
    if (Pattern == 1) return 1.0 - semicircle(q.x);
    if (Pattern == 2) return fbm(q * 3.0, 5.0);
    if (Pattern == 3) {
        vec2 w = q * 0.12;
        w += vec2(fbm(w + 3.1, 2.0), fbm(w + 7.7, 2.0)) * 1.2;
        return fbm(w, 3.0);
    }
    if (Pattern == 4) {
        float r = voronoi(q, 0.0) / 0.48;
        return sqrt(max(1.0 - r * r, 0.0));
    }
    if (Pattern == 5) {
        float wob = sin(q.y * TAU * 0.35);
        return 0.5 + 0.5 * sin(q.x * TAU + wob * 1.5);
    }
    if (Pattern == 6) return pow(voronoiSmooth(q) + 0.15, 0.8);
    return semicircle(q.x) * semicircle(q.y);
}

// How strongly each Pattern bends light at the same Depth (keeps presets comparable).
float patternStrength() {
    if (Pattern == 2) return 0.5;
    if (Pattern == 3) return 5.0;
    if (Pattern == 5) return 0.8;
    if (Pattern == 6) return 0.6;
    return 1.0;
}

float animLayer(vec2 p) {
    if (Animate == 0) return 0.0;
    float t = TIME * AnimSpeed * TAU * 0.5;
    float aspect = RENDERSIZE.x / RENDERSIZE.y;
    float dist;
    if (Animate == 1) {
        vec2 c = (Center - 0.5) * vec2(aspect, 1.0);
        dist = length(p - c);
    } else {
        float a = radians(AnimDir);
        dist = dot(p, vec2(cos(a), sin(a)));
    }
    return sin(dist * AnimFreq * TAU - t) * AnimAmount;
}

float heightAt(vec2 p) {
    return patternHeight(p) + animLayer(p);
}

vec4 sampleSplit(vec2 uv, vec2 off) {
    // Dispersion: red bends less, blue bends more.
    float d = Dispersion * 0.6;
    vec4 g = IMG_INPUT(clamp(uv + off, 0.0, 1.0));
    float r = IMG_INPUT(clamp(uv + off * (1.0 - d), 0.0, 1.0)).r;
    float b = IMG_INPUT(clamp(uv + off * (1.0 + d), 0.0, 1.0)).b;
    return vec4(r, g.g, b, g.a);
}

void main() {
    vec4 original = IMG_INPUT(uv);
    float aspect = RENDERSIZE.x / RENDERSIZE.y;
    vec2 p = (uv - 0.5) * vec2(aspect, 1.0);

    // Slope of the glass surface, measured per Pattern cell.
    float e = 1.0 / RENDERSIZE.y;
    vec2 g = vec2(heightAt(p + vec2(e, 0.0)) - heightAt(p - vec2(e, 0.0)),
                  heightAt(p + vec2(0.0, e)) - heightAt(p - vec2(0.0, e))) / (2.0 * e);
    vec2 slope = g / Scale * patternStrength();

    // Refraction: shift the lookup against the slope (up to ~1.5 cells at full Depth).
    vec2 offP = -slope * Depth * 1.5 / Scale;
    vec2 off = offP * vec2(1.0 / aspect, 1.0);

    vec4 col;
    if (Frost > 0.001) {
        // Frost: average jittered lookups in a small spiral.
        col = vec4(0.0);
        float radius = Frost * 0.025;
        float jitter = hash12(gl_FragCoord.xy) * TAU;
        for (int i = 0; i < 12; i++) {
            float fi = float(i);
            float a = fi * 2.39996 + jitter;
            vec2 o = vec2(cos(a), sin(a)) * radius * sqrt((fi + 0.5) / 12.0);
            col += sampleSplit(uv, off + o * vec2(1.0 / aspect, 1.0));
        }
        col /= 12.0;
    } else {
        col = sampleSplit(uv, off);
    }

    // Sheen: a soft highlight where the surface faces a light up and to the left.
    vec3 n = normalize(vec3(-slope * Depth * 2.0, 1.0));
    vec3 l = normalize(vec3(-0.45, 0.6, 1.0));
    float spec = pow(max(dot(reflect(-l, n), vec3(0.0, 0.0, 1.0)), 0.0), 24.0);
    col.rgb = col.rgb * Tint.rgb + spec * Sheen;

    fragColor = mix(original, vec4(col.rgb, col.a), Mix);
}
