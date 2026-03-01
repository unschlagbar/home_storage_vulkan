#version 460

layout(binding = 0, set = 1) uniform sampler2D texSampler;

layout(location = 0) in vec2 fragUv;
layout(location = 1) in flat vec4 fragColor;
layout(location = 2) in flat float fragScreenPxRange;

layout(location = 0) out vec4 outColor;

float median(float r, float g, float b) {
    return max(min(r, g), min(max(r, g), b));
}

void main() {
    vec3 msd = textureLod(texSampler, fragUv, 0).rgb;
    float sd = median(msd.r, msd.g, msd.b);
    float alpha = fragScreenPxRange * sd * 1.5;

    outColor = vec4(fragColor.rgb, fragColor.a * alpha);
}