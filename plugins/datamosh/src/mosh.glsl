// Pass 1 (stored in "mosh"): decode a P-frame onto the moshed picture.
// Last frame's mosh moves by this frame's motion, then gets the residual an
// encoder would have sent: this frame minus the moved previous input.
// Because the reference is the mosh, not the real previous frame, old
// content rides along with the new motion. That is datamoshing.
//
// Alpha holds 1 + the auto-keyframe epoch, so a new epoch (or an empty
// buffer, alpha 0) is noticed and the clip shows through as a keyframe.

void main() {
    vec2 v = texture(motion, uv).xy * motionGain;
    vec2 from = uv - v / RENDERSIZE;
    vec4 cur = IMG_INPUT(uv);
    vec4 last = texture(mosh, from);
    vec3 residual = cur.rgb - texture(prevInput, from).rgb;
    // A real encoder would start a new scene (or a big change) with a
    // keyframe, and that keyframe is what moshing deletes. So big changes
    // carry no residual, and the old picture survives them.
    float change = length(residual);
    residual *= 1.0 - smoothstep(cutThreshold, cutThreshold + 0.25, change);
    vec3 col = last.rgb + residual * residualAmount;

    float epoch = keyInterval > 0.0 ? mod(floor(TIME / keyInterval), 512.0) : 0.0;
    bool key = keyframe || last.a < 0.5 || abs(last.a - 1.0 - epoch) > 0.5;

    // I-blocks: a few random blocks refresh from the clip each frame.
    float bs = max(blockSize * sizeScale(), 4.0);
    vec2 block = floor(uv * RENDERSIZE / bs);
    bool refreshBlock = hash12(block + float(FRAMEINDEX % 997) * 0.731) < refresh * 0.1;

    if (key || refreshBlock) col = cur.rgb;
    fragColor = vec4(clamp(col, 0.0, 1.0), 1.0 + epoch);
}
