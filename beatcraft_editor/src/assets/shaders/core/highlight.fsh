#version 450 core

uniform sampler2D Sampler0;

uniform vec2 texelSize;
uniform float GameTime;

in vec2 texCoord0;

out vec4 fragColor;

void main() {
    vec3 col = texture(Sampler0, texCoord0).rgb;
    float c = max(col.r, max(col.g, col.b));
    if (c <= 0.2 || c > 0.5) {
        discard;
    }
    fragColor = vec4(col + vec3(0.5), 1.0);
}
