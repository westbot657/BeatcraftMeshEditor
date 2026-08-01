#version 450 core

uniform float u_beat_spacing;
uniform float u_step_spacing;
uniform float u_thick_line_px;
uniform float u_thin_line_px;

uniform sampler2D u_digit_tex;

in vec2 v_uv;

out vec4 fragColor;

void main() {
    if (v_uv.x < -0.5) {
        vec2 remap = v_uv + vec2(2.0); // both in [0,1]: remap.x = across track, remap.y = along beat span

        float world_z = remap.y * u_beat_spacing; // fragment's distance from this quad's start (z0)

        // distance to the nearest beat boundary (start or end of this span)
        float dist_to_boundary = min(world_z, u_beat_spacing - world_z);

        // distance to the nearest minor subdivision line
        float step_pos = mod(world_z, u_step_spacing);
        float dist_to_step = min(step_pos, u_step_spacing - step_pos);

        // fwidth(world_z) ~ world units per screen pixel at this fragment —
        // multiplying by this converts your hard-coded "line weight in pixels"
        // into the correct world-space threshold at this specific distance/angle
        float px = fwidth(world_z);

        float thick = 1.0 - smoothstep(0.0, u_thick_line_px * px, dist_to_boundary);
        float thin  = 1.0 - smoothstep(0.0, u_thin_line_px  * px, dist_to_step);
        float line = max(thick, thin);

        if (line <= 0.001) discard;
        fragColor = vec4(1.0, 1.0, 1.0, line); // white line, antialiased alpha falloff
    } else {
        fragColor = texture(u_digit_tex, v_uv);
    }
}

