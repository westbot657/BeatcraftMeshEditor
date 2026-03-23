// mesh.frag
#version 330 core
flat in int   vCh;
in vec3       vN;
in vec4       vColorAlpha;
uniform int   uWire;
out vec4 fragColor;

const vec3 LIGHT = normalize(vec3(0.6, 1.0, 0.4));
const vec3 COLORS[8] = vec3[8](
    vec3(0.55,0.70,1.00), vec3(1.00,0.25,0.35),
    vec3(0.15,0.95,0.45), vec3(1.00,0.90,0.10),
    vec3(0.20,0.50,1.00), vec3(0.90,0.20,1.00),
    vec3(0.10,0.95,0.95), vec3(1.00,0.55,0.10)
);

void main() {
    if (uWire != 0) { fragColor = vec4(vec3(0.4), 0.4); return; }
    vec3 N = normalize(vN);
    float diff = max(dot(N, LIGHT), 0.0) * 0.6 + 0.4;
    vec3 base  = (vCh > 7) ? vColorAlpha.rgb : COLORS[vCh & 7];
    fragColor  = vec4(base * diff, vColorAlpha.a);
}
