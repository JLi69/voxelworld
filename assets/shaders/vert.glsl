/*
	Very simple 3D vertex shader, does not account for normals or texture coordinates
*/

#version 330 core

layout(location = 0) in vec4 position;

uniform mat4 persp;
uniform mat4 view;
uniform mat4 transform;

out vec3 fragpos;
out vec3 untransformedpos;
out float z;

void main() {
	untransformedpos = position.xyz + vec3(0.5, 0.5, 0.5);
	z = (persp * view * transform * position).z;
	fragpos = (transform * position).xyz;
	gl_Position = persp * view * transform * position;
}
