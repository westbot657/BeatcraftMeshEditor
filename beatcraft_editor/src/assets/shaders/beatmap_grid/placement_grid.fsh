#version 450 core

in vec2 v_uv;

out vec4 fragColor;

uniform uint u_hovered_cell;

const float GRID_COLS = 4.0;
const float GRID_ROWS = 3.0;
const float RECT_HALF_SIZE = 0.4;
const float CORNER_RADIUS = 0.08;

void main() {
    vec2 cell_uv = vec2(fract(v_uv.x * GRID_COLS), fract(v_uv.y * GRID_ROWS));
    vec2 p = cell_uv - 0.5;

    vec2 half_size = vec2(RECT_HALF_SIZE);
    float safe_radius = min(CORNER_RADIUS, min(half_size.x, half_size.y));

    vec2 d = abs(p) - (half_size - vec2(safe_radius));
    float dist = length(max(d, 0.0)) + min(max(d.x, d.y), 0.0) - safe_radius;

    float aa = fwidth(dist) * 0.5;
    float alpha = 1.0 - smoothstep(-aa, aa, dist);
    if (alpha <= 0.001) discard;

    uint col = uint(floor(v_uv.x * GRID_COLS));
    uint row = uint(floor(v_uv.y * GRID_ROWS));
    uint cell_id = row * uint(GRID_COLS) + col;

    vec3 color = (cell_id == u_hovered_cell) ? vec3(0.5) : vec3(0.25);
    fragColor = vec4(color, alpha * 0.5);
}
