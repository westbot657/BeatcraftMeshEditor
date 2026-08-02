#version 450 core

layout(points) in;
layout(triangle_strip, max_vertices = 28) out;

flat in int v_beat_number[];
flat in float v_x0[];

uniform float u_beat_spacing;
uniform mat4 u_view_proj;
uniform float u_track_width;
uniform vec2 u_digit_offset;
uniform vec2 u_digit_size;

out vec2 v_uv;

void emit_quad(float x0, float x1, float z0, float z1, vec4 uv) {
    gl_Position = u_view_proj * vec4(x0, 0.0, z0, 1.0);  v_uv = uv.xy;  EmitVertex();
    gl_Position = u_view_proj * vec4(x0, 0.0, z1, 1.0);  v_uv = uv.xw;  EmitVertex();
    gl_Position = u_view_proj * vec4(x1, 0.0, z0, 1.0);  v_uv = uv.zy;  EmitVertex();
    gl_Position = u_view_proj * vec4(x1, 0.0, z1, 1.0);  v_uv = uv.zw;  EmitVertex();
    EndPrimitive();
}

void main() {
    int un = v_beat_number[0];
    if (un < 0) {
        return;
    }
    uint n = uint(un);
    float z0 = v_x0[0];
    float z1 = z0 + u_beat_spacing;

    emit_quad(
        u_track_width,
        -u_track_width,
        z0, z1,
        vec4(-2.0, -2.0, -1.0, -1.0)
    );

    uint digits[5];
    int count = 0;
    if (n == 0u) { digits[count++] = 0u; }
    while (n > 0u && count < 5) { digits[count++] = n % 10u; n /= 10u; }

    for (int i = 0; i < count; i++) {
        int column = count - 1 - i;
        float dx0 = -u_track_width - u_digit_offset.x - float(column) * u_digit_size.x;
        float dx1 = dx0 - u_digit_size.x;
        float dz0 = z0 + u_digit_offset.y;
        float dz1 = dz0 + u_digit_size.y;
        float u0 = float(digits[i]) / 10.0;
        emit_quad(dx0, dx1, dz1, dz0, vec4(u0, 0.0, u0 + (1.0/11.0), 1.0));

    }

}

