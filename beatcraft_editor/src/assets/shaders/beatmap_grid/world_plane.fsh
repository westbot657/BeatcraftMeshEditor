#version 450 core

in vec2 v_uv;
in vec3 v_world_pos;
in float v_camera_height;

uniform float u_z;
uniform float u_rotation;

out vec4 fragColor;

void main() {
    float lane_half_width = 0.25; // 0.5 total world-space thickness for lanes

    // 1. Inverted rotation (-u_rotation) around Y-axis for the 4 lanes
    float cos_r = cos(-u_rotation);
    float sin_r = sin(-u_rotation);

    vec2 pos_2d = v_world_pos.xz;
    vec2 local_pos;
    local_pos.x =  pos_2d.x * cos_r + pos_2d.y * sin_r;
    local_pos.y = -pos_2d.x * sin_r + pos_2d.y * cos_r; // local Z

    // 2. Extent check for lanes: start at u_z and extend toward positive local Z
    float edge_z = clamp(fwidth(local_pos.y), 0.0001, 0.5);
    float alpha_z = smoothstep(u_z - edge_z, u_z + edge_z, local_pos.y);

    // 3. Lane distance check: 4 centers (-0.9, -0.3, 0.3, 0.9)
    float d1 = abs(local_pos.x - (-0.9));
    float d2 = abs(local_pos.x - (-0.3));
    float d3 = abs(local_pos.x - 0.3);
    float d4 = abs(local_pos.x - 0.9);

    float min_dist_lanes = min(min(d1, d2), min(d3, d4));

    float edge_x = clamp(fwidth(min_dist_lanes), 0.0001, 0.5);
    float alpha_lanes_x = 1.0 - smoothstep(lane_half_width - edge_x, lane_half_width + edge_x, min_dist_lanes);

    // Alpha for grey lanes (halved to 0.5 max alpha)
    float alpha_lanes = alpha_lanes_x * alpha_z * 0.5;

    // 4. Dynamic Screen-Space Quad Edge Fade (Anti-shimmer)
    vec2 dist_from_edge = 1.0 - abs(v_uv);
    vec2 fade_buffer = fwidth(v_uv) * 2.5; 

    vec2 uv_fade = smoothstep(vec2(0.0), fade_buffer, dist_from_edge);
    float alpha_uv = uv_fade.x * uv_fade.y;

    // Final alpha combination
    float final_alpha = alpha_lanes * alpha_uv;

    if (final_alpha <= 0.001) {
        discard;
    }

    fragColor = vec4(vec3(0.5), final_alpha);
}
