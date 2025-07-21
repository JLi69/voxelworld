/*
	Assumes that we are drawing a quad
*/

#version 330 core

layout(location = 0) in vec4 position;

uniform mat4 persp;
uniform mat4 view;
uniform mat4 transform;

out vec2 tc;
out vec3 fragpos;

void main() {
	tc = position.xy * 0.5 + vec2(0.5, 0.5);
	vec4 untransformed = vec4(position.xy, 0.0, 1.0);
	untransformed.z -= 0.01 * gl_InstanceID;
	vec4 transformed = transform * untransformed;
	gl_Position = persp * view * transformed;
	fragpos = transformed.xyz;
}
