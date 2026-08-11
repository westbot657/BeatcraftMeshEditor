#version 450 core

uniform mat4 u_view;
uniform mat4 u_proj;

out vec2 v_uv;
out vec3 v_world_pos;
out float v_camera_height;

const vec2 corners[4] = vec2[4](
    vec2(-1.0, -1.0),
    vec2( 1.0, -1.0),
    vec2(-1.0,  1.0),
    vec2( 1.0,  1.0)
);

void main() {
    vec3 cameraPos = -transpose(mat3(u_view)) * u_view[3].xyz;
    float height = max(abs(cameraPos.y), 1.0);

    vec2 offset = corners[gl_VertexID];
    vec3 pos = vec3(cameraPos.x + offset.x * 1000.0, 0.0, cameraPos.z + offset.y * 1000.0);

    v_uv = offset;
    v_world_pos = pos;
    v_camera_height = height;

    gl_Position = u_proj * u_view * vec4(pos, 1.0);
}

