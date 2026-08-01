#version 450 core

uniform uint u_beat_i;
uniform float u_beat_f;
uniform float u_beat_spacing;
uniform int u_beats_before;

flat out int v_beat_number;
out float v_x0;

void main() {
    int beat_offset = gl_VertexID - u_beats_before;
    v_beat_number = u_beat_i + beat_offset;
    v_x0 = (float(beat_offset) + u_beat_f) * u_beat_spacing;
    gl_Position = vec4(0.0);
}


