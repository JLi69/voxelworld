#version 330 core

out vec4 color;

uniform sampler2D tex;
in vec2 tc;
uniform float alpha;
uniform vec2 texscale;
uniform vec2 texoffset;
uniform vec2 texperc;

void main() {
	if(tc.x > texperc.x || 1.0 - tc.y > texperc.y)
		discard;

	vec2 transformedTc = tc * texscale + texoffset;
	color = texture(tex, transformedTc);
	color.a *= alpha;
}
