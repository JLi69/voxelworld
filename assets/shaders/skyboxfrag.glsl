#version 330 core

uniform vec3 topcolor;
uniform vec3 botcolor;

in vec3 fragpos;
out vec4 color;

void main() {
	float topdist = (1.0 - fragpos.y) / 2.0 - 0.15;
	float t = clamp((12.0 * pow(topdist - 0.5, 3.0)) + 0.5, 0.0, 1.0);
	color = vec4(mix(topcolor, botcolor, t), 1.0);
}
