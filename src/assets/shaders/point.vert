// point.vert
#version 330 core
layout(location=0) in vec3  aPos;
layout(location=3) in vec4  aModel0;
layout(location=4) in vec4  aModel1;
layout(location=5) in vec4  aModel2;
layout(location=6) in vec4  aModel3;
layout(location=7) in vec4  aColorAlpha;

uniform mat4  uVP;
uniform float uPointSize;

out vec4 vColorAlpha;

void main() {
    mat4 model    = mat4(aModel0, aModel1, aModel2, aModel3);
    gl_Position   = uVP * model * vec4(aPos, 1.0);
    gl_PointSize  = uPointSize;
    vColorAlpha   = aColorAlpha;
}
