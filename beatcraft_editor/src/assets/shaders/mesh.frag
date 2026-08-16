#version 450 core

in vec2 v_uv;
in vec4 v_color;
in vec3 v_pos;
in vec3 v_normal;
flat in int v_material;
flat in int v_style;
flat in int v_flags;
in vec3 screenUV;
in vec3 worldPos;

uniform int passType;
uniform sampler2D u_texture;
uniform sampler2D u_bloomfog;
uniform sampler2D u_depth;
uniform float u_time;

uniform sampler2D u_noise;
uniform int u_render_mode;

uniform vec2 u_fog;

out vec4 fragColor;

const vec3 LIGHT = normalize(vec3(0.6, 1.0, 0.4));


const int MAT_SOLID             = 0;
const int MAT_SOLID_LIGHT       = 1;
const int MAT_TRANSPARENT_LIGHT = 2;
const int MAT_TINTED            = 3;

const int MAT_NOTE              = 4;
const int MAT_ARROW             = 5;

const int MAT_OBSTACLE          = 6;

const int MAT_ARC               = 7;


const int PASS_NORMAL      = 0;
const int PASS_BLOOM       = 1;
const int PASS_BLOOMFOG    = 2;
const int PASS_LATE_LIGHTS = 3;
const int PASS_OBSTACLE    = 4;
const int PASS_HIGHLIGHT   = 5;


const int MODE_BEATCRAFT        = 0;
const int MODE_EDITOR           = 1;
const int MODE_EDITOR_WIREFRAME = 2;


const int STYLE_DEFAULT = 0;
const int STYLE_CIRCLE  = 1;


const int FLAG_OVERRIDE_BLACK_BIT = 31;


const int BAYER[16] = int[16](
     0,  8,  2, 10,
    12,  4, 14,  6,
     3, 11,  1,  9,
    15,  7, 13,  5
);


// Perlin Noise Functions
vec4 permute(vec4 x){return mod(((x*34.0)+1.0)*x, 289.0);}
vec4 taylorInvSqrt(vec4 r){return 1.79284291400159 - 0.85373472095314 * r;}
vec3 fade(vec3 t) {return t*t*t*(t*(t*6.0-15.0)+10.0);}

