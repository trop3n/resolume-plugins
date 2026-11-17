// Pass 2: decode the sorted pixels and mix with the original.

void main() {
    ivec2 p = pixelCoord();
    vec4 src = FETCH_INPUT(p);
    vec4 s = FETCH_PASS(p);
    bool sortable = s.a >= 0.0;
    float a = sortable ? s.a : -s.a - 1.0;
    vec3 col = mix(src.rgb, s.rgb, mixAmount);
    if (showMask && sortable) col = mix(col, vec3(1.0, 0.0, 0.0), 0.5);
    fragColor = vec4(col, mix(src.a, a, mixAmount));
}
