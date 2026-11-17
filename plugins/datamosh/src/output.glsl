// Pass 3: show the mosh (or, with Show Motion, the block vectors as
// colour: red = right, green = up, grey = still).

void main() {
    vec4 src = IMG_INPUT(uv);
    vec4 m = texture(mosh, uv);
    vec3 col = m.a < 0.5 ? src.rgb : m.rgb;
    if (showMotion) {
        float bs = max(blockSize * sizeScale(), 4.0);
        vec2 v = texture(motion, uv).xy / bs;
        col = vec3(0.5 + 0.5 * v, 0.5);
    }
    fragColor = vec4(mix(src.rgb, col, mixAmount), src.a);
}
