#version 330 core

in vec2 a_pos;
in vec2 a_texcoord;
in vec4 a_color;

out vec4 v_color;
out vec2 v_texcoord;

uniform mat3x2 u_transform;
uniform vec4 u_color;
uniform float u_gamma;

vec3 srgbToLinear(vec3 c) {
	return mix(c / 12.92, pow((c + 0.055) / 1.055, vec3(2.4)), step(0.04045, c));
}

vec4 srgbToLinear(vec4 c) {
	return vec4(srgbToLinear(c.rgb), c.a);
}

void main() {
	v_color = srgbToLinear(a_color) * u_color;
	v_texcoord = a_texcoord;
	gl_Position = vec4(u_transform * vec3(a_pos, 1.0), 0.0, 1.0);
}
