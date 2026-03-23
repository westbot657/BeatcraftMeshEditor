// mesh.vert
#version 330 core
layout(location=0) in vec3 aPos;
layout(location=1) in vec3 aNorm;
layout(location=2) in int  aChannel;
// per-instance
layout(location=3) in vec4 aModel0;
layout(location=4) in vec4 aModel1;
layout(location=5) in vec4 aModel2;
layout(location=6) in vec4 aModel3;
layout(location=7) in vec4 aColorAlpha;    // rgb + alpha

uniform mat4 uVP;
uniform vec3 uCamPos;

flat out int vCh;
out vec3 vN;
out vec4 vColorAlpha;
out float vDepth;

void main() {
    mat4 model = mat4(aModel0, aModel1, aModel2, aModel3);
    vec4 world = model * vec4(aPos, 1.0);
    gl_Position = uVP * world;
    vN = normalize(mat3(model) * aNorm);
    vCh = aChannel;
    vColorAlpha = aColorAlpha;
    vDepth = distance(world.xyz, uCamPos);
}
