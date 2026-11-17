// Pass 1: odd-even transposition sort along each line. Neighbouring pixels
// pair up (pairs alternate each step) and swap if out of order. A span of n
// pixels is fully sorted after n steps; fewer leave the streaky half-sorted
// look.
//
// Each run of this pass does STEPS steps in registers on a window of
// 2 * STEPS + 1 pixels around this one, so the frame goes through memory
// once per STEPS steps instead of once per step. Pixels at the window's
// ends have no partner outside it, so after each step one more pixel at
// each end may be wrong; after STEPS steps the centre is still exact, and
// that is what this pixel becomes. All array indices are constants so the
// window stays in registers.

const int STEPS = 4;

vec4 v[9];
float k[9];
bool ascending;

// Compare-exchange window slots a < b (adjacent).
void cs(int a, int b) {
    if (v[a].a < 0.0 || v[b].a < 0.0) return;   // masked pixels never move
    if (ascending ? k[a] > k[b] : k[a] < k[b]) {
        vec4 tv = v[a]; v[a] = v[b]; v[b] = tv;
        float tk = k[a]; k[a] = k[b]; k[b] = tk;
    }
}

void main() {
    ivec2 p = pixelCoord();
    vec4 centre = FETCH_PASS(p);
    fragColor = centre;
    if (centre.a < 0.0) return;

    bool vertical = direction < 2;
    int i = vertical ? p.y : p.x;
    int n = vertical ? int(RENDERSIZE.y) : int(RENDERSIZE.x);
    ivec2 axis = vertical ? ivec2(0, 1) : ivec2(1, 0);

    for (int o = 0; o < 9; o++) {
        int j = i + o - STEPS;
        // Off the edge counts as masked, so nothing swaps with it.
        v[o] = (j < 0 || j >= n) ? vec4(0.0, 0.0, 0.0, -1.0) : FETCH_PASS(p + axis * (o - STEPS));
        k[o] = sortKey(v[o].rgb);
    }

    // Up/Right: larger keys move to higher indices (up the screen, right).
    // Down/Left: larger keys move to lower indices.
    ascending = direction == 1 || direction == 2;
    int first = ITERATION * STEPS;
    for (int s = 0; s < STEPS; s++) {
        if (first + s >= iterations) break;
        // Line position x pairs with x + 1 when x + step is even. Slot o is
        // position i + o - STEPS, so pairs start at slot 0 or 1.
        if (((i - STEPS + first + s) & 1) == 0) {
            cs(0, 1); cs(2, 3); cs(4, 5); cs(6, 7);
        } else {
            cs(1, 2); cs(3, 4); cs(5, 6); cs(7, 8);
        }
    }
    fragColor = v[STEPS];
}
