#version 150
in vec3 frag_color;
out vec4 f_color;

void main() {
    f_color = vec4(frag_color, 0.0);
}