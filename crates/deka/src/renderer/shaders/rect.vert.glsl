#version 450

layout(location = 0) in vec3 position;
layout(location = 1) in vec4 color;
layout(location = 2) in vec2 uv;
layout(location = 3) in vec2 size;
layout(location = 4) in float radius;
layout(location = 5) in float stroke_width;

layout(location = 0) out vec4 v_color;
layout(location = 1) out vec2 v_uv;
layout(location = 2) out vec2 v_size;
layout(location = 3) out float v_radius;
layout(location = 4) out float v_stroke_width;

layout(push_constant) uniform PushConstants {
    vec2 screen_size;
} pc;

void main() {
    float x = (position.x / pc.screen_size.x) * 2.0 - 1.0;
    float y = (position.y / pc.screen_size.y) * 2.0 - 1.0;

    gl_Position = vec4(x, y, 0.0, 1.0);
    v_color = color;
    v_uv = uv;
    v_size = size;
    v_radius = radius;
    v_stroke_width = stroke_width;
}
