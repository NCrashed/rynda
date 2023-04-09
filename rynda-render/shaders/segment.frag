#version 150
uniform float segment;
in vec2 tex_coords;
out vec4 f_color;

void main() {
    if (segment < 1.0) {
        f_color = vec4(0.4 + 0.2 * (int(tex_coords.y * 10.0) % 2), 0.0, 0.0, 1.0);
    } else if (segment < 2.0) {
        f_color = vec4(0.0, 0.4 + 0.2 * (int(tex_coords.x * 10.0) % 2), 0.0, 1.0);
    } else if (segment < 3.0) {
        f_color = vec4(0.0, 0.0, 0.4 + 0.2 * (int(tex_coords.y * 10.0) % 2), 1.0);
    } else {
        f_color = vec4(0.4 + 0.2 * (int(tex_coords.x * 10.0) % 2), 0.0, 0.4 + 0.2 * (int(tex_coords.x * 10.0) % 2), 1.0);;
    }
}