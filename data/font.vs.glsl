#version 330 core
layout (location = 0) in vec2 aPos;
layout (location = 1) in vec2 aUV;
layout (location = 2) in vec4 aFG;
layout (location = 3) in vec4 aBG;

out vec2 v_texcoord;
out vec4 v_color;
out vec4 v_outline;

uniform mat3x2 u_transform;
uniform float u_gamma;

void main()
{
	v_texcoord = aUV;
	v_color = pow(aFG, vec4(u_gamma));
	v_outline = pow(aBG, vec4(u_gamma));
	gl_Position = vec4(u_transform * vec3(aPos, 1.0), 0.0, 1.0);
}
