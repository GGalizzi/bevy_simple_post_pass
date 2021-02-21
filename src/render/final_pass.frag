#version 450

layout(location = 0) in vec2 v_Uv;
layout(location = 1) in vec4 gl_FragCoord;

layout(location = 0) out vec4 o_Target;

layout(set = 1, binding = 0) uniform ColorMaterial_color {
    vec4 Color;
};

layout(set = 1, binding = 1) uniform texture2D ColorMaterial_texture;
layout(set = 1, binding = 2) uniform sampler ColorMaterial_texture_sampler;

void main() {
    vec4 color = Color;

    vec4 sampled_color = texture(
        sampler2D(ColorMaterial_texture, ColorMaterial_texture_sampler),
        v_Uv);
    
    if (sampled_color == vec4(0.0,0.0,1.0,1.0)) {
        sampled_color = vec4(0.25,0.1,0.1, 0.9);
    }

    float rem = mod(gl_FragCoord.y, 5.0);
    if (rem >= 0.0 && rem <= 1.1) {
        sampled_color += vec4(0.0012, 0.001, 0.0015, 0.0);
    }

    color *= sampled_color;
    o_Target = color;
}