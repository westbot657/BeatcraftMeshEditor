#version 330 core

flat in int vCh;
in vec3 vN;
in vec2 vUV;
in vec4 vColorAlpha;

uniform int uWire;
uniform sampler2D uTexture;
uniform sampler2D uNoise;

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
    if (uWire != 0) {
        fragColor = vec4(vec3(0.4), 0.4);
        return;
    }
    float x = gl_FragCoord.x;
    float y = gl_FragCoord.y;

    int bx = int(mod(x, 4.0));
    int by = int(mod(y, 4.0));
    int bayer = BAYER[by * 4 + bx];

    float noise = texture(uNoise, vec2(x, y) / vec2(textureSize(uNoise, 0))).r;
    float depth = gl_FragCoord.z / gl_FragCoord.w + (noise - 0.5) * 3.5;
    vec4 vColor = vColorAlpha;
    if (!gl_FrontFacing) {
        float t = 1.0 - clamp(depth / 100.0, 0.0, 1.0);
        int threshold = int(mix(15.0, 1.0, t));
        if (bayer >= threshold) discard;
        vColor = vec4(vColor.rgb * vec3(2.0, 2.0, 4.0), vColor.a);
    }

    vec3 N = normalize(vN);
    if (!gl_FrontFacing) N = -N;
    float diff = max(dot(N, LIGHT), 0.0) * 0.2 + 0.8;
    vec3 base3 = (vCh > 7) ? vColor.rgb : COLORS[vCh & 7];
    vec4 base = vec4(base3 * diff, vColor.a);
    if (gl_FrontFacing) {
        base = base * texture(uTexture, vUV);
    }
    fragColor = base;
}
