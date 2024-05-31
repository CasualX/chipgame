#version 330 core

layout (location = 0) in vec3 aPos;
layout (location = 1) in vec2 aTexCoord;
layout (location = 2) in vec4 aColor;

out vec4 VertexColor;
out vec2 TexCoord;

uniform mat4x4 transform;

void main()
{
	VertexColor = aColor;
	TexCoord = aTexCoord;
	gl_Position = transform * vec4(aPos, 1.0);
}
