#version 450 core
in vec2 v_uv;
uniform float u_start;
uniform float u_end;
uniform float u_cursor;
uniform float u_coverage;
uniform sampler2D u_texture;
out vec4 fragColor;
void main() {
    float u = mix(u_start, u_end, v_uv.x);

    float t = u / max(u_coverage, 1e-5);

    vec4 tex = texture(u_texture, vec2(v_uv.y, t));

    float dist = abs(u_cursor - u);
    float px = fwidth(u);
    float highlight = 1.0 - smoothstep(0.0, px * 3.0, dist);
    fragColor = tex + vec4(highlight, highlight, highlight, 0.0);
}
