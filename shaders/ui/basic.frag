#version 450

layout(binding = 1) uniform sampler2D texSampler[2];

layout(location = 0) in vec2 fragTexCoord;
layout(location = 1) in flat vec4 inColor;
layout(location = 2) in flat vec4 fragBorderColor;
layout(location = 3) in flat float fragWidth;  // Instanz-Daten
layout(location = 4) in flat float fragHeight; // Instanz-Daten
layout(location = 5) in flat float border;     // Instanz-Daten
layout(location = 6) in flat float corner;     // Instanz-Daten

layout(location = 0) out vec4 outColor;

void main() {

    vec3 color = inColor.rgb;

    vec2 uv = fragTexCoord * vec2(fragWidth, fragHeight);
    float alpha = 1.0;

    // Define the positions of the corners
    vec2 topLeft = vec2(corner, fragHeight - corner);
    vec2 topRight = vec2(fragWidth - corner, fragHeight - corner);
    vec2 bottomRight = vec2(fragWidth - corner, corner);
    vec2 bottomLeft = vec2(corner, corner);

    float antialiasWidth = 0.85;

    // Check for rounded corners and adjust alpha accordingly
    if (corner > 0.0) {
        float dist = 0;

        if (uv.y <= corner && uv.x <= corner) {
            dist = length(uv - bottomLeft);
        } else if (uv.y <= corner && uv.x >= fragWidth - corner) {
            dist = length(uv - bottomRight);
        } else if (uv.y >= fragHeight - corner && uv.x >= fragWidth - corner) {
            dist = length(uv - topRight);
        } else if (uv.y >= fragHeight - corner && uv.x <= corner) {
            dist = length(uv - topLeft);
        }

        if (dist > corner - antialiasWidth && dist <= corner) {
            alpha = smoothstep(corner, corner - antialiasWidth, dist);
        } else if (dist > corner) {
            discard;
        }
    }

    if (border > 0.0 && alpha != 0.0) {
        if (uv.x <= border || uv.x >= fragWidth - border || uv.y <= border || uv.y >= fragHeight - border) {
            color = fragBorderColor.rgb;
        } else if (corner != 0.0) {

            vec3 borderColor = fragBorderColor.rgb;
            float innerCorner = corner - border;
            float dist = 0.0;

            if (uv.y <= corner && uv.x <= corner) {
                dist = length(uv - bottomLeft);
            } else if (uv.y <= corner && uv.x >= fragWidth - corner) {
                dist = length(uv - bottomRight + 0.1);
            } else if (uv.y >= fragHeight - corner && uv.x >= fragWidth - corner) {
                dist = length(uv - topRight + 0.1);
            } else if (uv.y >= fragHeight - corner && uv.x <= corner) {
                dist = length(uv - topLeft + 0.1);
            }

            if (dist > innerCorner - antialiasWidth && dist <= innerCorner) {
                float mixFactor = smoothstep(innerCorner, innerCorner - antialiasWidth, dist);
                color = mix(borderColor, color, mixFactor);
                alpha *= mix(fragBorderColor.a, inColor.a, mixFactor);
            } else if (dist > innerCorner) {
                color = borderColor;
                alpha *= fragBorderColor.a;
            } else {
                alpha *= inColor.a;
            }
        }
    }
    outColor = vec4(color, alpha);
}

    //Text
    //} else if (mode == 1) {
    //    uint bits0 = floatBitsToUint(border); // Konvertiere den float in seine Bit-Darstellung
    //    uint uv_width = (bits0 >> 16) & 0xFFFFu;  // Die oberen 16 Bit
    //    uint uv_x = bits0 & 0xFFFFu;

    //    uint bits1 = floatBitsToUint(corner); // Konvertiere den float in seine Bit-Darstellung
    //    uint uv_height = (bits1 >> 16) & 0xFFFFu;  // Die oberen 16 Bit
    //    uint uv_y = bits1 & 0xFFFFu;

    //    vec2 uv = vec2( mix(uv_x, uv_x + uv_width, fragTexCoord.r),  mix(uv_y, uv_y + uv_height, fragTexCoord.g)) / 128;
    //    float texture = texture(texSampler[0], uv).r;
    //    outColor = vec4(color * texture, texture);

