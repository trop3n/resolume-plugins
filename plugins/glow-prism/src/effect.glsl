// Glow Prism: ported from prism-ffgl's GlowPrism.fs (ISF) to ffgl-fx.
// Parameters keep their ISF names as uniforms; see lib.rs.

// Port of Prism Lab's Blender Glow (Glare node, Bloom type).
// Pass 0: keep pixels above Threshold and blur them horizontally.
// Pass 1: blur that vertically.
// Pass 2: add the glow on top of the video.
// Each pass reads the one before through passIn (IMG_PASS). Every pass runs
// at full resolution, so the blur spreads TAPS samples over the radius
// instead of working on a smaller buffer.

#define TAPS 16

// The part of a color brighter than Threshold, with a soft knee.
vec3 brightPart(vec3 c) {
    float l = luma(c);
    float k = max(Softness, 1e-3);
    float w = smoothstep(Threshold - k, Threshold + k, l);
    return c * w;
}

float gauss(float x) {
    return exp(-x * x * 4.5);
}

void main() {
    // Blur radius: Size 1 = 15% of frame height.
    float radius = max(Size * 0.15, 0.5 / RENDERSIZE.y);

    if (PASSINDEX == 0) {
        vec2 stepUV = vec2(radius * RENDERSIZE.y / RENDERSIZE.x, 0.0) / float(TAPS);
        vec3 sum = vec3(0.0);
        float norm = 0.0;
        for (int i = -TAPS; i <= TAPS; i++) {
            float w = gauss(float(i) / float(TAPS));
            sum += brightPart(IMG_INPUT(clamp(uv + stepUV * float(i), 0.0, 1.0)).rgb) * w;
            norm += w;
        }
        fragColor = vec4(sum / norm, 1.0);
    } else if (PASSINDEX == 1) {
        vec2 stepUV = vec2(0.0, radius) / float(TAPS);
        vec3 sum = vec3(0.0);
        float norm = 0.0;
        for (int i = -TAPS; i <= TAPS; i++) {
            float w = gauss(float(i) / float(TAPS));
            sum += IMG_PASS(clamp(uv + stepUV * float(i), 0.0, 1.0)).rgb * w;
            norm += w;
        }
        fragColor = vec4(sum / norm, 1.0);
    } else {
        vec4 original = IMG_INPUT(uv);
        vec3 glow = IMG_PASS(uv).rgb * Tint.rgb * Strength;
        // Screen-style add, so strong glow saturates to white instead of clipping harshly.
        vec3 lit = 1.0 - (1.0 - original.rgb) * exp(-glow);
        fragColor = mix(original, vec4(lit, original.a), Mix);
    }
}
