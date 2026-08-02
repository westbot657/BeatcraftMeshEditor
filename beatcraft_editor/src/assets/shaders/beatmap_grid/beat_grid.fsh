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
        vec2 remap = v_uv + vec2(2.0);
        float world_z = remap.y * u_beat_spacing;

        float dist_to_boundary = min(world_z, u_beat_spacing - world_z);

        float safe_step = max(u_step_spacing, 1e-5);
        float step_pos_norm = mod(remap.y, safe_step);
        float dist_to_step_norm = min(step_pos_norm, safe_step - step_pos_norm);
        float dist_to_step = dist_to_step_norm * u_beat_spacing;

        float px = max(fwidth(world_z), 1e-6);

        float thick = 1.0 - smoothstep(0.0, u_thick_line_px * px, dist_to_boundary);
        float thin  = 1.0 - smoothstep(0.0, u_thin_line_px  * px, dist_to_step);

        // draw thick (beat boundary) at full brightness, thin (subdivision) dimmer;
        // where they overlap, let the brighter one win rather than adding
        vec3 thick_color = vec3(1.0);
        vec3 thin_color  = vec3(0.35);

        float alpha = max(thick, thin);
        if (alpha <= 0.001) discard;

        vec3 color = mix(thin_color, thick_color, thick);
        fragColor = vec4(color, alpha);
    } else {
        fragColor = texture(u_digit_tex, v_uv);
    }
}
