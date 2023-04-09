#version 330
uniform usampler2D pointermap;
in vec2 tex_coords;
layout(location = 0) out vec4 color;

uniform uvec3 volume_size;
uniform int mode;

uint pointer(uvec4 column) {
    return uint(column.r) + (uint(column.g) << 16);
}

uint rle_count(uvec4 column) {
    return column.b & uint(0xFFFF);
}

uint skipped(uvec4 column) {
    return column.a & uint(0x3FF);
}

uint drawn(uvec4 column) {
    return (column.a >> 10) & uint(0x3F);
}

void main() {
    uvec4 pcol = texture(pointermap, tex_coords);
    float height = 0.0; 
    
    if (mode == 0) {
        height = float(drawn(pcol)) / float(volume_size.y);
    } else {
        height = float(skipped(pcol)) / float(volume_size.y);
    }
    
    color.r = height;
    color.g = height;
    color.b = height;
}