// Pass 1 (stored in "prevInput"): keep this frame for next frame's motion.

void main() {
    fragColor = vec4(IMG_INPUT(uv).rgb, 1.0);
}
