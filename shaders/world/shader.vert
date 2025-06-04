#version 460

layout(binding = 0) uniform UniformBufferObject {
    mat4 viewProj;
} ubo;

layout(location = 0) in vec3 inPosition;
layout(location = 1) in vec3 nrm;
layout(location = 2) in vec2 uv;
layout(location = 3) in float materialIndex;
layout(location = 4) in mat4 modelMatrix;

layout(location = 0) out vec2 FragUv;


void main() {
    //vec2 outUV = vec2((gl_VertexIndex << 1) & 2, gl_VertexIndex & 2);
    //gl_Position = ubo.viewProj * vec4(outUV * inSize + inPosition, 0.0, 1.0);
    gl_Position = ubo.viewProj * modelMatrix * vec4(inPosition, 1);
    FragUv = uv;
}