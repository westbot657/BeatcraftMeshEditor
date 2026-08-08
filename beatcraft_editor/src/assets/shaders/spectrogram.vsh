#version 450 core

out vec2 v_uv;

void main() {
    float x = float(gl_VertexID >> 1);
    float y = float(gl_VertexID & 1);

    v_uv = vec2(x, y);
    gl_Position = vec4((x * 2.0) - 1.0, (y * 2.0) - 1.0, 0.0, 1.0);
}


