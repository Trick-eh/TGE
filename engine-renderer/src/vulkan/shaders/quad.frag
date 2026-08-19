#version 450

layout(location = 0) in vec2 uv;
layout(location = 1) in vec4 vColor;

layout(set = 0, binding = 0) uniform sampler2D tex;

layout(push_constant) uniform PushConstants {
    mat4 projection;
    float use_texture;
    float is_text;
} pc;

layout(location = 0) out vec4 FragColor;

void main()
{
    if (pc.use_texture > 0.5) {
        if (pc.is_text > 0.5) {
            vec2 text_uv = vec2(uv.x, 1.0 - uv.y);
            float alpha = texture(tex, text_uv).r;
            FragColor = vec4(vColor.rgb, vColor.a * alpha);
        } else {
            FragColor = texture(tex, vec2(uv.x, 1.0 - uv.y)) * vColor;
        }
    } else {
        FragColor = vColor;
    }
}
