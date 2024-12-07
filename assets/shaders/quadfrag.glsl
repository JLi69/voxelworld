#version 330 core

out vec4 color;

uniform sampler2D tex;
in vec2 tc;
uniform float alpha;

void main() {
	color = texture(tex, tc);
	color.a *= alpha;
}
