#version 450

layout(binding = 0, set = 1) uniform sampler2D texSampler;

layout(location = 0) in vec2 uv;

layout(location = 0) out vec4 outColor;  

void main() {
    outColor = textureLod(texSampler, uv, 0);
}