#version 330 core

in vec2 a_pos;
in vec4 a_color;

out vec4 v_color;

uniform mat3x2 u_transform;
uniform vec4 u_color;
uniform float u_gamma;

void main()
{
	v_color = pow(a_color, vec4(u_gamma, u_gamma, u_gamma, 1.0)) * u_color;
	gl_Position = vec4(u_transform * vec3(a_pos, 1.0), 0.0, 1.0);
}
