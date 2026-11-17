// Shared by the Pixel Sort passes.
//
// Between passes each pixel is stored as (rgb, a) where a >= 0 means
// "sortable, with alpha a" and a < 0 means "masked out, with alpha -a - 1".
// Masked pixels never move, so they split each line into separate spans.

float sortKey(vec3 c) {
    if (sortBy == 1) return rgb2hsv(c).x;
    if (sortBy == 2) return rgb2hsv(c).y;
    return luma(c);
}
