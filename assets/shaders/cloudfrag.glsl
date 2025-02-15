#version 330 core

uniform vec3 campos;
uniform float fogdist;
uniform float fogstrength;
uniform vec4 fogcolor;
uniform sampler2D tex;
uniform float total_time;

out vec4 color;
in vec3 fragpos;

const float scale = 6.0;
const float scrollspeed = 3.0;

void main() {
	vec2 texsize = textureSize(tex, 0) * 2.0;
	vec2 offset = total_time * vec2(0.0, scrollspeed);
	vec2 tc = (fragpos.xz + offset) / (texsize.x * scale);
	color = texture(tex, fract(tc / scale));	

	if(color.a < 0.5)
		discard;
	float fogamt = max(length(fragpos - campos) - fogdist, 0.0) / (scale * 4.0);
	float mixamt = min(fogamt * fogstrength, 1.0);
	color = mix(color, fogcolor, mixamt);
}
