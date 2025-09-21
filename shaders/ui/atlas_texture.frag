#version 450

layout(binding = 1) uniform sampler2D texSampler[1];

layout(location = 0) in vec2 fragTexCoord;
layout(location = 1) in flat float fragWidth;
layout(location = 2) in flat float fragHeight;
layout(location = 3) in flat uint uvStart;
layout(location = 4) in flat uint uvSize;

layout(location = 0) out vec4 outColor;

void main() {
    uint uv_x = uvStart & 0xffff;
    uint uv_y = (uvStart >> 16) & 0xffff;

    uint uv_x_size = uvSize & 0xffff;
    uint uv_y_size = (uvSize >> 16) & 0xffff;

    vec2 uv = vec2(mix(uv_x, uv_x + uv_x_size, fragTexCoord.x), mix(uv_y, uv_y + uv_y_size, fragTexCoord.y));

    outColor = texture(texSampler[0], uv);
}