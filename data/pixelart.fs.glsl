#version 330 core

out vec4 FragColor;

in vec4 v_color;
in vec2 v_texcoord;

uniform sampler2D u_tex;
uniform float u_pixel_bias; // 0.0 = no shift (original filtering), 1.0 = snap to nearest texel center (nearest-like)

void main() {
	// Push UVs toward the nearest texel center to reduce blurriness on pixel art
	// Compute the nearest texel center in UV space and blend toward it
	vec2 texSize = vec2(textureSize(u_tex, 0));
	vec2 uv_pix   = v_texcoord * texSize;
	vec2 uv_center = (floor(uv_pix) + 0.5) / texSize;
	vec2 uv_biased = mix(v_texcoord, uv_center, u_pixel_bias);

	vec4 color = texture(u_tex, uv_biased);
	if (color.a < 0.2) {
		discard;
	}

	color *= v_color;
	FragColor = color;
}
