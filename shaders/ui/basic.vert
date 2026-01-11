#version 460

layout(binding = 0) uniform UniformBufferObject {
    mat4 view_proj;
} ubo;

layout(location = 0) in vec4 inColor;
layout(location = 1) in vec4 inBorderColor;
layout(location = 2) in uvec4 inBorder;
layout(location = 3) in int x;
layout(location = 4) in int y;
layout(location = 5) in int width;
layout(location = 6) in int height;
layout(location = 7) in uint inCorner; 
layout(location = 8) in int z_index;

layout(location = 0) out vec2 fragTexCoord;
layout(location = 1) out flat vec4 fragColor;
layout(location = 2) out flat vec4 fragBorderColor;
layout(location = 3) out flat float fragWidth;
layout(location = 4) out flat float fragHeight;
layout(location = 5) out flat vec4 fragBorder;
layout(location = 6) out flat float fragCorner;

void main() {
    vec2 uv = vec2(((gl_VertexIndex << 1) & 2) >> 1, (gl_VertexIndex & 2) >> 1);
    gl_Position = ubo.view_proj * vec4(vec2(x, y) + vec2(width, height) * uv, z_index, 1.0);
    
    fragTexCoord = uv * vec2(width, height);
    fragColor = inColor;
    fragBorderColor = inBorderColor;
    fragWidth = width;
    fragHeight = height;
    fragBorder = inBorder;
    fragCorner = inCorner;
}