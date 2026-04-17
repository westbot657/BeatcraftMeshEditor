#version 330 core

in vec2 v_uv;
in vec4 v_color;
in vec3 v_pos;
in vec3 v_normal;
flat in int v_material; // 0 = solid, 1 = light/solid, 2 = light/nothing
flat in int v_flags;
in vec3 screenUV;

uniform int passType; // 0 = normal, 1 = bloom, 2 = bloomfog, 3 = late lights
uniform sampler2D u_texture;
uniform sampler2D u_bloomfog;
uniform sampler2D u_depth;

uniform sampler2D u_noise;
uniform int u_render_mode; // 0 = Beatcraft, 1 = Editor, 2 = Wireframe

uniform vec2 u_fog;

out vec4 fragColor;

const vec3 LIGHT = normalize(vec3(0.6, 1.0, 0.4));


const int BAYER[16] = int[16](
     0,  8,  2, 10,
    12,  4, 14,  6,
     3, 11,  1,  9,
    15,  7, 13,  5
);

float clampF(float t) {
    return clamp((t / 100) - 0.001, 0.0, 0.8);
}

vec4 lerpColor(vec4 c1, vec4 c2, float t) {
    return c1 + (c2 * clamp(t, 0.0, 1.0));
}

void main() {
    if (u_render_mode == 2) {
        fragColor = vec4(vec3(0.4), 0.4);
        return;
    } else if (u_render_mode == 1) {
        float x = gl_FragCoord.x;
        float y = gl_FragCoord.y;

        int bx = int(mod(x, 4.0));
        int by = int(mod(y, 4.0));
        int bayer = BAYER[by * 4 + bx];

        float noise = texture(u_noise, vec2(x, y) / vec2(textureSize(u_noise, 0))).r;
        float depth = gl_FragCoord.z / gl_FragCoord.w + (noise - 0.5) * 3.5;
        vec4 vColor = v_color;
        if (!gl_FrontFacing) {
            if (vColor.r < 0.01 && vColor.g < 0.01 && vColor.b < 0.01) {
                vColor = vec4(0.2, 0.3, 0.8, 1.0);
            }
            float t = 1.0 - clamp(depth / 100.0, 0.0, 1.0);
            int threshold = int(mix(15.0, 1.0, t));
            if (bayer >= threshold) discard;
            vColor = vec4(vColor.rgb * vec3(2.0, 2.0, 4.0), vColor.a);
        }

        vec3 N = normalize(v_normal);
        if (!gl_FrontFacing) N = -N;
        float diff = max(dot(N, LIGHT), 0.0) * 0.2 + 0.8;
        vec4 base = vColor;
        if (gl_FrontFacing) {
            if (v_flags == 2147483648) {
                base = vec4(vec3(0.0), 1.0);
            } else {
                base = base * texture(u_texture, v_uv);
            }
        }
        fragColor = base;
    } else {
        if (passType == 0 /* Normal */ && v_material != 2 /* Not Light/Nothing */) {
            vec4 tex = texture(u_texture, v_uv) * v_color;
            if (v_flags == 2147483648) { // 1 << 31
                tex = vec4(vec3(0.0), 1.0);
            }
            if (v_material == 1 /* Light/Solid */) {
                tex = vec4(tex.rgb, 1.0);
            }
            vec4 fog = texture(u_bloomfog, (screenUV.xy/(-screenUV.z*4.0))+0.5);
            float fadeHeight = clamp((v_pos.y - u_fog.x) / (u_fog.y - u_fog.x), 0.0, 1.0);
            fragColor = lerpColor(tex * fadeHeight, fog, clampF(abs(screenUV.z)));
        } else if (passType == 1 /* Bloom */) {
            if (v_material == 0 /* Solid */) {
                discard;
            } else {
                vec2 uv = (screenUV.xy / (-screenUV.z * 2)) + 0.5;
                float sceneDepth = texture(u_depth, uv).r;
                if (sceneDepth < gl_FragCoord.z-0.000001) {
                    discard;
                }
                vec4 tex = texture(u_texture, v_uv) * v_color;
                float fadeHeight = clamp((v_pos.y - u_fog.x) / (u_fog.y - u_fog.x), 0.0, 1.0);
                fragColor = lerpColor(tex * fadeHeight, vec4(0.0), clampF(abs(screenUV.z)));
            }
        } else if (passType == 2 /* Bloomfog */) {
            if (v_material == 0 /* Solid */) {
                discard;
            } else {
                fragColor = v_color;
            }
        } else if (passType == 3 /* Late Lights */ && v_material == 2 /* Light/Nothing */) {
            vec4 tex = texture(u_texture, v_uv) * v_color;
            float fadeHeight = clamp((v_pos.y - u_fog.x) / (u_fog.y - u_fog.x), 0.0, 1.0);
            fragColor = lerpColor(tex * fadeHeight, vec4(0.0), clampF(abs(screenUV.z)));
        } else {
            discard;
        }
    }
}
