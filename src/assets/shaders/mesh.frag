// mesh.frag
#version 330 core
flat in int vCh;
in vec3 vN;
in vec4 vColorAlpha;
in float vDepth;
uniform int uWire;
out vec4 fragColor;

const vec3 LIGHT = normalize(vec3(0.6, 1.0, 0.4));
const vec3 COLORS[8] = vec3[8](
    vec3(0.55,0.70,1.00), vec3(1.00,0.25,0.35),
    vec3(0.15,0.95,0.45), vec3(1.00,0.90,0.10),
    vec3(0.20,0.50,1.00), vec3(0.90,0.20,1.00),
    vec3(0.10,0.95,0.95), vec3(1.00,0.55,0.10)
);

const int BAYER[16] = int[16](
     0,  8,  2, 10,
    12,  4, 14,  6,
     3, 11,  1,  9,
    15,  7, 13,  5
);

void main() {
    if (uWire != 0) { fragColor = vec4(vec3(0.4), 0.4); return; }

    int bx = int(mod(gl_FragCoord.x, 4.0));
    int by = int(mod(gl_FragCoord.y, 4.0));
    int bayer = BAYER[by * 4 + bx];

    if (!gl_FrontFacing) {
        // threshold goes from 15 (near, almost fully discarded)
        // down to 1 (far, almost fully kept)
        // clamp depth to [0, 50] then remap to threshold [15, 1]
        float t = 1.0 - clamp(vDepth / 50.0, 0.0, 1.0);
        int threshold = int(mix(15.0, 1.0, t));
        if (bayer >= threshold) discard;
    }

    vec3 N = normalize(vN);
    if (!gl_FrontFacing) N = -N;
    float diff = max(dot(N, LIGHT), 0.0) * 0.6 + 0.4;
    vec3 base  = (vCh > 7) ? vColorAlpha.rgb : COLORS[vCh & 7];
    fragColor  = vec4(base * diff, vColorAlpha.a);
}
