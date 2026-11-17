// Shared by the Pixel Sort passes.
//
// Between passes each pixel is stored as (rgb, a) where a >= 0 means
// "sortable, with alpha a" and a < 0 means "masked out, with alpha -a - 1".
// Masked pixels never move, so they split each line into separate spans.

float hash12(vec2 p) {
    vec3 p3 = fract(vec3(p.xyx) * 0.1031);
    p3 += dot(p3, p3.yzx + 33.33);
    return fract((p3.x + p3.y) * p3.z);
}

float luma(vec3 c) {
    return dot(c, vec3(0.2126, 0.7152, 0.0722));
}

// Hue and saturation via the usual branchless RGB -> HSV.
vec3 rgb2hsv(vec3 c) {
    vec4 K = vec4(0.0, -1.0 / 3.0, 2.0 / 3.0, -1.0);
    vec4 p = mix(vec4(c.bg, K.wz), vec4(c.gb, K.xy), step(c.b, c.g));
    vec4 q = mix(vec4(p.xyw, c.r), vec4(c.r, p.yzx), step(p.x, c.r));
    float d = q.x - min(q.w, q.y);
    return vec3(abs(q.z + (q.w - q.y) / (6.0 * d + 1e-10)), d / (q.x + 1e-10), q.x);
}

float sortKey(vec3 c) {
    if (sortBy == 1) return rgb2hsv(c).x;
    if (sortBy == 2) return rgb2hsv(c).y;
    return luma(c);
}
