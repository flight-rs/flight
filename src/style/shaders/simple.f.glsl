#version 410

in vec3 I_POS;
in vec3 I_COLOR;

out vec4 f_color;

void main() {
    f_color = vec4(pow(I_COLOR, vec3(1 / 2.2)), 1);
}