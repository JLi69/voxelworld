#version 330 core

out vec4 color;
in vec3 fragpos;

void main() {
	//Test color
	//TODO: add textures/shading
	color = vec4(fract(fragpos), 1.0);
}
