/*
	Very simple fragment shader, for drawing an outline
*/

#version 330 core

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

void main() {
	color = incolor;
	color.a = float(outline());

	if(color.a < 0.5)
		discard;
}
