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

    // Pack corners into vec4s: separate x and y for all 4 corners (bottomLeft, bottomRight, topRight, topLeft)
    // Eliminates array and duplicates by vectorizing
    vec4 cornerX = vec4(corner, fragWidth - corner, fragWidth - corner, corner);
    vec4 cornerY = vec4(corner, corner, fragHeight - corner, fragHeight - corner);

    // Masken für Ecken-Bereiche (vektorisiert, 1.0 wenn in Area)
    vec4 inCornerMask = vec4(
        step(uv.y, corner) * step(uv.x, corner),
        step(uv.y, corner) * step(fragWidth - corner, uv.x),
        step(fragHeight - corner, uv.y) * step(fragWidth - corner, uv.x),
        step(fragHeight - corner, uv.y) * step(uv.x, corner)
    );

    float cornerActive = step(1, corner); // Maske für corner > 0.0 (vermeidet Branch)

    // Wähle cornerCenter vektorisiert ohne Loop/Array
    vec2 cornerCenter = vec2(dot(cornerX, inCornerMask), dot(cornerY, inCornerMask));
    
    // Äußere Distanz (nur wenn in Ecke)
    float dist = length(uv - cornerCenter) * step(0.001, dot(inCornerMask, vec4(1.0))); // any(inCornerMask) als dot

    float aaStart = corner - antialiasWidth;
    float smoothOuter = smoothstep(aaStart, corner, dist);
    outColor.a *= mix(1.0, 1.0 - smoothOuter, cornerActive);
    if (dist > corner && cornerActive > 0.0) discard;

    // Border-Handling
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

    // Innere Corner-Border (nur wenn corner und border aktiv und in Ecke)
    float inCorner = cornerActive * borderActive * step(0.001, dot(inCornerMask, vec4(1.0)));

    if (inCorner > 0.0) {
        vec3 borderColor = fragBorderColor.rgb;

        // Vektorisierte effBorders (bottomLeft, bottomRight, topRight, topLeft) – korrigiert
        vec4 effBorders = vec4(
            (border.x + border.y) * 0.5, // bottomLeft: left + bottom
            (border.z + border.y) * 0.5, // bottomRight: right + bottom
            (border.z + border.w) * 0.5, // topRight: right + top
            (border.x + border.w) * 0.5  // topLeft: left + top
        );

        float eff_border = dot(effBorders, inCornerMask); // Gewichtete Summe ohne Loop

        float localInnerCorner = max(0.0, corner - eff_border);

        float aaStartInner = localInnerCorner - antialiasWidth;
        float mixFactor = 1.0 - smoothstep(aaStartInner, localInnerCorner, dist);

        // Mathematische Zonen (transition, outer, inner)
        float isTransition = step(aaStartInner, dist) * (1.0 - step(localInnerCorner, dist));
        float isOuter = step(localInnerCorner, dist);
        float isInner = 1.0 - (isTransition + isOuter);

        outColor.rgb = mix(outColor.rgb, mix(borderColor, outColor.rgb, mixFactor), isTransition + isOuter);
        outColor.a *= mix(inColor.a, mix(fragBorderColor.a, inColor.a, mixFactor), isTransition) + fragBorderColor.a * isOuter;
    }
}