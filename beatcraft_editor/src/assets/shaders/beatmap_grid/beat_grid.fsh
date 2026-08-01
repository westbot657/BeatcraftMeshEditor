#version 450 core

uniform float u_beat_spacing;
uniform float u_step_spacing;

uniform Sampler2D u_digit_tex;

in vec2 v_uv;

ouv vec4 fragColor;

void main() {
    if (v_uv.x < -0.5) {
        vec2 remap = v_uv + vec2(2.0);

        // thick lines at start/end Z, thin line every u_step_spacing between.

        // thick line length goes to quad width
        // thin line goes to width - 0.25 each side

    } else {
        fragColor = texture(u_digit_tex, v_uv);
    }
}

