// Dot Matrix: a square grid of dots whose size follows the image. Print
// style grows dark dots on a light ground; LED style grows lit dots (with
// a halo) on a dark ground.

void main() {
    vec4 src = IMG_INPUT(uv);
    float pitch = max(cellSize * sizeScale(), 3.0);
    vec2 px = uv * RENDERSIZE;
    vec2 cell = floor(px / pitch);
    vec2 f = px / pitch - cell - 0.5;
    vec3 c = IMG_INPUT((cell + 0.5) * pitch / RENDERSIZE).rgb;
    float l = clamp((luma(c) - 0.5) * contrast + 0.5 + brightness, 0.0, 1.0);
    float amount = style == 1 ? l : 1.0 - l;
    float r = mix(minSize, 1.0, amount) * 0.5;
    float d = shape == 1 ? max(abs(f.x), abs(f.y)) : length(f);
    float aa = 1.0 / pitch;
    float cov = 1.0 - smoothstep(r - aa, r + aa, d);
    if (style == 1) {
        cov += glow * amount * exp(-d * d / max(r * r, 1e-4) * 1.5);
    }
    vec3 dotCol = imageColour ? c : dotColour;
    vec3 col = mix(backgroundColour, dotCol, clamp(cov, 0.0, 1.0));
    fragColor = vec4(mix(src.rgb, col, mixAmount), src.a);
}
