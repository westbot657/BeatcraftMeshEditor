#version 330 core
flat in int vCh;
in vec3 vN;
in float vA;
in vec3 vPC;
flat in int vUPC;
uniform int uWire;
out vec4 fragColor;
const vec3 LIGHT=normalize(vec3(0.6,1.0,0.4));
const vec3 COLORS[8]=vec3[8](
    vec3(0.55,0.60,0.70),vec3(0.80,0.35,0.35),
    vec3(0.35,0.70,0.45),vec3(0.80,0.75,0.30),
    vec3(0.35,0.55,0.80),vec3(0.75,0.40,0.75),
    vec3(0.30,0.75,0.80),vec3(0.80,0.60,0.35)
);
void main(){
    vec3 N=normalize(vN);
    float diff=max(dot(N,LIGHT),0.0)*0.6+0.4;
    vec3 base=(vUPC!=0)?vPC:COLORS[vCh&7];
    if(uWire!=0){ fragColor=vec4(0.65,0.75,1.0,0.6); return; }
    fragColor=vec4(base*diff,vA);
}
