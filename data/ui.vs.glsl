#version 330 core

layout (location = 0) in vec2 aPos;
layout (location = 1) in vec2 aTexCoord;
layout (location = 2) in vec4 aColor;

out vec4 VertexColor;
out vec2 TexCoord;

uniform mat3x2 transform;
uniform vec4 color;

void main()
{
	VertexColor = aColor * color;
	TexCoord = aTexCoord;
	gl_Position = vec4(transform * vec3(aPos, 1.0), 0.0, 1.0);
}
