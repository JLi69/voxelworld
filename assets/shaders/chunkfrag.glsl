#version 330 core

out vec4 color;
in vec3 fragpos;

uniform vec3 selected;
uniform bool selectedEmpty;

bool fragIsSelected() {
	return 
		abs(fragpos.x - (selected.x + 0.5)) <= 0.5 &&
		abs(fragpos.y - (selected.y + 0.5)) <= 0.5 &&
		abs(fragpos.z - (selected.z + 0.5)) <= 0.5 &&
		!selectedEmpty;
}

void main() {
	//Test color
	//TODO: add textures/shading
	color = vec4(fract(fragpos), 1.0);
	vec4 highlightColor = 
		color * (1.0 - float(fragIsSelected())) +
		vec4(1.0, 1.0, 1.0, 1.0) * float(fragIsSelected());
	color = mix(color, highlightColor, 0.5);
}
