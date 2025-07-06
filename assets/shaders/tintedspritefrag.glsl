#version 330 core

out vec4 color;

uniform sampler2D tex;
in vec2 tc;
uniform vec4 tint;
uniform vec2 texscale;
uniform vec2 texoffset;

void main() {
	vec2 transformedTc = tc * texscale + texoffset;
	color = texture(tex, transformedTc) * tint;
}
