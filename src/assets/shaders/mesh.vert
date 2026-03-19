#version 330 core
layout(location=0) in vec3 aPos;
layout(location=1) in vec3 aNorm;
layout(location=2) in int  aChannel;
uniform mat4 uMVP;
uniform mat4 uNormalMat;
uniform vec3 uPartColor;
uniform int  uUsePartColor;
uniform float uAlpha;
flat out int  vCh;
out vec3 vN;
out float vA;
out vec3 vPC;
flat out int vUPC;
void main(){
    gl_Position=uMVP*vec4(aPos,1.0);
    vCh=aChannel; vN=normalize(mat3(uNormalMat)*aNorm);
    vA=uAlpha; vPC=uPartColor; vUPC=uUsePartColor;
}
