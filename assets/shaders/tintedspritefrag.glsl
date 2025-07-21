#version 330 core

out vec4 color;

uniform sampler2D tex;
in vec2 tc;
uniform vec4 tint;
uniform vec2 texscale;
uniform vec2 texoffset;

uniform vec3 campos;
uniform float fogdist;
uniform float fogstrength;
uniform vec4 fogcolor;

in vec3 fragpos;

void main() {
	vec2 transformedTc = tc * texscale + texoffset;
	color = texture(tex, transformedTc) * tint;

	float alpha = color.a;
	float mixamt = min(max(length(fragpos - campos) - fogdist, 0.0) * fogstrength, 1.0);
	color = mix(color, fogcolor, mixamt);
	color.a = alpha;

	if(color.a < 0.1)
		discard;
}
