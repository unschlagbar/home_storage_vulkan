#version 450

layout(binding = 0) uniform UniformBufferObject {
    mat4 view_proj;
} ubo;

layout(location = 0) in float x;
layout(location = 1) in float y;
layout(location = 2) in float width;
layout(location = 3) in float height;
layout(location = 4) in uint uvStart;
layout(location = 5) in uint uvEnd;
layout(location = 6) in float z_index;

layout(location = 0) out vec2 fragTexCoord;
layout(location = 1) out float fragWidth;
layout(location = 2) out float fragHeight;
layout(location = 3) out uint fragUvStart;
layout(location = 4) out uint fragUvEnd;

void main() {
    vec2 uv = vec2(((gl_VertexIndex << 1) & 2) >> 1, (gl_VertexIndex & 2) >> 1);
    gl_Position = ubo.view_proj * vec4(vec2(x, y) + vec2(width, height) * uv, -z_index, 1.0);
    
    fragTexCoord = uv;
    fragWidth = width;
    fragHeight = height;
    fragUvStart = uvStart;
    fragUvEnd = uvEnd;
}