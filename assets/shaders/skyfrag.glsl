#version 330 core

out vec4 color;

uniform sampler2D tex;
in vec2 tc;
uniform float alpha;
uniform vec2 tcScale;
uniform vec2 tcOffset;
uniform vec4 tint;

void main() {
	color = texture(tex, tc * tcScale + tcOffset) * tint;
	color.a *= alpha;
}
