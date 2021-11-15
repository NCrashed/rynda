#version 440
layout(local_size_x = 1, local_size_y = 1, local_size_z = 1) in;

uniform vec2 vanish_point;
uniform uint segment;
uniform uint np;
// uniform mat4 camera_mat;

layout (rgba8, binding = 1) uniform image2D img_output;

void main() {
    uint line_i = uint(gl_GlobalInvocationID.x);
    ivec2 size = imageSize(img_output);

    vec2 line_start;
    vec4 color1 = vec4(0.25, 0.1, 0.1, 1.0);
    vec4 color2 = vec4(0.5, 0.2, 0.2, 1.0);
    if (segment == 0) {
        line_start = vec2(size.x, vanish_point.y - float(np/2) + float(line_i));
        color1 = vec4(0.25, 0.1, 0.1, 1.0);
        color2 = vec4(0.5, 0.2, 0.2, 1.0);
    } else if (segment == 1) {
        line_start = vec2(vanish_point.x + float(np/2) - float(line_i), 0);
        if (vanish_point.y > size.y)
        {
            float dx = float(size.x * size.y) / (vanish_point.y - float(size.y));
            if (line_start.x < -dx || line_start.x > size.x + dx) return;
        }
        color1 = vec4(0.1, 0.25, 0.1, 1.0);
        color2 = vec4(0.25, 0.5, 0.25, 1.0);
    } else if (segment == 2) {
        line_start = vec2(0, vanish_point.y - float(np/2) + float(line_i));
        color1 = vec4(0.1, 0.1, 0.25, 1.0);
        color2 = vec4(0.25, 0.25, 0.5, 1.0);
    } else {
        line_start = vec2(vanish_point.x + float(np/2) - float(line_i), size.y);
        if (vanish_point.y < -size.y)
        {
            float dx = float(size.x * size.y) / (2 * (-vanish_point.y));
            if (line_start.x < -dx || line_start.x > size.x + dx) return;
        }
        color1 = vec4(0.1, 0.25, 0.25, 1.0);
        color2 = vec4(0.25, 0.5, 0.5, 1.0);
    }

    vec2 line_dir = normalize(vanish_point - line_start);
    float line_length = distance(line_start, vanish_point);
    vec2 current_point = line_start;

    vec4 pixel;
    vec4 other_pixel;
    if ((line_i / 40) % 2 == 0) {
        pixel = color1;
        other_pixel = color2;
    } else {
        pixel = color2;
        other_pixel = color1;
    }

    for(int i=0; i<uint(line_length); ++i) {
        ivec2 cell_coord = ivec2(current_point);
        imageStore(img_output, cell_coord, pixel);

        int curr_megacell;
        int next_megacell;
        if (segment % 2 == 0) {
            curr_megacell = (int(current_point.x) / 80) % 2;
            next_megacell = (int((current_point + line_dir).x) / 80) % 2;
        } else {
            curr_megacell = (int(current_point.y) / 80) % 2;
            next_megacell = (int((current_point + line_dir).y) / 80) % 2;
        }
        if (curr_megacell != next_megacell) {
            vec4 temp = pixel;
            pixel = other_pixel;
            other_pixel = temp;
        }

        current_point += line_dir;
        // if (current_point.y > float(size.y) || current_point.y < 0 || current_point.x > float(size.x) || current_point.x < 0) break;
    }
}