#version 330 core

out vec4 color;

uniform sampler2D tex;
in vec2 tc;
uniform float alpha;
uniform float texscale;
uniform vec2 texoffset;

void main() {
	vec2 transformedTc = tc * texscale + texoffset;
	color = texture(tex, transformedTc);
	color.a *= alpha;
}
