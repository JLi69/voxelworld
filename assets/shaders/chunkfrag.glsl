#version 330 core

out vec4 color;
in vec3 fragpos;
in vec3 chunkfragpos;
flat in uint blockid;
flat in uint faceid;

uniform vec3 selected;
uniform bool selectedEmpty;
uniform sampler2D tex;

const float shading[] = float[](0.9, 1.0, 0.7);
vec2 texturecoords[] = vec2[](
	vec2(fract(chunkfragpos.z), 1.0 - fract(chunkfragpos.y)),
	vec2(fract(chunkfragpos.z), 1.0 - fract(chunkfragpos.x)),
	vec2(fract(chunkfragpos.x), 1.0 - fract(chunkfragpos.y))
);
const float TEX_FRAC = 1.0 / 16.0;

bool fragIsSelected() {
	return 
		abs(fragpos.x - (selected.x + 0.5)) <= 0.5 &&
		abs(fragpos.y - (selected.y + 0.5)) <= 0.5 &&
		abs(fragpos.z - (selected.z + 0.5)) <= 0.5 &&
		!selectedEmpty;
}

vec2 transformTc(vec2 tc) {
	float x = float(int(blockid) % 16) * TEX_FRAC;
	float y = float(int(blockid) / 16) * TEX_FRAC;
	float tcx = tc.x * TEX_FRAC * 0.98;
	float tcy = tc.y * TEX_FRAC * 0.98;
	float offset = TEX_FRAC * 0.01;
	return vec2(tcx + x + offset, tcy + y + offset);
}

void main() {
	vec2 tc = transformTc(texturecoords[faceid]);
	color = texture(tex, tc);
	color *= shading[faceid];
	vec4 highlightColor = 
		color * (1.0 - float(fragIsSelected())) +
		vec4(1.0, 1.0, 1.0, 1.0) * float(fragIsSelected());
	color = mix(color, highlightColor, 0.2);
}
