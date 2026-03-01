#version 460

layout(binding = 0) uniform UniformBufferObject {
    mat4 view_proj;
};

layout(location = 0) in vec4 color;
layout(location = 1) in vec2 pos;
layout(location = 2) in vec2 size;
layout(location = 3) in vec2 uvStart;
layout(location = 4) in vec2 uvEnd;

layout(location = 0) out vec2 fragUv;
layout(location = 1) out flat vec4 fragColor;
layout(location = 2) out flat float fragScreenPxRange;

void main() {
    vec2 uv = vec2(((gl_VertexIndex << 1) & 2) >> 1, (gl_VertexIndex & 2) >> 1);
    gl_Position = view_proj * vec4(pos + size * uv, 1.0, 1.0);

    fragUv = mix(uvStart, uvEnd, uv);
    fragColor = color;
    fragScreenPxRange = (size.y / (uvEnd.y - uvStart.y)) * 4.0;
}