#version 330 core

layout(location = 0) in vec4 pos;

uniform mat4 transform;
uniform mat4 screen;
out vec2 tc;

void main() {
	gl_Position = screen * transform * pos;
	tc = pos.xy * 0.5 + vec2(0.5, 0.5);
	tc.y = 1.0 - tc.y;
}
