#version 330 core

out vec4 FragColor;

in vec4 VertexColor;
in vec2 TexCoord;

uniform sampler2D tex;
uniform vec2 texSize;

void main() {
	vec4 color = texture(tex, TexCoord / texSize);
	if (color.a < 0.2) {
		discard;
	}

	color *= VertexColor;
	FragColor = color;
}
