// Ripple Prism: ported from prism-ffgl's RipplePrism.fs (ISF) to ffgl-fx.
// Parameters keep their ISF names as uniforms; see lib.rs.

// Port of Prism Lab's Blender "Surface Ripple". The same wave height field is used,
// but instead of moving mesh vertices its slope shifts where the video is sampled.
// Distances are in frame heights (Wavelength 0.25 = 4 rings per frame height).

#define TAU 6.28318530718

float vnoise(vec2 p) {
    vec2 i = floor(p);
    vec2 f = fract(p);
    vec2 u = f * f * (3.0 - 2.0 * f);
    return mix(mix(hash12(i), hash12(i + vec2(1.0, 0.0)), u.x),
               mix(hash12(i + vec2(0.0, 1.0)), hash12(i + vec2(1.0, 1.0)), u.x), u.y);
}

// Roughly Blender's Noise Texture with Detail 2.
float fbm3(vec2 p) {
    float sum = 0.0;
    float amp = 0.5;
    float norm = 0.0;
    for (int i = 0; i < 3; i++) {
        sum += amp * vnoise(p);
        norm += amp;
        p = p * 2.03 + vec2(17.1, 9.7);
        amp *= 0.5;
    }
    return sum / norm;
}

// Unit-amplitude wave at p (centered, aspect-corrected), same formulas as ripple.py.
float wave(vec2 p) {
    float aspect = RENDERSIZE.x / RENDERSIZE.y;
    vec2 rel = p - (Center - 0.5) * vec2(aspect, 1.0);
    float wobble = (fbm3(p * 1.5 / Wavelength) - 0.5) * Noise * Wavelength;
    float d = length(rel) + wobble;
    float k = TAU / Wavelength;
    float ph = radians(Phase);
    float travel = TIME * Speed;
    float damp = exp(-d * Falloff);

    if (Pattern == 0) return sin((d - travel) * k + ph) * damp;
    if (Pattern == 1) {
        float a = radians(Direction);
        float along = dot(rel, vec2(cos(a), sin(a))) + wobble;
        return sin((along - travel) * k + ph);
    }
    if (Pattern == 2) {
        // One ring whose front moves outward, restarting every Radius.
        float front = mod(travel, Radius);
        float x = (d - front) / (Wavelength * 0.75);
        float fade = max(1.0 - front / Radius, 0.0);
        return sin((d - front) * k + ph) * exp(-x * x) * fade * damp;
    }
    return sin(d * k + ph) * cos(travel * k) * damp;
}

void main() {
    vec4 original = IMG_INPUT(uv);
    float aspect = RENDERSIZE.x / RENDERSIZE.y;
    vec2 p = (uv - 0.5) * vec2(aspect, 1.0);

    // Slope of the wave, normalized so Amplitude is the largest shift (in frame heights)
    // whatever the Wavelength.
    float e = 1.0 / RENDERSIZE.y;
    vec2 g = vec2(wave(p + vec2(e, 0.0)) - wave(p - vec2(e, 0.0)),
                  wave(p + vec2(0.0, e)) - wave(p - vec2(0.0, e))) / (2.0 * e);
    vec2 slope = g * Wavelength / TAU;

    vec2 off = -slope * Amplitude * vec2(1.0 / aspect, 1.0);
    vec4 col = IMG_INPUT(clamp(uv + off, 0.0, 1.0));

    // Shading: wave faces turned toward a light up and to the left get brighter, the others darker.
    float light = dot(-slope, normalize(vec2(-0.45, 0.6)));
    col.rgb *= 1.0 + light * Shading * 0.5;

    fragColor = mix(original, col, Mix);
}
