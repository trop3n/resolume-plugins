// Pass 2 (stored in "track"): the tracker's state, kept in two texels.
//   (0, 0): box centre and half-size, in uv units
//   (1, 0): x = confidence 0..1, y = 1 once the tracker has a box,
//           z = top of the strong motion (uv y), for the head box
// Only those two pixels do any work; the rest of the buffer is unused.
//
// The box is the motion energy's weighted centre plus two standard
// deviations each way (robust to stray specks), smoothed over time.

void main() {
    ivec2 p = pixelCoord();
    if (p.y != 0 || p.x > 1) {
        fragColor = vec4(0.0);
        return;
    }
    vec4 box = texelFetch(track, ivec2(0, 0), 0);
    vec4 info = texelFetch(track, ivec2(1, 0), 0);
    bool had = info.y > 0.5;

    // Weighted mean and spread of the energy on a 48x48 grid.
    float sum = 0.0;
    vec2 mean = vec2(0.0);
    vec2 sq = vec2(0.0);
    for (int y = 0; y < 48; y++) {
        for (int x = 0; x < 48; x++) {
            vec2 q = (vec2(x, y) + 0.5) / 48.0;
            float w = texture(energy, q).r;
            w *= w;   // favour strong motion over faint flicker
            sum += w;
            mean += w * q;
            sq += w * q * q;
        }
    }

    float conf = info.x;
    bool found = sum > 2.0;
    if (found) {
        mean /= sum;
        vec2 sd = sqrt(max(sq / sum - mean * mean, vec2(0.0)));

        // Second look, counting only motion near the first estimate, so a
        // few far-off specks (a clock, flicker) don't stretch the box.
        vec2 sigma = max(sd * 1.5, vec2(0.02));
        float sum2 = 0.0;
        vec2 mean2 = vec2(0.0);
        vec2 sq2 = vec2(0.0);
        float top = 0.0;
        for (int y = 0; y < 48; y++) {
            for (int x = 0; x < 48; x++) {
                vec2 q = (vec2(x, y) + 0.5) / 48.0;
                float w = texture(energy, q).r;
                vec2 z = (q - mean) / sigma;
                w *= w * exp(-0.5 * dot(z, z));
                sum2 += w;
                mean2 += w * q;
                sq2 += w * q * q;
                if (w > 0.15) top = max(top, q.y);
            }
        }
        if (sum2 > 0.5) {
            mean = mean2 / sum2;
            sd = sqrt(max(sq2 / sum2 - mean * mean, vec2(0.0)));
        }
        vec2 halfSize = clamp(sd * 2.0 + padding, vec2(0.03), vec2(0.5));
        // Don't let the box reach far above the highest strong motion.
        if (top > 0.0) {
            float boxTop = min(mean.y + halfSize.y, top + 1.0 / 48.0 + padding);
            float boxBottom = mean.y - halfSize.y;
            mean.y = (boxTop + boxBottom) * 0.5;
            halfSize.y = max((boxTop - boxBottom) * 0.5, 0.03);
        } else {
            top = mean.y + halfSize.y;
        }
        vec4 measured = vec4(mean, halfSize);
        float k = had ? 1.0 - smoothing * 0.95 : 1.0;
        box = mix(box, measured, k);
        info.z = had ? mix(info.z, top, k) : top;
        conf = min(conf + 0.15, 1.0);
    } else {
        // Lost: keep the last box and fade it out.
        conf = max(conf - 0.03, 0.0);
    }
    bool has = had || found;
    fragColor = p.x == 0 ? box : vec4(conf, has ? 1.0 : 0.0, info.z, 1.0);
}
