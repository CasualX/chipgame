#version 330 core

in vec3 a_pos;
in vec2 a_texcoord;
in vec4 a_color;

out vec4 v_color;
out vec2 v_texcoord;
out vec3 v_worldpos;

uniform sampler2D u_tex;
uniform mat4x4 u_transform;

vec3 srgbToLinear(vec3 c) {
	return mix(c / 12.92, pow((c + 0.055) / 1.055, vec3(2.4)), step(0.04045, c));
}

vec4 srgbToLinear(vec4 c) {
	return vec4(srgbToLinear(c.rgb), c.a);
}

void main() {
	v_color = srgbToLinear(a_color);
	v_texcoord = a_texcoord / textureSize(u_tex, 0);
	v_worldpos = a_pos;
	gl_Position = u_transform * vec4(a_pos, 1.0);
}
