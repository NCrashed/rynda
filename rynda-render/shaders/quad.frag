#version 150
uniform sampler2D img_output;
in vec2 tex_coords;
out vec4 f_color;

void main() {
    f_color = texture(img_output, tex_coords);
}