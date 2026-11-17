// Pass 0: mark which pixels get sorted.

void main() {
    ivec2 p = pixelCoord();
    vec4 c = FETCH_INPUT(p);
    float l = luma(c.rgb);
    bool sortable = l >= lowThreshold && l <= highThreshold;
    if (invertMask) sortable = !sortable;
    // Random breaks make streaks of varied length.
    if (hash12(vec2(p)) < breakUp * 0.03) sortable = false;
    fragColor = vec4(c.rgb, sortable ? c.a : -c.a - 1.0);
}
