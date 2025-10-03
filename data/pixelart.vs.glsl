#version 330 core

in vec3 a_pos;
in vec2 a_texcoord;
in vec4 a_color;

out vec4 v_color;
out vec2 v_texcoord;

uniform sampler2D u_tex;
uniform mat4x4 u_transform;

void main()
{
	v_color = a_color;
	v_texcoord = a_texcoord / textureSize(u_tex, 0);
	gl_Position = u_transform * vec4(a_pos, 1.0);
}
