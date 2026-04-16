#version 330 core

// Vertex Layout
layout(location =  0) in vec4  in_position_u;
layout(location =  1) in vec4  in_normal_v;
layout(location =  2) in ivec3 in_colorLayer_materialLayer_flags;
// Instance Layout
layout(location =  3) in vec4  clipping_plane;
layout(location =  4) in mat4  instance_model;
//     location =  5           column 2
//     location =  6           column 3
//     location =  7           column 4
layout(location =  8) in vec4  c0;
layout(location =  9) in vec4  c1;
layout(location = 10) in vec4  c2;
layout(location = 11) in vec4  c3;
layout(location = 12) in vec4  c4;
layout(location = 13) in vec4  c5;
layout(location = 14) in vec4  c6;
layout(location = 15) in vec4  c7;

uniform int passType; // 0 = normal, 1 = bloom, 2 = bloomfog, 3 = late lights
uniform mat4 u_projection;
uniform mat4 u_view;
uniform mat4 world_transform;

uniform int u_render_mode;

out vec4 v_screen_uv;


void main() {
    vec4 pos = vec4(u_view * vec4(in_position_u.xyz, 1.0));
    vec4 pos2 = u_projection * pos;
    gl_Position = pos2;
    v_screen_uv = vec4(pos2.xyz, -pos.z);
}



