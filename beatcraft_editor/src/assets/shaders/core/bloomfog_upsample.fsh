#version 150


uniform sampler2D Sampler0;

in vec2 texCoord0;

out vec4 fragColor;


// void main() {
//     vec2 texelSize = 1.0 / vec2(textureSize(Sampler0, 0));
//     vec4 d = texelSize.xyxy * vec4(1.0, 1.0, -1.0, 0.0);
//
//     vec4 s;
//     s  = texture(Sampler0, texCoord0 - d.xy);
//     s += texture(Sampler0, texCoord0 - d.wy) * 2.0;
//     s += texture(Sampler0, texCoord0 - d.zy);
//
//     s += texture(Sampler0, texCoord0 + d.zw) * 2.0;
//     s += texture(Sampler0, texCoord0       ) * 4.0;
//     s += texture(Sampler0, texCoord0 + d.xw) * 2.0;
//
//     s += texture(Sampler0, texCoord0 + d.zy);
//     s += texture(Sampler0, texCoord0 + d.wy) * 2.0;
//     s += texture(Sampler0, texCoord0 + d.xy);
//
//     fragColor = s / 8.0;
// }

void main() {
    vec2 texelSize = 1.0 / vec2(textureSize(Sampler0, 0));
    vec4 d = texelSize.xyxy * vec4(0.5, 0.5, -0.5, 0.0);

    vec4 s;
    s  = texture(Sampler0, texCoord0 - d.xy);  // (-0.5, -0.5)
    s += texture(Sampler0, texCoord0 - d.zy);  // (+0.5, -0.5)
    s += texture(Sampler0, texCoord0 + d.zy);  // (-0.5, +0.5)
    s += texture(Sampler0, texCoord0 + d.xy);  // (+0.5, +0.5)

    fragColor = s * 0.5; // * 0.25 // 0.5 = 2x brightness, 0.25 = original brightness
}
