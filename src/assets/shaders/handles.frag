#version 330 core

flat in int vCh;
in vec2 vUV;
in vec4 vColorAlpha;

uniform int uWire;
uniform sampler2D uNoise;

out vec4 fragColor;

const vec3 COLORS[4] = vec3[4](
    vec3(1.0, 1.0, 1.0),
    vec3(1.0, 0.0, 0.0),
    vec3(0.0, 1.0, 0.0),
    vec3(0.0, 0.0, 1.0)
);

void main() {
    float dist = distance(vUV, vec2(0.0));
    if (dist > 1.0 || dist < 0.9) {
        discard;
    }
    fragColor = vColorAlpha * vec4(COLORS[vCh & 3], 0.5);
}


