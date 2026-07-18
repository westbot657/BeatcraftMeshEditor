#version 330 core

uniform mat4 view;
uniform mat4 proj;

out vec2 localXZ;   // camera-relative offset — small, precision-safe
out vec3 worldPos;  // absolute world position — for axis-line checks
out float camHeight;

const vec2 corners[4] = vec2[4](
    vec2(-1.0, -1.0),
    vec2( 1.0, -1.0),
    vec2(-1.0,  1.0),
    vec2( 1.0,  1.0)
);

void main() {
    vec3 cameraPos = -transpose(mat3(view)) * view[3].xyz;
    float height = max(abs(cameraPos.y), 1.0);

    // Quad extent scales with camera height, so it always covers the
    // visible ground regardless of zoom level.
    float halfSize = height * 50.0;

    vec2 offset = corners[gl_VertexID] * halfSize;
    vec3 pos = vec3(cameraPos.x + offset.x, 0.0, cameraPos.z + offset.y);

    localXZ = offset;       // bounded magnitude, always precision-safe
    worldPos = pos;
    camHeight = height;

    gl_Position = proj * view * vec4(pos, 1.0);
}
