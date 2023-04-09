#version 150
in vec2 position;
in float segment;
out vec4 window_pos;
out float segment_frag;

uniform mat4 MVP;

void main() {
    gl_Position = MVP*vec4(position, 0.0, 1.0);
    window_pos = gl_Position;
    segment_frag = segment;
}