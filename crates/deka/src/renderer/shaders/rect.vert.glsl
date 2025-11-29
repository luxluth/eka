#version 450

layout(location = 0) in vec3 position;
layout(location = 1) in vec4 color;
layout(location = 2) in vec2 uv;

layout(location = 0) out vec4 v_color;
layout(location = 1) out vec2 v_uv;

layout(push_constant) uniform PushConstants {
    vec2 screen_size;
} pc;

void main() {
    float x = (position.x / pc.screen_size.x) * 2.0 - 1.0;
    float y = (position.y / pc.screen_size.y) * 2.0 - 1.0;

    gl_Position = vec4(x, y, position.z, 1.0);
    v_color = color;
    v_uv = uv;
}