float cnoise(vec3 P){
    vec3 Pi0 = floor(P);
    vec3 Pi1 = Pi0 + vec3(1.0);
    Pi0 = mod(Pi0, 289.0);
    Pi1 = mod(Pi1, 289.0);
    vec3 Pf0 = fract(P);
    vec3 Pf1 = Pf0 - vec3(1.0);
    vec4 ix = vec4(Pi0.x, Pi1.x, Pi0.x, Pi1.x);
    vec4 iy = vec4(Pi0.yy, Pi1.yy);
    vec4 iz0 = Pi0.zzzz;
    vec4 iz1 = Pi1.zzzz;

    vec4 ixy = permute(permute(ix) + iy);
    vec4 ixy0 = permute(ixy + iz0);
    vec4 ixy1 = permute(ixy + iz1);

    vec4 gx0 = ixy0 / 7.0;
    vec4 gy0 = fract(floor(gx0) / 7.0) - 0.5;
    gx0 = fract(gx0);
    vec4 gz0 = vec4(0.5) - abs(gx0) - abs(gy0);
    vec4 sz0 = step(gz0, vec4(0.0));
    gx0 = gx0 - (sz0 * (step(0.0, gx0) - 0.5));
    gy0 = gy0 - (sz0 * (step(0.0, gy0) - 0.5));

    vec4 gx1 = ixy1 / 7.0;
    vec4 gy1 = fract(floor(gx1) / 7.0) - 0.5;
    gx1 = fract(gx1);
    vec4 gz1 = vec4(0.5) - abs(gx1) - abs(gy1);
    vec4 sz1 = step(gz1, vec4(0.0));
    gx1 = gx1 - (sz1 * (step(0.0, gx1) - 0.5));
    gy1 = gy1 - (sz1 * (step(0.0, gy1) - 0.5));

    vec3 g000 = vec3(gx0.x,gy0.x,gz0.x);
    vec3 g100 = vec3(gx0.y,gy0.y,gz0.y);
    vec3 g010 = vec3(gx0.z,gy0.z,gz0.z);
    vec3 g110 = vec3(gx0.w,gy0.w,gz0.w);
    vec3 g001 = vec3(gx1.x,gy1.x,gz1.x);
    vec3 g101 = vec3(gx1.y,gy1.y,gz1.y);
    vec3 g011 = vec3(gx1.z,gy1.z,gz1.z);
    vec3 g111 = vec3(gx1.w,gy1.w,gz1.w);

    vec4 norm0 = taylorInvSqrt(vec4(dot(g000, g000), dot(g010, g010), dot(g100, g100), dot(g110, g110)));
    g000 = g000 * norm0.x;
    g010 = g010 * norm0.y;
    g100 = g100 * norm0.z;
    g110 = g110 * norm0.w;
    vec4 norm1 = taylorInvSqrt(vec4(dot(g001, g001), dot(g011, g011), dot(g101, g101), dot(g111, g111)));
    g001 = g001 * norm1.x;
    g011 = g011 * norm1.y;
    g101 = g101 * norm1.z;
    g111 = g111 * norm1.w;

    float n000 = dot(g000, Pf0);
    float n100 = dot(g100, vec3(Pf1.x, Pf0.yz));
    float n010 = dot(g010, vec3(Pf0.x, Pf1.y, Pf0.z));
    float n110 = dot(g110, vec3(Pf1.xy, Pf0.z));
    float n001 = dot(g001, vec3(Pf0.xy, Pf1.z));
    float n101 = dot(g101, vec3(Pf1.x, Pf0.y, Pf1.z));
    float n011 = dot(g011, vec3(Pf0.x, Pf1.yz));
    float n111 = dot(g111, Pf1);

    vec3 fade_xyz = fade(Pf0);
    vec4 n_z = mix(vec4(n000, n100, n010, n110), vec4(n001, n101, n011, n111), fade_xyz.z);
    vec2 n_yz = mix(n_z.xy, n_z.zw, fade_xyz.y);
    float n_xyz = mix(n_yz.x, n_yz.y, fade_xyz.x);
    return clamp(2.2 * n_xyz, -1.0, 1.0);
}

float clampF(float t) {
    return clamp((t / 100) - 0.001, 0.0, 0.8);
}

vec4 lerpColor(vec4 c1, vec4 c2, float t) {
    return c1 + (c2 * clamp(t, 0.0, 1.0));
}

vec4 blendColors(vec4 bgColor, vec4 fgColor) {
    float outAlpha = fgColor.a + bgColor.a * (1.0 - fgColor.a);
    vec3 outRGB = (fgColor.rgb * fgColor.a + bgColor.rgb * bgColor.a * (1.0 - fgColor.a)) / outAlpha;
    return vec4(outRGB, outAlpha);
}

