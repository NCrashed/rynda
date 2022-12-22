#version 440 
layout(local_size_x = 1, local_size_y = 1, local_size_z = 1) in;

uniform uvec3 volume_size;
uniform int mode;

struct PointerColumn{
    uint pointer;
    uint fields; // unpacked fields
};

layout (shared, binding = 0) readonly buffer InputData {
    PointerColumn columns[];
};

layout (rgba8, binding = 1) uniform image2D img_output;

uint rle_count(uint fields) {
    return fields & uint(0xFFFF);
}

uint skipped(uint fields) {
    return (fields >> 16) & uint(0x3FF);
}

uint drawn(uint fields) {
    return (fields >> 26) & uint(0x3F);
}

uint flat_index(uvec2 pos)
{
    return pos.x + pos.y * volume_size.x;
}

void main() {
    uvec2 cell_coord = uvec2(gl_GlobalInvocationID.xy);

    vec4 pixel = vec4(0.0, 0.0, 0.0, 1.0);
    
    PointerColumn pcol = columns[flat_index(cell_coord)];
    float height = 0.0; 
    
    if (mode == 0) {
        height = float(drawn(pcol.fields)) / float(volume_size.y);
    } else {
        height = float(skipped(pcol.fields)) / float(volume_size.y);
    }
    
    pixel.r = height;
    pixel.g = height;
    pixel.b = height;

    imageStore(img_output, ivec2(cell_coord), pixel);
}