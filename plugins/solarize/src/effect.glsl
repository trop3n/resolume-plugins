// Solarize: tones past the threshold fold back toward black (the Sabattier
// effect). Rainbow runs the folded tone through a hue gradient.

// 0 -> 0, threshold -> 1, 1 -> 0.
float fold(float v) {
    return v <= threshold ? v / threshold : (1.0 - v) / (1.0 - threshold);
}

void main() {
    vec4 src = IMG_INPUT(uv);
    vec3 c = src.rgb;
    float l = luma(c);
    vec3 col;
    if (palette == 0) {
        // Rainbow: dark tones blue/violet, the fold peak red.
        float t = mix(l, fold(l), amount);
        float h = fract((1.0 - t) * 0.75 * cycles + hueShift + TIME * speed);
        col = hsv2rgb(vec3(h, saturation, 0.75 + 0.25 * l));
    } else if (palette == 1) {
        // Colour: each channel folds on its own.
        col = mix(c, vec3(fold(c.r), fold(c.g), fold(c.b)), amount);
    } else {
        col = vec3(mix(l, fold(l), amount));
    }
    fragColor = vec4(mix(src.rgb, col, mixAmount), src.a);
}
