// Pass 2: a depth value per pixel, 1 = near. The scene gets a fake depth
// (Floor: lower in frame is nearer; Brightness; or Flat), squeezed between
// Far and part of the way to Near. Foreground from the matte is pushed to
// Near, with a little relief from its own shading.

void main() {
    vec3 c = IMG_INPUT(uv).rgb;
    float m = texture(matte, uv).r;
    float scene = sceneMode == 0 ? 1.0 - uv.y
                : sceneMode == 1 ? luma(c)
                : 0.5;
    float sceneDepth = farLevel + sceneAmount * (nearLevel - farLevel) * scene;
    float subject = nearLevel + relief * (luma(c) - 0.5) * 0.25;
    float depth = mix(sceneDepth, subject, m);
    fragColor = vec4(clamp(depth, 0.0, 1.0), 0.0, 0.0, 1.0);
}
