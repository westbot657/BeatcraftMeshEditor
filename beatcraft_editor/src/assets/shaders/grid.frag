#version 450 core

in vec2 v_uv;
in vec3 v_world_pos;
in float v_camera_height;

out vec4 fragColor;

float log10(float x) {
    return log(x) / log(10.0);
}

float gridLineAA(vec2 worldXZ, float base, float pixelWidth) {
    vec2 coord = worldXZ / base;
    vec2 deriv = fwidth(coord) + 1e-7;
    vec2 d = abs(fract(coord + 0.5) - 0.5) / deriv;
    float line = min(d.x, d.y);
    return 1.0 - smoothstep(0.0, pixelWidth, line);
}

void main() {
    const float pixelWidth = 1.25;

    float horizDist = length(v_uv);
    float dist = length(vec2(horizDist, v_camera_height));

    float decade = log10(max(dist, 0.0001));
    float decadeFloor = floor(decade);
    float base = pow(10.0, decadeFloor);
    float blend = smoothstep(0.6, 0.999, fract(decade));

    // Detect quads that straddle the floor(decade) discontinuity — fwidth(decadeFloor)
    // is ~0 everywhere except right at the transition ring, where it spikes.
    // Suppress the grid there instead of trusting an ill-defined derivative.
    float seam = fwidth(decadeFloor);
    float seamMask = 1.0 - step(0.5, seam);

    vec2 worldXZ = v_world_pos.xz;
    float lineFine = gridLineAA(worldXZ, base, pixelWidth);
    float lineCoarse = gridLineAA(worldXZ, base * 10.0, pixelWidth);
    float alpha = mix(lineFine, lineCoarse, blend) * seamMask;

    fragColor = vec4(0.35, 0.35, 0.35, alpha);

    float halfSize = v_camera_height * 50.0;
    float fade = 1.0 - smoothstep(halfSize * 0.7, halfSize, horizDist);
    fragColor.a *= fade;

    float axisPixelWidth = pixelWidth * 0.8;
    float axisZ = abs(v_world_pos.z) / (fwidth(v_world_pos.z) + 1e-7);
    float axisX = abs(v_world_pos.x) / (fwidth(v_world_pos.x) + 1e-7);

    float xAxisAlpha = (1.0 - smoothstep(0.0, axisPixelWidth, axisZ)) * step(0.0, v_world_pos.x);
    float zAxisAlpha = (1.0 - smoothstep(0.0, axisPixelWidth, axisX)) * step(0.0, v_world_pos.z);

    if (xAxisAlpha > 0.0) fragColor = vec4(0.9, 0.2, 0.2, max(fade * xAxisAlpha, fragColor.a));
    if (zAxisAlpha > 0.0) fragColor = vec4(0.2, 0.4, 0.9, max(fade * zAxisAlpha, fragColor.a));

    if (fragColor.a <= 0.001) discard;
}
