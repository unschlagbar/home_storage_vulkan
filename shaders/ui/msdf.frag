#version 460

layout(binding = 0, set = 1) uniform sampler2D texSampler;

layout(location = 0) in vec2 uv;
layout(location = 1) in flat vec4 color;

layout(location = 0) out vec4 outColor;  

float median(float r, float g, float b) {
    return max(min(r, g), min(max(r, g), b));
}

const float threshold = 0.18;
const float smoothness = 0.18;

void main() {
    vec3 tx = textureLod(texSampler, uv, 0).rgb;
    float signd = median(tx.r, tx.g, tx.b);
    outColor = mix(threshold - smoothness, threshold + smoothness, signd) * color;
}