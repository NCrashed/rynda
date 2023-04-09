#version 440
uniform vec2 vp_point;
layout(binding = 0) uniform sampler2D segment_right;
layout(binding = 1) uniform sampler2D segment_front;
layout(binding = 2) uniform sampler2D segment_left;
layout(binding = 3) uniform sampler2D segment_bottom;

in vec4 window_pos;
in float segment_frag;
layout(location = 0) out vec4 color;

void main() {
    float xs = window_pos.x / window_pos.w;
    float ys = window_pos.y / window_pos.w;
    vec2 uv = vec2(0.0, 0.0);
    float s1 = 0;
    float s2 = 0;
    float s3 = 0;
    float s4 = 0;
    if (segment_frag < 1.0) {
        uv.y = (ys - vp_point.y) / abs(xs - vp_point.x) + s1;
        uv.x = xs - vp_point.x;
        color = texture( segment_right, uv );
    } else if (segment_frag < 2.0) {
        uv.x = (xs - vp_point.x) / abs(ys - vp_point.y) + s2;
        uv.y = ys - vp_point.y;
        color = texture( segment_front, uv );
    } else if (segment_frag < 3.0) {
        uv.y = (ys - vp_point.y) / abs(xs - vp_point.x) + s3;
        uv.x = xs - vp_point.x;
        color = texture( segment_left, uv );
    } else {
        uv.x = (xs - vp_point.x) / abs(ys - vp_point.y) + s4;
        uv.y = ys - vp_point.y;
        color = texture( segment_bottom, uv );
    }
}