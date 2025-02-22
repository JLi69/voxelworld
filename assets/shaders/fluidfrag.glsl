#version 330 core

out vec4 color;
in vec3 fragpos;
in vec3 chunkfragpos;
flat in uint blockid;
flat in uint faceid;
in vec3 tint;

uniform sampler2D tex;

uniform vec3 campos;
uniform float fogdist;
uniform float fogstrength;
uniform vec4 fogcolor;
uniform float timepassed;
uniform float flowspeed;

const float shading[] = float[](0.9, 1.0, 0.7);
vec2 texturecoords[] = vec2[](
	vec2(fract(chunkfragpos.z), 1.0 - fract(chunkfragpos.y)),
	vec2(fract(chunkfragpos.z), 1.0 - fract(chunkfragpos.x)),
	vec2(fract(chunkfragpos.x), 1.0 - fract(chunkfragpos.y))
);
const float TEX_FRAC = 1.0 / 16.0;

vec2 transformTc(vec2 tc) {
	vec2 timeoffset = timepassed * flowspeed * vec2(0.0, -1.0);
	float x = float(int(blockid) % 16) * TEX_FRAC;
	float y = float(int(blockid) / 16) * TEX_FRAC;
	float tcx = fract(tc.x + timeoffset.x) * TEX_FRAC * 0.98;
	float tcy = fract(tc.y + timeoffset.y) * TEX_FRAC * 0.98;
	float offset = TEX_FRAC * 0.01;
	return vec2(tcx + x + offset, tcy + y + offset);
}

void main() {
	vec2 tc = transformTc(texturecoords[faceid]);
	color = texture(tex, tc);
	color *= vec4(tint, 1.0);
	float alpha = color.a;
	color *= shading[faceid];
	float mixamt = min(max(length(fragpos - campos) - fogdist, 0.0) * fogstrength, 1.0);
	color = mix(color, fogcolor, mixamt);
	color.a = alpha;

	if(color.a < 0.5)
		discard;
}
