// Pass 0 (stored in "background"): the learned empty scene. Learn
// Background copies the clip in (do it with the stage empty); otherwise
// the background slowly adapts to lighting changes, but only where the
// matte says it's background, so a subject standing still isn't absorbed.
// The first frame seeds it if nothing was learned.
//
// If most of the frame reads as foreground, the shot has probably cut or
// the camera moved, so the whole background re-learns quickly, and keeps
// going until little foreground is left. Alpha holds that state: 1 is
// normal, 2 is re-learning (every pixel computes the same decision).

void main() {
    vec4 bg = texture(background, uv);
    vec4 cur = IMG_INPUT(uv);
    if (learnBackground || bg.a < 0.5) {
        fragColor = vec4(cur.rgb, 1.0);
        return;
    }
    // Foreground coverage, from last frame's matte on a sparse 8x8 grid.
    float coverage = 0.0;
    for (int y = 0; y < 8; y++) {
        for (int x = 0; x < 8; x++) {
            coverage += texture(matte, (vec2(x, y) + 0.5) / 8.0).r;
        }
    }
    coverage /= 64.0;

    bool relearning = bg.a > 1.5;
    relearning = autoRelearn && (relearning ? coverage > 0.1 : coverage > 0.6);

    float m = texture(matte, uv).r;
    float rate = relearning ? 0.3 : adapt * 0.02 * (1.0 - m);
    fragColor = vec4(mix(bg.rgb, cur.rgb, rate), relearning ? 2.0 : 1.0);
}
