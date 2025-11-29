#version 450

layout(location = 0) in vec4 v_color;
layout(location = 1) in vec2 v_uv;
layout(location = 2) in vec2 v_size;
layout(location = 3) in float v_radius;
layout(location = 4) in float v_stroke_width;

layout(location = 0) out vec4 f_color;

// Standard SDF for a rounded box
// p: position relative to center
// b: half-extents (width/2, height/2)
// r: corner radius
float sdRoundedBox(vec2 p, vec2 b, float r) {
    vec2 q = abs(p) - b + r;
    return min(max(q.x, q.y), 0.0) + length(max(q, 0.0)) - r;
}

void main() {
    // Calculate pixel position from UV (0..1) -> (0..width, 0..height)
    // We center it by subtracting size/2
    vec2 pos = (v_uv * v_size) - (v_size * 0.5);
    vec2 half_size = v_size * 0.5;

    // Distance to the edge of the rounded box
    // dist <= 0 is inside, dist > 0 is outside
    float dist = sdRoundedBox(pos, half_size, v_radius);

    // Smoothstep for anti-aliasing (approx 1.0 pixel width)
    // We use a width of 0.5 on each side of the threshold

    float alpha;

    if (v_stroke_width > 0.0) {
        // STROKE RENDER
        // Valid pixels are between dist = 0 (outer edge) and dist = -v_stroke_width (inner edge)

        // Outer edge alpha (fades out as dist > 0)
        float outer_alpha = 1.0 - smoothstep(-0.5, 0.5, dist);

        // Inner edge alpha (fades out as dist < -v_stroke_width)
        // We want full alpha when dist > -width
        float inner_alpha = smoothstep(-v_stroke_width - 0.5, -v_stroke_width + 0.5, dist);

        alpha = outer_alpha * inner_alpha;
    } else {
        // FILL RENDER
        // Valid pixels are dist <= 0
        alpha = 1.0 - smoothstep(-0.5, 0.5, dist);
    }

    if (alpha <= 0.0) {
        discard;
    }

    f_color = vec4(v_color.rgb, v_color.a * alpha);
}
