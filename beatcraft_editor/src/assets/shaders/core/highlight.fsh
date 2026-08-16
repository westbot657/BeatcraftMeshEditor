#version 450 core

uniform sampler2D Sampler0;

uniform vec2 texelSize;
uniform float GameTime;

in vec2 texCoord0;

out vec4 fragColor;

void main() {
    float c = texture(Sampler0, texCoord0).r;
    if (c <= 0.1 || c > 0.5) {
        discard;
    }
    fragColor = vec4(0.5 + c);
}
