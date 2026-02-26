#version 460

layout(binding = 0, set = 1) uniform sampler2D texSampler;

layout(location = 0) in vec2 fragUv;
layout(location = 1) in flat vec4 fragColor;
layout(location = 2) in flat float fragScreenPxRange;

layout(location = 0) out vec4 outColor;

float median(float r, float g, float b) {
    return max(min(r, g), min(max(r, g), b));
}

const float WEIGHT = 0.5;

void main() {
    vec3 msd = textureLod(texSampler, fragUv, 0).rgb;
    float sd = median(msd.r, msd.g, msd.b);

    float screenPxDistance = fragScreenPxRange * (sd - WEIGHT);
    float alpha = clamp(screenPxDistance + WEIGHT, 0.0, 1.0);

    outColor = vec4(fragColor.rgb, fragColor.a * alpha);
}