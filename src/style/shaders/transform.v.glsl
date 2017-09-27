#version 410
 
layout(std140) uniform transform {
    mat4 model;
    mat4 view;
    mat4 proj;
    vec4 eye_pos;
    float clip_offset;
};

in vec3 a_pos;
out vec3 v_pos;

#ifdef NORM
in vec3 a_norm;
out vec3 v_norm;
#endif

#ifdef TEX
in vec2 a_tex;
out vec2 v_tex;
#endif

#ifdef COLOR
in vec3 a_color;
out vec3 v_color;
#endif

#ifdef TAN
in vec3 a_tan;
out vec3 v_tan;
in vec3 a_bitan;
out vec3 v_bitan;
#endif

void main() {
    vec4 p = model * vec4(a_pos, 1);
    v_pos = p.xyz;

    #ifdef NORM
    v_norm = (model * vec4(a_norm, 0)).xyz;
    #endif

    #ifdef TEX
    v_tex = a_tex;
    v_tex.y = 1 - v_tex.y;
    #endif

    #ifdef COLOR
    v_color = a_color;
    #endif

    #ifdef TAN
    v_tan = (model * vec4(a_tan, 0)).xyz;
    v_bitan = (model * vec4(a_bitan, 0)).xyz;
    #endif

    vec4 c = proj * view * p;
    // Fake an opengl viewport
    // TODO: Submit a PR to GFX
    c.x /= 2 * c.w;
    c.x += clip_offset;
    c.x *= c.w;
    gl_Position = c;
}