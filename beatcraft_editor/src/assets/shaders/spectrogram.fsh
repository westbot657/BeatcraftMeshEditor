#version 450 core

in vec2 v_uv;

uniform float u_start;
uniform float u_end;
uniform float u_cursor;
uniform float u_coverage;

uniform sampler2D u_texture;

out vec4 fragColor;

vec3 spect_color(float t) {
    t = clamp(t, 0.0, 1.0);
    const vec3 STOPS[8] = vec3[8](
        vec3(0.0,  0.0,  0.0),    // 0.00 - black
        vec3(0.08, 0.0,  0.2),    // 0.14 - deep purple
        vec3(0.35, 0.0,  0.45),   // 0.29 - purple/magenta
        vec3(0.75, 0.0,  0.35),   // 0.43 - magenta-red
        vec3(1.0,  0.1,  0.0),    // 0.57 - red
        vec3(1.0,  0.5,  0.0),    // 0.71 - orange
        vec3(1.0,  0.9,  0.2),    // 0.86 - yellow
        vec3(1.0,  1.0,  0.9)     // 1.00 - near-white
    );
    float scaled = t * 7.0; // 7 segments between 8 stops
    int idx = int(floor(scaled));
    idx = clamp(idx, 0, 6);
    float frac = scaled - float(idx);
    return mix(STOPS[idx], STOPS[idx + 1], frac);
}

void main() {
    float u = mix(u_start, u_end, v_uv.x);

    float t = u / max(u_coverage, 1e-5);

    float mag = texture(u_texture, vec2(v_uv.y, t)).r;
    vec3 tex = spect_color(mag);

    float dist = abs(u_cursor - u);
    float px = fwidth(u);
    float highlight = 1.0 - smoothstep(0.0, px * 3.0, dist);
    fragColor = vec4(tex + vec3(highlight), 1.0);
}
