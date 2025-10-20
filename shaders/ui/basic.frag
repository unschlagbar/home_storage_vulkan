#version 450

layout(location = 0) in vec2 uv;
layout(location = 1) in flat vec4 inColor;
layout(location = 2) in flat vec4 fragBorderColor;
layout(location = 3) in flat float fragWidth;
layout(location = 4) in flat float fragHeight;
layout(location = 5) in flat vec4 border;
layout(location = 6) in flat float corner;

layout(location = 0) out vec4 outColor;

void main() {
    outColor = inColor;
    const float antialiasWidth = 0.85;

    vec4 cornerX = vec4(corner, fragWidth - corner, fragWidth - corner, corner);
    vec4 cornerY = vec4(corner, corner, fragHeight - corner, fragHeight - corner);

    vec4 inCornerMask = vec4(
        step(uv.y, corner) * step(uv.x, corner),
        step(uv.y, corner) * step(fragWidth - corner, uv.x),
        step(fragHeight - corner, uv.y) * step(fragWidth - corner, uv.x),
        step(fragHeight - corner, uv.y) * step(uv.x, corner)
    );

    float cornerActive = step(1, corner);

    vec2 cornerCenter = vec2(dot(cornerX, inCornerMask), dot(cornerY, inCornerMask));
    
    float dist = length(uv - cornerCenter) * step(0.001, dot(inCornerMask, vec4(1.0)));

    float aaStart = corner - antialiasWidth;
    float smoothOuter = smoothstep(aaStart, corner, dist);
    outColor.a *= mix(1.0, 1.0 - smoothOuter, cornerActive);
    if (dist > corner && cornerActive > 0.0) discard;

    float maxBorder = max(max(border.x, border.y), max(border.z, border.w));
    float borderActive = step(0.001, maxBorder);

    vec4 onBorderMask = vec4(
        step(uv.x, border.x),
        step(fragWidth - border.z, uv.x),
        step(uv.y, border.y),
        step(fragHeight - border.w, uv.y)
    );

    float onStraightBorder = step(0.001, dot(onBorderMask, vec4(1.0))); // any(onBorderMask)
    outColor.rgb = mix(outColor.rgb, fragBorderColor.rgb, borderActive * onStraightBorder);

    float inCorner = cornerActive * borderActive * step(0.001, dot(inCornerMask, vec4(1.0)));

    if (inCorner > 0.0) {
        vec3 borderColor = fragBorderColor.rgb;

        vec4 effBorders = vec4(
            (border.x + border.y) * 0.5,
            (border.z + border.y) * 0.5,
            (border.z + border.w) * 0.5,
            (border.x + border.w) * 0.5
        );

        float eff_border = dot(effBorders, inCornerMask);

        float localInnerCorner = max(0.0, corner - eff_border);

        float aaStartInner = localInnerCorner - antialiasWidth;
        float mixFactor = 1.0 - smoothstep(aaStartInner, localInnerCorner, dist);

        float isTransition = step(aaStartInner, dist) * (1.0 - step(localInnerCorner, dist));
        float isOuter = step(localInnerCorner, dist);
        float isInner = 1.0 - (isTransition + isOuter);

        outColor.rgb = mix(outColor.rgb, mix(borderColor, outColor.rgb, mixFactor), isTransition + isOuter);
        outColor.a *= mix(inColor.a, mix(fragBorderColor.a, inColor.a, mixFactor), isTransition) + fragBorderColor.a * isOuter;
    }
}