#version 330 core

out vec4 FragColor;

in vec4 v_color;
in vec2 v_texcoord;

uniform sampler2D tex;

void main() {
	vec4 color = texture(tex, v_texcoord);
	if (color.a < 0.2) {
		discard;
	}

	color *= v_color;
	FragColor = color;
}
