#version 330 core

layout(location = 0) in vec4 pos;
out vec2 tc;

void main() {
	gl_Position = pos;
	tc = pos.xy * 0.5 + vec2(0.5, 0.5);
}
