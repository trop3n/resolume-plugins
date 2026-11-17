// Fluid Prism: ported from prism-ffgl's FluidPrism.fs (ISF) to ffgl-fx.
// Parameters keep their ISF names as uniforms; see lib.rs.

// Port of the "volumetric fluid distortion" idea: advect the image's sampling
// coordinates with a fluid velocity field, then sample R, G and B at slightly
// different offsets for chromatic dispersion.
//
// `field` (persistent, float) holds, per pixel: rg = offset from this pixel to
// where the video is sampled (uv units), b = smoke density.
// Pass 0 (stored in "field"): step the simulation. It reads last frame's
//         `field`; ffgl-fx double-buffers it, so no copy pass is needed (the ISF
//         version had one).
// Pass 1: draw the video through the offsets.
//
// The velocity is curl noise (divergence-free, so it swirls like a fluid) plus a
// buoyancy push along Direction, and it only acts where there is smoke. This is
// much cheaper than a pressure solve, which would need many full-res passes in ffgl-isf.
//
// The simulation steps a fixed 1/60 s per frame (scaled by Speed), as in the
// ISF version.

#define TAU 6.28318530718
#define STEP (1.0 / 60.0)

float vnoise(vec2 p) {
    vec2 i = floor(p);
    vec2 f = fract(p);
    vec2 u = f * f * (3.0 - 2.0 * f);
    return mix(mix(hash12(i), hash12(i + vec2(1.0, 0.0)), u.x),
               mix(hash12(i + vec2(0.0, 1.0)), hash12(i + vec2(1.0, 1.0)), u.x), u.y);
}

float fbm3(vec2 p) {
    float sum = 0.0;
    float amp = 0.5;
    for (int i = 0; i < 3; i++) {
        sum += amp * vnoise(p);
        p = p * 2.03 + vec2(17.1, 9.7);
        amp *= 0.5;
    }
    return sum;
}

// Bilinear read of the field done by hand, so slow motion (under half a pixel per
// frame) still moves even if the host samples pass buffers without filtering.
vec4 fieldAt(vec2 uv) {
    vec2 t = uv * RENDERSIZE - 0.5;
    vec2 i = floor(t);
    vec2 f = t - i;
    vec2 px = 1.0 / RENDERSIZE;
    vec2 b = (i + 0.5) * px;
    vec4 a00 = texture(field, clamp(b, 0.0, 1.0));
    vec4 a10 = texture(field, clamp(b + vec2(px.x, 0.0), 0.0, 1.0));
    vec4 a01 = texture(field, clamp(b + vec2(0.0, px.y), 0.0, 1.0));
    vec4 a11 = texture(field, clamp(b + px, 0.0, 1.0));
    return mix(mix(a00, a10, f.x), mix(a01, a11, f.x), f.y);
}

// Curl of a drifting noise potential, in frame-height units per second.
vec2 curlNoise(vec2 p, float t) {
    vec2 q = p * Scale + vec2(0.0, -t * 0.35);
    float e = 0.05;
    float n1 = fbm3(q + vec2(0.0, e) + t * 0.1);
    float n2 = fbm3(q - vec2(0.0, e) + t * 0.1);
    float n3 = fbm3(q + vec2(e, 0.0) + t * 0.1);
    float n4 = fbm3(q - vec2(e, 0.0) + t * 0.1);
    return vec2(n1 - n2, n4 - n3) / (2.0 * e) / Scale;
}

void main() {
    float aspect = RENDERSIZE.x / RENDERSIZE.y;
    vec2 toUV = vec2(1.0 / aspect, 1.0);

    if (PASSINDEX == 0) {
        // Start from a still field on the first frames.
        if (FRAMEINDEX < 2) {
            fragColor = vec4(0.0);
            return;
        }
        float dt = STEP * Speed;
        vec2 p = (uv - 0.5) * vec2(aspect, 1.0);

        float a = radians(Direction);
        vec2 vel = curlNoise(p, TIME * Speed) * Turbulence + vec2(cos(a), sin(a)) * Buoyancy;
        vec2 velUV = vel * toUV;

        // Semi-Lagrangian step: fetch what flowed into this pixel. The flow moves
        // everywhere, but the picture only gets pushed around where there is smoke.
        vec4 prev = fieldAt(uv - velUV * dt);
        float dens = prev.b;
        vec2 off = prev.rg - velUV * dt * smoothstep(0.0, 0.25, dens) * Strength;

        // Emitter: a soft disc at Center feeds smoke in.
        vec2 c = (Center - 0.5) * vec2(aspect, 1.0);
        float r = max(Radius, 1e-3);
        float e = 1.0 - smoothstep(r * 0.5, r, length(p - c));
        dens = clamp(dens * (1.0 - Fade * dt) + e * Emit * dt * 6.0, 0.0, 1.0);

        // Let the picture settle back where the smoke has gone.
        off *= 1.0 - Recover * dt;
        float m = length(off);
        if (m > 0.5) off *= 0.5 / m;

        fragColor = vec4(off, dens, 1.0);
    } else {
        vec4 original = IMG_INPUT(uv);
        vec4 f = texture(field, uv);
        vec2 off = f.rg;
        float k = Dispersion * 0.5;
        vec4 g = IMG_INPUT(clamp(uv + off, 0.0, 1.0));
        float rr = IMG_INPUT(clamp(uv + off * (1.0 - k), 0.0, 1.0)).r;
        float bb = IMG_INPUT(clamp(uv + off * (1.0 + k), 0.0, 1.0)).b;
        vec3 col = vec3(rr, g.g, bb);
        // Haze: a pale veil where the smoke is.
        col += (1.0 - col) * f.b * Haze;
        fragColor = mix(original, vec4(col, original.a), Mix);
    }
}
