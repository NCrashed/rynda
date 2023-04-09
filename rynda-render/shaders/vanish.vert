#version 150
in vec2 position;
in float segment;
out vec2 tex_coords;
out float segment_frag;

uniform mat4 MVP;

const vec2 madd=vec2(0.5,0.5);

void main() {
    gl_Position = MVP*vec4(position, 0.0, 1.0);
    vec2 tex_pos = position.xy;
    tex_pos.y *= -1;
    tex_coords = tex_pos.xy*madd+madd;
    segment_frag = segment;
}