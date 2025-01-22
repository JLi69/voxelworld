#version 330 core

layout(location = 0) in uvec4 vertdata;
layout(location = 1) in uvec3 data;

uniform mat4 persp;
uniform mat4 view;
uniform vec3 chunkpos;

flat out uint faceid;
out vec2 texcoord;
out vec3 fragpos;

const float TEX_FRAC = 1.0 / 16.0;

vec2 transformTc(vec2 tc, uint blockid) {
	float x = float(int(blockid) % 16) * TEX_FRAC;
	float y = float(int(blockid) / 16) * TEX_FRAC;
	float tcx = tc.x * TEX_FRAC * 0.98;
	float tcy = tc.y * TEX_FRAC * 0.98;
	float offset = TEX_FRAC * 0.01;
	return vec2(tcx + x + offset, tcy + y + offset);
}

float getIntegerPart(uint x) {
	uint integerPart = x & 0x3Fu;
	//If we get integerPart = 33, then it is actually -1
	float isNegative = float(integerPart == 33u);
	return isNegative * -1.0 + (1.0 - isNegative) * float(integerPart);
}

void main() {
	float fx = float(vertdata.x >> 6) / 4.0 + float(data.y & 3u) / 16.0;
	float fy = float(vertdata.y >> 6) / 4.0 + float((data.y >> 2) & 3u) / 16.0;
	float fz = float(vertdata.z >> 6) / 4.0 + float((data.y >> 4) & 3u) / 16.0;
	
	float ix = getIntegerPart(vertdata.x);
	float iz = getIntegerPart(vertdata.z);

	float x = ix + fx;
	float y = float(vertdata.y & 0x3Fu) + fy;
	float z = iz + fz;
	uint id = vertdata.w;
	faceid = data.x & 3u;
	float tcx = float((data.z & 0xFu) | (((data.y >> 6) & 1u) << 4)) / 16.0;
	float tcy = float(((data.z >> 4) & 0xFu) | (((data.y >> 7) & 1u) << 4)) / 16.0;
	vec2 tc = vec2(tcx, 1.0 - tcy);
	tc = transformTc(tc, id);
	texcoord = tc;
	
	vec4 pos = vec4(x, y, z, 1.0) + vec4(chunkpos.xyz, 0.0);
	fragpos = pos.xyz;
	gl_Position = persp * view * pos;
}
