// Mosaic Tiles: a grid of tiles that keep flipping between the image and a
// solid colour, with grout between them. Each tile flips on its own
// schedule, so the pattern shuffles without ever changing all at once.

void main() {
    vec4 src = IMG_INPUT(uv);
    float tw = RENDERSIZE.x / float(columns);
    vec2 px = uv * RENDERSIZE;
    vec2 tile = floor(px / tw);
    vec2 f = px / tw - tile;

    // Stagger each tile's flip times so they don't all change together.
    float t = TIME * flipSpeed + hash12(tile + 3.7) * 10.0;
    float epoch = floor(t);
    bool solid = hash12(tile + epoch * 1.731 + seed) < solidAmount;

    vec3 c = src.rgb;
    if (imageStyle == 1) {
        c = vec3(smoothstep(level - 0.02, level + 0.02, luma(c)));
    } else if (imageStyle == 2) {
        c = vec3(luma(c));
    }
    vec3 col = solid ? tileColour : c;

    // Grout lines between tiles.
    float g = grout * 0.5;
    vec2 inside = step(vec2(g), f) * step(f, vec2(1.0 - g));
    col = mix(groutColour, col, inside.x * inside.y);
    fragColor = vec4(mix(src.rgb, col, mixAmount), src.a);
}
