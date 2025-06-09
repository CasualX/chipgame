#version 330 core

in vec3 a_pos;
in vec2 a_texcoord;
in vec4 a_color;

out vec4 v_color;
out vec2 v_texcoord;

uniform mat4x4 transform;

void main()
{
	v_color = a_color;
	v_texcoord = a_texcoord;
	gl_Position = transform * vec4(a_pos, 1.0);
}
