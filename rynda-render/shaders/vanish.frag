#version 330
//uniform vec2 vp_point;
uniform sampler2D segment_front;
uniform sampler2D segment_bottom;
uniform sampler2D segment_left;
uniform sampler2D segment_right;

in vec2 tex_coords;
in float segment_frag;
layout(location = 0) out vec4 color;

void main() {
    if (segment_frag < 1.0) {
        color = vec4(1.0, 0.0, 0.0, 1.0);
    } else if (segment_frag < 2.0) {
        color = vec4(0.0, 1.0, 0.0, 1.0);
    } else if (segment_frag < 3.0) {
        color = vec4(0.0, 0.0, 1.0, 1.0);
    } else {
        color = vec4(1.0, 0.5, 0.5, 1.0);
    }
}