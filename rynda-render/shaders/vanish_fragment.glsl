#version 330
uniform vec2 vp_point;
uniform sampler2D segment_front;
uniform sampler2D segment_bottom;
uniform sampler2D segment_left;
uniform sampler2D segment_right;

in vec2 tex_coords;
flat in int segment_frag;
layout(location = 0) out vec4 color;

void main() {
    if (segment_frag == 0) {
        color = vec4(1.0 + vp_point.x, 0.0, 0.0, 1.0);
    } else if (segment_frag == 1) {
        color = vec4(0.0, 1.0, 0.0, 1.0);
    } else if (segment_frag == 2) {
        color = vec4(0.0, 0.0, 1.0, 1.0);
    } else {
        color = vec4(1.0, 0.5, 0.5, 1.0);
    }
}