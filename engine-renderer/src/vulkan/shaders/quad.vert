#version 450

layout(location = 0) in vec2 aPos;
layout(location = 1) in vec2 aUV;
layout(location = 2) in vec4 aColor;

layout(push_constant) uniform PushConstants {
    mat4 projection;
    float use_texture;
    float is_text;
} pc;

layout(location = 0) out vec2 uv;
layout(location = 1) out vec4 vColor;

void main()
{
    gl_Position = pc.projection * vec4(aPos, 0.0, 1.0);

    gl_Position.y = -gl_Position.y;

    uv = aUV;
    vColor = aColor;
}
