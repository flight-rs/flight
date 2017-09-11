#version 410

in vec3 I_POS;
in vec3 I_COLOR;

out vec4 f_color;

void main() {
    f_color = vec4(I_COLOR, 1);
}