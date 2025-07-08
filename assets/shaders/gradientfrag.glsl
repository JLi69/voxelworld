#version 330 core

out vec4 color;

uniform vec4 start;
uniform vec4 mid;
uniform vec4 end;

//A value between 0.0 and 1.0
uniform float perc;

void main() {
	color = 
		((mid - start) * perc * 2.0 + start) * float(perc < 0.5) +
		((end - mid) * (perc - 0.5) * 2.0 + mid) * float(perc >= 0.5);
}
