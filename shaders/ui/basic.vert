#version 450

layout(binding = 0) uniform UniformBufferObject {
    mat4 view_proj;
} ubo;

layout(location = 0) in vec4 inColor;
layout(location = 1) in vec4 inBorderColor;
layout(location = 2) in float inBorder;
layout(location = 3) in float x;
layout(location = 4) in float y;
layout(location = 5) in float width;
layout(location = 6) in float height;
layout(location = 7) in float inCorner;

layout(location = 0) out vec2 fragTexCoord;
layout(location = 1) out vec4 fragColor;
layout(location = 2) out vec4 fragBorderColor;
layout(location = 3) out float fragWidth;
layout(location = 4) out float fragHeight;
layout(location = 5) out float fragBorder;
layout(location = 6) out float fragCorner;

void main() {
    vec2 uv = vec2(((gl_VertexIndex << 1) & 2) >> 1, (gl_VertexIndex & 2) >> 1);
    gl_Position = ubo.view_proj * vec4(vec2(x, y) + vec2(width, height) * uv, 0.0, 1.0);
    fragTexCoord = uv;
    fragColor = inColor;
    fragBorderColor = inBorderColor;
    fragWidth = width;
    fragHeight = height;
    fragBorder = inBorder;
    fragCorner = inCorner;
}