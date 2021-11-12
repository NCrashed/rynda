#version 440
layout(local_size_x = 1, local_size_y = 1, local_size_z = 1) in;

uniform vec3 vanish_point;
uniform mat4 camera_mat;

layout (rgba8, binding = 1) uniform image2D img_output;

void main() {
    uvec2 cell_coord = uvec2(gl_GlobalInvocationID.xy);
    ivec2 image_size = imageSize(img_output);

    vec4 pixel = vec4(0.0, 0.0, 0.0, 1.0);
    vec4 vanish_cam = camera_mat * vec4(vanish_point, 1.0);
    vanish_cam.x = vanish_cam.x / vanish_cam.w;
    vanish_cam.y = vanish_cam.y / vanish_cam.w;
    vanish_cam.z = 0.0;

    vanish_cam.x = (vanish_cam.x + 1) * float(image_size.x) * 0.5;
    vanish_cam.y = (-vanish_cam.y + 1) * float(image_size.y) * 0.5;

    float dist = distance(vec4(cell_coord, 0.0, 0.0), vanish_cam) / 1024;

    pixel.r = dist;
    pixel.g = dist;
    pixel.b = dist;

    imageStore(img_output, ivec2(cell_coord), pixel);
}