#version 450

layout(binding = 0) uniform UniformBufferObject {
    mat4 view_proj;
} ubo;

layout(location = 0) in vec4 color;
layout(location = 1) in vec2 pos;
layout(location = 2) in vec2 size;
layout(location = 3) in uint uvStart;
layout(location = 4) in uint uvSize;
layout(location = 5) in float z_index;

layout(location = 0) out vec2 fragUv;
layout(location = 1) out flat vec4 fragColor;

void main() {
    vec2 uv = vec2(((gl_VertexIndex << 1) & 2) >> 1, (gl_VertexIndex & 2) >> 1);
    gl_Position = ubo.view_proj * vec4(pos + size * uv, z_index, 1.0);

    vec2 uv_start = vec2(uvStart & 0xffff, uvStart >> 16);
    vec2 uv_size = vec2(uvSize & 0xffff, uvSize >> 16);
    
    fragUv = mix(uv_start, uv_start + uv_size, uv);
    fragColor = color;
}