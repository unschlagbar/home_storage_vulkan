#version 460

layout(location = 0) in vec2 fragTexCoord;
layout(location = 1) in vec2 fragBoxHalf;
layout(location = 2) in flat vec4 fragColor;
layout(location = 3) in flat float fragblur;
layout(location = 4) in flat float fragCorner;

layout(location = 0) out vec4 outColor;

/* Signed distance to rounded box */
float roundedBoxSDF(vec2 p, vec2 b, float r) {
    vec2 q = abs(p) - b + r;
    return min(max(q.x, q.y), 0.0) + length(max(q, 0.0)) - r;
}

void main() {
    float corner = fragCorner;

    // Clamp corner to valid range
    corner = min(corner, min(fragBoxHalf.x, fragBoxHalf.y));

    // Center box inside padded quad
    vec2 boxCenter = fragBoxHalf + vec2(fragblur);
    vec2 p = fragTexCoord - boxCenter;

    // Distance field
    float dist = roundedBoxSDF(p, fragBoxHalf, corner);

    // Blur falloff
    float alpha = 1.0 - clamp(dist / fragblur, 0.0, 1.0);

    // CSS-like softness
    alpha *= alpha;

    if (alpha < 0.01) {
        discard;
    }

    outColor = vec4(fragColor.rgb, fragColor.a * alpha);
}
