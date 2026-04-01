#version 330 core
layout(location=0) in vec3 aPos;
layout(location=1) in vec2 aUV;
layout(location=2) in vec3 aNorm;
layout(location=3) in int  aChannel;
layout(location=4) in vec4 aModel0;
layout(location=5) in vec4 aModel1;
layout(location=6) in vec4 aModel2;
layout(location=7) in vec4 aModel3;
layout(location=8) in vec4 aColorAlpha;

uniform mat4 uVP;

flat out int vCh;
out vec3 vN;
out vec2 vUV;
out vec4 vColorAlpha;

void main() {
    mat4 model = mat4(aModel0, aModel1, aModel2, aModel3);
    vec4 world = model * vec4(aPos, 1.0);
    vec4 pos = uVP * world;
    gl_Position = pos;
    vN = normalize(mat3(model) * aNorm);
    vCh = aChannel;
    vUV = vec2(aUV);
    vColorAlpha = aColorAlpha;
}