void main() {
    bool overrideBlack = (v_flags & (1 << FLAG_OVERRIDE_BLACK_BIT)) != 0;
    if (u_render_mode == MODE_EDITOR_WIREFRAME) {
        fragColor = vec4(vec3(0.4), 0.4);
        return;
    }
    if (v_style == STYLE_CIRCLE) {
        if (length(v_uv - 0.5) > 0.5) {
            discard;
        }
    }
    if (passType == PASS_HIGHLIGHT) {
        fragColor = v_color;
        return;
    }

    if (u_render_mode == MODE_EDITOR) {
        float x = gl_FragCoord.x;
        float y = gl_FragCoord.y;

        int bx = int(mod(x, 4.0));
        int by = int(mod(y, 4.0));
        int bayer = BAYER[by * 4 + bx];

        float noise = texture(u_noise, vec2(x, y) / vec2(textureSize(u_noise, 0))).r;
        float depth = gl_FragCoord.z / gl_FragCoord.w + (noise - 0.5) * 3.5;
        vec4 vColor = v_color;
        vec3 N = normalize(v_normal);
        if (!gl_FrontFacing) {
            if (v_material == MAT_OBSTACLE) {
                discard;
            }
            if (vColor.r > 0.99 && vColor.g > 0.99 && vColor.b > 0.99) {
                vColor = vec4(0.2, 0.3, 0.8, 1.0);
            }
            float t = 1.0 - clamp(depth / 100.0, 0.0, 1.0);
            int threshold = int(mix(15.0, 1.0, t));
            if (bayer >= threshold) discard;
            vColor = vec4(vColor.rgb * vec3(2.0, 2.0, 4.0), vColor.a);
            N = -N;
        }

        float diff = max(dot(N, LIGHT), 0.0) * 0.2 + 0.8;
        vec4 base = vColor;
        if (gl_FrontFacing) {
            if (overrideBlack) {
                base = vec4(vec3(0.0), 1.0);
            } else if (v_style == STYLE_DEFAULT) {
                base = base * texture(u_texture, v_uv);
            }
        }
        fragColor = base;
    } else {

        vec4 tex_sample = ((v_style == STYLE_DEFAULT)
            ? texture(u_texture, v_uv)
            : vec4(1.0)) * v_color;

        if (passType == PASS_NORMAL && v_material != MAT_TRANSPARENT_LIGHT) {
            vec4 tex = tex_sample;
            if (overrideBlack) {
                tex = vec4(vec3(0.0), 1.0);
            }
            else if (v_material == MAT_SOLID_LIGHT) {
                tex = vec4(tex.rgb, 1.0);
            }
            vec4 fog = texture(u_bloomfog, (screenUV.xy/(-screenUV.z*4.0))+0.5);
            float fadeHeight = clamp((v_pos.y - u_fog.x) / (u_fog.y - u_fog.x), 0.0, 1.0);
            fragColor = lerpColor(tex * fadeHeight, fog, clampF(abs(screenUV.z)));
        } else if (passType == PASS_BLOOM ) {
            if (v_material == MAT_SOLID || v_material == MAT_TINTED ) {
                discard;
            } else {
                vec2 uv = (screenUV.xy / (-screenUV.z * 2)) + 0.5;
                float sceneDepth = texture(u_depth, uv).r;
                if (sceneDepth < gl_FragCoord.z-0.000001) {
                    discard;
                }
                float fadeHeight = clamp((v_pos.y - u_fog.x) / (u_fog.y - u_fog.x), 0.0, 1.0);
                fragColor = lerpColor(tex_sample * fadeHeight, vec4(0.0), clampF(abs(screenUV.z)));
            }
        } else if (passType == PASS_BLOOMFOG ) {
            if (v_material == MAT_SOLID || v_material == MAT_TINTED ) {
                discard;
            } else {
                fragColor = v_color;
            }
        } else if (passType == PASS_LATE_LIGHTS && v_material == MAT_TRANSPARENT_LIGHT) {
            float fadeHeight = clamp((v_pos.y - u_fog.x) / (u_fog.y - u_fog.x), 0.0, 1.0);
            fragColor = lerpColor(tex_sample * fadeHeight, vec4(0.0), clampF(abs(screenUV.z)));
        } else if (passType == PASS_OBSTACLE) {
            vec2 uv = (screenUV.xy / (-screenUV.z * 2)) + 0.5;
            float depth = texture(u_depth, uv).r;
            if (depth < gl_FragCoord.z-0.000001) {
                discard;
            }
            float distortion_strength = 0.01 / screenUV.z;
            float time = u_time * 1.25;
            vec3 noise_in = worldPos + vec3(time);
            vec2 distortion = vec2(
                cnoise(noise_in + vec3(23.1, 0.0, 0.0)),
                cnoise(noise_in + vec3(0.0, 23.1, 0.0))
            ) * distortion_strength;

            vec2 distorted_uv = uv + distortion;

            // u_bloomfog is replaced with the screen grab-pass
            // for the obstacle pass
            vec4 color = texture(u_bloomfog, distorted_uv);
            fragColor = blendColors(color, v_color);
        } else {
            discard;
        }
    }
}
