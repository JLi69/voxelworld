#version 330 core

out vec4 color;
in vec2 texcoord;
in vec3 fragpos;
in vec3 tint;

uniform sampler2D tex;

uniform vec3 campos;
uniform vec3 lightcolor;
uniform float fogdist;
uniform float fogstrength;
uniform vec4 fogcolor;

const float QUADRATIC = 1.0 / 12.0;
const float LINEAR = 1.0 / 16.0;
const float CONSTANT = 1.0;

void main() {
	color = texture(tex, texcoord);
	
	float d = length(fragpos - campos);
	vec3 attenuated = lightcolor * 1.0 / (d * d * QUADRATIC + d * LINEAR + CONSTANT);
	vec4 light = clamp(vec4(tint, 0.0) + vec4(attenuated, 0.0), 0.0, 1.0);
	light.a = 1.0;
	color *= light;
	
	float alpha = color.a;
	float mixamt = min(max(length(fragpos - campos) - fogdist, 0.0) * fogstrength, 1.0);
	color = mix(color, fogcolor, mixamt);
	color.a = alpha;

	if(color.a < 0.5)
		discard;
}
