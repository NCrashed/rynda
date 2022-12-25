#version 150
uniform int segment;
in vec2 tex_coords;
out vec4 f_color;

void main() {
    if (segment == 0) {
        f_color = vec4(1.0, 0.0, 0.0, 1.0);
    } else if (segment == 1) {
        f_color = vec4(0.0, 1.0, 0.0, 1.0);
    } else if (segment == 2) {
        f_color = vec4(0.0, 0.0, 1.0, 1.0);
    } else {
        f_color = vec4(1.0, 0.0, 1.0, 1.0);
    }
}