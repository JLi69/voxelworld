/*
	Very simple fragment shader, for drawing an outline
*/

#version 330 core

uniform sampler2D breakingtexture;
uniform uint frame;
const uint FRAMES = 8u;
const float TEX_FRACT = 1.0 / float(FRAMES);

in vec3 fragpos;
uniform vec4 incolor;
uniform float outlinesz;
out vec4 color;
uniform vec3 scale;
in vec3 untransformedpos;
in float z;

bool atEdge(float x, float s) {
	float depth = min(z, 1.0);
	return fract(x) < outlinesz * depth / s || fract(x) > 1.0 - outlinesz * depth / s;
}

bool outline() {
	return 
		(atEdge(untransformedpos.x, scale.x) || atEdge(untransformedpos.y, scale.y)) &&
		(atEdge(untransformedpos.x, scale.x) || atEdge(untransformedpos.z, scale.z)) &&
		(atEdge(untransformedpos.y, scale.y) || atEdge(untransformedpos.z, scale.z));
}

vec2 getTc() {
	return
		fract(vec2(fragpos.x, fragpos.y)) * float(fract(untransformedpos.z) == 0.0) +
		fract(vec2(fragpos.x, fragpos.z)) * float(fract(untransformedpos.y) == 0.0) +
		fract(vec2(fragpos.y, fragpos.z)) * float(fract(untransformedpos.x) == 0.0);
}

void main() {
	color = incolor;
	color.a = float(outline());

	vec2 tc = getTc() * vec2(TEX_FRACT, 1.0) + vec2(TEX_FRACT * float(frame), 0.0);
	color += texture(breakingtexture, tc);

	if(color.a < 0.1)
		discard;
}
