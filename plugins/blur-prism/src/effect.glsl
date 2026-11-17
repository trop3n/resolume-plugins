// Blur Prism: ported from prism-ffgl's BlurPrism.fs (ISF) to ffgl-fx.
// Parameters keep their ISF names as uniforms; see lib.rs.

// Port of Prism Lab's Blender "Optic Blur" region. One region per effect instance;
// stack the effect in Resolume for more. The Selected Mesh mode becomes a key
// (luma, alpha or color), since a video frame has no object pass.

#define TAU 6.28318530718
#define MAX_SAMPLES 64

vec2 rotate2(vec2 v, float a) {
    float s = sin(a);
    float c = cos(a);
    return vec2(c * v.x - s * v.y, s * v.x + c * v.y);
}

// Area mask at uv: 1 inside the box/ellipse, feathered over Softness (frame heights).
float areaMask(vec2 uv) {
    float aspect = RENDERSIZE.x / RENDERSIZE.y;
    vec2 q = rotate2((uv - Center) * vec2(aspect, 1.0), -radians(Rotation));
    vec2 hs = max(vec2(Width * aspect, Height) * 0.5, vec2(1e-4));
    float sd;
    if (Shape == 0) {
        vec2 d = abs(q) - hs;
        sd = length(max(d, 0.0)) + min(max(d.x, d.y), 0.0);
    } else {
        sd = (length(q / hs) - 1.0) * min(hs.x, hs.y);
    }
    float s = max(Softness, 1.0 / RENDERSIZE.y);
    return 1.0 - smoothstep(-0.5 * s, 0.5 * s, sd);
}

// Key mask from a video color.
float keyMask(vec4 c) {
    float v;
    float s = max(Softness, 1e-3);
    if (Key == 2) {
        // Color: 1 where the pixel is within Threshold of KeyColor.
        v = 1.0 - smoothstep(Threshold, Threshold + s, distance(c.rgb, KeyColor.rgb));
    } else {
        float x = Key == 0 ? dot(c.rgb, vec3(0.2126, 0.7152, 0.0722)) : c.a;
        v = smoothstep(Threshold - 0.5 * s, Threshold + 0.5 * s, x);
    }
    return Invert ? 1.0 - v : v;
}

float maskAt(vec2 uv, vec4 c) {
    return Mode == 0 ? areaMask(uv) : keyMask(c);
}

void main() {
    vec4 original = IMG_INPUT(uv);
    float aspect = RENDERSIZE.x / RENDERSIZE.y;

    float len = Length;
    float dir = radians(Direction);
    if (Animate) {
        len *= 1.0 - PulseDepth * (0.5 - 0.5 * cos(TAU * PulseSpeed * TIME));
        dir += TAU * Spin * TIME;
    }
    // Streak in uv: Length 1 = 5% of the frame width. Sampling ahead along Direction
    // makes the streak trail behind the motion.
    vec2 streak = vec2(cos(dir), sin(dir) * aspect) * len * 0.05;

    int n = int(clamp(Quality, 2.0, float(MAX_SAMPLES)));
    vec4 blurred = vec4(0.0);
    float maskTrail = 0.0;
    for (int i = 0; i < MAX_SAMPLES; i++) {
        if (i >= n) break;
        vec2 suv = clamp(uv + streak * (float(i) / float(n - 1)), 0.0, 1.0);
        vec4 c = IMG_INPUT(suv);
        blurred += c;
        maskTrail += maskAt(suv, c);
    }
    blurred /= float(n);
    maskTrail /= float(n);

    // Trail: use the streaked mask too, so the blur smears past the region's edge.
    float mask = maskAt(uv, original);
    if (Trail) mask = max(mask, maskTrail);
    float f = clamp(mask * Amount, 0.0, 1.0);

    fragColor = mix(original, mix(original, blurred, f), Mix);
}
