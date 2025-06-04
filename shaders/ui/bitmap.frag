#version 450

layout(binding = 1) uniform sampler2D texSampler[2];

layout(location = 0) in vec2 fragUv;          // Vertex Data
layout(location = 1) in flat vec4 fragColor;  // Instance Data
layout(location = 2) in flat uint fragUvStart;// Instance Data
layout(location = 3) in flat uint fragUvSize;  // Instance Data

layout(location = 0) out vec4 outColor;  

void main() {
    uint uv_x = fragUvStart & 0xffff;
    uint uv_y = (fragUvStart >> 16) & 0xffff;

    uint uv_x_size = fragUvSize & 0xffff;
    uint uv_y_size = (fragUvSize >> 16) & 0xffff;

    //vec2 uv = vec2(mix(uv_x, uv_x_size, fragUv.x) / 1.0, mix(uv_y, uv_y_size, fragUv.y) / 1.0);
    vec2 uv = vec2(mix(uv_x, uv_x + uv_x_size, fragUv.x), mix(uv_y, uv_y + uv_y_size, fragUv.y));

    float texture = texture(texSampler[0], uv).r;
    outColor = vec4(fragColor.rgb * texture, texture);
}