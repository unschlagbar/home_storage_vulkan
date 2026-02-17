#version 460

layout(binding = 0) uniform UniformBufferObject {
    mat4 view_proj;
};

layout(location = 0) in vec4 inColor;
layout(location = 1) in int x;
layout(location = 2) in int y;
layout(location = 3) in int width;
layout(location = 4) in int height;
layout(location = 5) in uint blur;
layout(location = 6) in uint corner; 

layout(location = 0) out vec2 fragTexCoord;
layout(location = 1) out vec2 fragBoxHalf;
layout(location = 2) out flat vec4 fragColor;
layout(location = 3) out flat float fragBlur;
layout(location = 4) out flat float fragCorner;

void main() {
    vec2 uv = vec2((gl_VertexIndex << 1 & 2) >> 1, (gl_VertexIndex & 2) >> 1);
    vec2 quadSize = vec2(width, height) + 2.0 * blur;
    
    fragTexCoord = uv * quadSize;
    fragBoxHalf = vec2(width, height) * 0.5;
    fragColor = inColor;
    fragBlur = blur;
    fragCorner = corner;

    gl_Position = view_proj * vec4(vec2(x - blur, y - blur) + quadSize * uv, 1.0, 1.0);
}