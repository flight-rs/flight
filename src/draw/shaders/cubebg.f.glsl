#version 410


uniform samplerCube cube_map;

layout(std140) uniform params {
    mat4 sun_matrix;
    vec4 sun_color;
    float sun_in_env;
    int radiance_levels;

    float gamma;
    float exposure;
};

in vec3 I_POS;
out vec4 f_color;

const float base_edge = 1.0 - cos(0.02);

void main() {
    vec3 lum = textureLod(cube_map, I_POS, 0.0).rgb;
    
    vec3 B = normalize(I_POS);
    vec3 L = -(sun_matrix * vec4(0.0, 0.0, -1.0, 0.0)).xyz;
    float sun_dot = dot(B, L);
    vec3 sun_lum = vec3(1.0, 1.0, 1.0);

    float ratio = length(sun_lum);
    float edge = 1.0 - base_edge * ratio;

    lum += sun_lum * smoothstep(edge, 1.0, sun_dot);

    // hdr to ldr  
    vec3 mapped = vec3(1.0) - exp(-lum * exposure);
    mapped = pow(mapped, vec3(1.0 / gamma));
    f_color = vec4(mapped, 1.0);
}
