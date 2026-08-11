#version 450 core

uniform mat4 u_view_proj;
uniform float u_rotation;
uniform float u_z;

out vec2 v_uv;

vec3 rotateY(vec3 v, float angle) {
    float s = sin(angle);
    float c = cos(angle);
    return vec3(
        v.x * c + v.z * s,
        v.y,
        -v.x * s + v.z * c
    );
}

void main() {
    float u = float(gl_VertexID >> 1);
    float v = float(gl_VertexID & 1);
    v_uv = vec2(u, v);

    float x = 2.0 - (u * 4.0);
    float y = (v * 3.0);

    vec3 pos = vec3(x * 0.6, y * 0.6, u_z);

    vec3 rotated = rotateY(pos, u_rotation);

    gl_Position = u_view_proj * vec4(rotated, 1.0);
}

