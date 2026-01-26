#version 300 es
precision mediump float;

out vec4 FragColor;

in vec4 v_color;

void main() {
	FragColor = v_color;
}
