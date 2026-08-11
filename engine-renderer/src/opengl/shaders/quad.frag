#version 330 core
in vec2 uv;
in vec4 vColor;

uniform sampler2D tex;
uniform bool use_texture;
uniform bool is_text;

out vec4 FragColor;

void main()
{
    if (use_texture) {
        if (is_text) {
            vec2 text_uv = vec2(uv.x, 1 - uv.y);
            float alpha = texture(tex, text_uv).r;
            FragColor = vec4(vColor.rgb, vColor.a * alpha);
        } else {
            FragColor = texture(tex, vec2(uv.x, 1.0 - uv.y)) * vColor;
        }
    } else {
        FragColor = vColor;
    }
}
