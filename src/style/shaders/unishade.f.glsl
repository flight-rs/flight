#version 410

layout(std140) uniform shade {
    vec4 dark;
    vec4 light;
};

in vec3 I_POS;
in vec3 I_NORM;

out vec4 f_color;

void main() {
    float lightness = max(dot(vec3(0, 1, 0), I_NORM), 0);
    f_color = mix(dark, light, lightness);
    f_color.xyz = pow(f_color.xyz, vec3(1 / 2.2));
}