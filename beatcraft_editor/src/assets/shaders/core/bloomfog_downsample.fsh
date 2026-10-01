#version 150

uniform sampler2D Sampler0;

in vec2 texCoord0;

out vec4 fragColor;

void main() {
    vec2 texelSize = 1.0 / vec2(textureSize(Sampler0, 0));
    vec4 d = texelSize.xyxy * vec4(-1.0, -1.0, 1.0, 1.0);

    vec4 s;
    s  = texture(Sampler0, texCoord0 + d.xy);
    s += texture(Sampler0, texCoord0 + d.zy);
    s += texture(Sampler0, texCoord0 + d.xw);
    s += texture(Sampler0, texCoord0 + d.zw);

    fragColor = s / 4.0;
}
