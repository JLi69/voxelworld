#version 330 core

uniform vec3 campos;
uniform float fogdist;
uniform float fogstrength;
uniform vec4 fogcolor;
uniform sampler2D tex;
uniform float total_time;
uniform float skybrightness;

out vec4 color;
in vec3 fragpos;

const float scale = 6.0;
const float scrollspeed = 3.0;

void main() {
	vec2 texsize = textureSize(tex, 0) * 2.0;
	vec2 offset = total_time * vec2(0.0, scrollspeed);
	vec2 tc = (fragpos.xz + offset) / (texsize.x * scale);
	color = texture(tex, fract(tc / scale));

	float alpha = color.a;
	color *= (skybrightness * 0.75 + 0.25);

	//subtract (0.0, 160.0, 0.0) and divide by 2 from campos to display more clouds
	float d = length(fragpos - campos - vec3(0.0, 160.0, 0.0)) / 2.0;
	float fogamt = max(d - fogdist, 0.0) / (scale * 4.0);
	float mixamt = clamp(fogamt * fogstrength, 0.0, 0.95);
	color = mix(color, fogcolor, mixamt);
	color.a = alpha;
}
