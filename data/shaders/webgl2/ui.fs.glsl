#version 300 es
precision mediump float;

out vec4 FragColor;

in vec4 v_color;
in vec2 v_texcoord;

uniform sampler2D u_texture;

void main() {
	vec4 color = texture(u_texture, v_texcoord);
	FragColor = color * v_color;
}
