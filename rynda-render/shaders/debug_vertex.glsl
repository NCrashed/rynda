#version 150
in vec3 position;
in vec3 color;

uniform mat4 MVP;

out vec3 frag_color;

void main() {
    gl_Position = MVP * vec4(position, 1.0);
    frag_color = color;
}