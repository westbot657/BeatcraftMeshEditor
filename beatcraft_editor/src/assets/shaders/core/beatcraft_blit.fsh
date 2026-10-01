#version 330

uniform sampler2D Sampler0;

uniform float GameTime; // required to be swappable with other effect passes

in vec2 texCoord0;

out vec4 fragColor;

void main() {
    vec4 texColor = texture(Sampler0, texCoord0);
    fragColor = texColor;
}
