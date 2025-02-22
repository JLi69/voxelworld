#version 330 core

out vec4 color;
in vec2 texcoord;
in vec3 fragpos;
in vec3 tint;

uniform sampler2D tex;

uniform vec3 campos;
uniform float fogdist;
uniform float fogstrength;
uniform vec4 fogcolor;

void main() {
	color = texture(tex, texcoord);
	color *= vec4(tint, 1.0);
	float alpha = color.a;
	float mixamt = min(max(length(fragpos - campos) - fogdist, 0.0) * fogstrength, 1.0);
	color = mix(color, fogcolor, mixamt);
	color.a = alpha;

	if(color.a < 0.5)
		discard;
}
