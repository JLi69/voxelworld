#version 330 core

layout(location = 0) in vec4 pos;

uniform mat4 persp;
uniform mat4 view;
uniform mat4 transform;

out vec3 fragpos;

void main() {
	vec4 glpos = persp * view * transform * pos;
	gl_Position = glpos.xyww;
	fragpos = pos.xyz;
}
