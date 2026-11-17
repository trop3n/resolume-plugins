// Pass 0 (stored in "energy"): where things are moving. The difference
// from the previous frame, softened over a few pixels, kept as a decaying
// maximum so a subject that pauses stays tracked for a while (Memory).

float diffAt(vec2 p) {
    return length(IMG_INPUT(p).rgb - texture(prevInput, p).rgb);
}

void main() {
    // Nothing to compare against on the first frame.
    if (texture(prevInput, uv).a < 0.5) {
        fragColor = vec4(0.0, 0.0, 0.0, 1.0);
        return;
    }
    vec2 e = 3.0 * sizeScale() / RENDERSIZE;
    float d = (diffAt(uv) * 2.0 + diffAt(uv + vec2(e.x, 0.0)) + diffAt(uv - vec2(e.x, 0.0))
             + diffAt(uv + vec2(0.0, e.y)) + diffAt(uv - vec2(0.0, e.y))) / 6.0;
    float th = 0.02 + (1.0 - sensitivity) * 0.3;
    float now = smoothstep(th, th + 0.08, d);
    float prev = texture(energy, uv).r;
    fragColor = vec4(max(prev * memory, now), 0.0, 0.0, 1.0);
}
