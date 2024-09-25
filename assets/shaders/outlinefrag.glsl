/*
	Very simple fragment shader, for drawing an outline
*/

#version 330 core

in vec3 fragpos;
uniform vec4 incolor;
uniform float outlinesz;
out vec4 color;

bool atEdge(float x) {
	return fract(x) < outlinesz || fract(x) > 1.0 - outlinesz;
}

bool outline() {
	return 
		(atEdge(fragpos.x) || atEdge(fragpos.y)) &&
		(atEdge(fragpos.x) || atEdge(fragpos.z)) &&
		(atEdge(fragpos.y) || atEdge(fragpos.z));
}

void main() {
	color = incolor;
	color.a = float(outline());

	if(color.a < 0.5)
		discard;
}
