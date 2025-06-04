#version 450

layout(binding = 0) uniform UniformBufferObject {
    mat4 view_proj;
} ubo;

layout(location = 0) in vec4 inColor;
layout(location = 1) in vec2 inPos;
layout(location = 2) in vec2 inSize;
layout(location = 3) in uint inUvStart;
layout(location = 4) in uint inUvEnd;

layout(location = 0) out vec2 fragUv;
layout(location = 1) out vec4 fragColor;
layout(location = 2) out uint fragUvStart;
layout(location = 3) out uint fragUvEnd;

void main() {
    vec2 uv = vec2(((gl_VertexIndex << 1) & 2) >> 1, (gl_VertexIndex & 2) >> 1);
    gl_Position = ubo.view_proj * vec4(inPos + inSize * uv, 0.0, 1.0);
    
    fragUv = uv;
    fragColor = inColor;
    fragUvStart = inUvStart;
    fragUvEnd = inUvEnd;
}