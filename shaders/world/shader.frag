#version 450

layout(binding = 1) uniform sampler2D texSampler;

layout(location = 0) in vec2 fragUv;

layout(location = 0) out vec4 outColor;

void main() {
    outColor = vec4(0.01, fragUv, 1);
}