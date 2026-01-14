#version 460
layout(location = 0) in vec2 uv;
layout(location = 1) in flat vec4 inColor;
layout(location = 2) in flat vec4 fragBorderColor;
layout(location = 3) in flat float fragWidth;
layout(location = 4) in flat float fragHeight;
layout(location = 5) in flat vec4 border;   // left, top, right, bottom
layout(location = 6) in flat float corner;
layout(location = 0) out vec4 outColor;

void main()
{
    const float AA  = 0.85;
    const float INV_AA = 1.0 / AA;
    const float EPS = 0.001;
    
    // STRAIGHT BORDER (cheap, early out für die meisten Pixel)
    float maxBorder = max(max(border.x, border.y), max(border.z, border.w));
    bool hasBorder = maxBorder > EPS;
    
    if (hasBorder) {
        vec4 onBorderMask = vec4(
            step(uv.x, border.x),
            step(fragWidth  - border.z, uv.x),
            step(uv.y, border.y),
            step(fragHeight - border.w, uv.y)
        );
        float onStraightBorder = step(EPS, dot(onBorderMask, vec4(1.0)));
        outColor = mix(inColor, fragBorderColor, onStraightBorder);
    } else {
        outColor = inColor;
    }
    
    // NO CORNERS → DONE (early exit für die meisten Pixel)
    if (corner < 1.0)
        return;
    
    // CORNER MASK (schneller Check ob wir überhaupt in einer Ecke sind)
    vec2 cornerDist = min(uv, vec2(fragWidth, fragHeight) - uv);
    if (cornerDist.x >= corner || cornerDist.y >= corner)
        return;
    
    // Welche Ecke?
    vec4 inCornerMask = vec4(
        step(uv.y, corner) * step(uv.x, corner),
        step(uv.y, corner) * step(fragWidth  - corner, uv.x),
        step(fragHeight - corner, uv.y) * step(fragWidth  - corner, uv.x),
        step(fragHeight - corner, uv.y) * step(uv.x, corner)
    );
    
    // CORNER CENTER
    vec4 cornerX = vec4(corner, fragWidth - corner, fragWidth - corner, corner);
    vec4 cornerY = vec4(corner, corner, fragHeight - corner, fragHeight - corner);
    vec2 cornerCenter = vec2(
        dot(cornerX, inCornerMask),
        dot(cornerY, inCornerMask)
    );
    
    // DISTANCE
    vec2 d = uv - cornerCenter;
    float dist2 = dot(d, d);
    float outerR2 = corner * corner;
    
    // Early discard (spart sqrt!)
    if (dist2 > outerR2)
        discard;
    
    float dist = sqrt(dist2);
    
    // OUTER AA
    float outerAA = (corner - dist) * INV_AA;
    
    // INNER CORNER BORDER
    if (hasBorder) {
        // Welcher Border-Wert für diese Ecke?
        vec4 borderWidths = vec4(
            max(border.x, border.y),
            max(border.z, border.y),
            max(border.z, border.w),
            max(border.x, border.w)
        );
        float borderWidth = dot(borderWidths, inCornerMask);
        float borderMix = clamp((corner - dist - borderWidth) * INV_AA, 0.0, 1.0);
            
        outColor = mix(fragBorderColor, inColor, borderMix);
    }
    
    outColor.a *= outerAA;
}