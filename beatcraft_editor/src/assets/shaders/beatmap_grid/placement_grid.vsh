#version 450 core

uniform mat4 u_view_proj;
uniform float u_z;

out vec2 v_uv;

void main() {
    float u = float(gl_VertexID >> 1);
    float v = float(gl_VertexID & 1);
    v_uv = vec2(u, v);

    float x = 2.0 - (u * 4.0);
    float y = (v * 3.0);

    gl_Position = u_view_proj * vec4(x * 0.6, y * 0.6, u_z, 1.0);
}

