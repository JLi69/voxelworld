#version 330 core

layout(location = 0) in uvec4 vertdata;
layout(location = 1) in uvec3 data;

uniform mat4 persp;
uniform mat4 view;
uniform vec3 chunkpos;

out vec3 fragpos;
out vec3 chunkfragpos;
flat out uint blockid;
flat out uint faceid;
out vec3 tint;

const float MIN_LIGHT = 0.15;

void main() {
	float geox = float((vertdata.x & 0x40u) >> 6) * 0.5;
	float geoy = float((vertdata.y & 0x40u) >> 6) * 0.5;
	float geoz = float((vertdata.z & 0x40u) >> 6) * 0.5;
	float x = float(vertdata.x & 0x3Fu) + geox;
	float y = float(vertdata.y & 0x3Fu) + geoy;
	float z = float(vertdata.z & 0x3Fu) + geoz;	
	uint id = vertdata.w;

	vec4 pos = vec4(x, y, z, 1.0) + vec4(chunkpos.xyz, 0.0);
	fragpos = pos.xyz;
	chunkfragpos = vec3(x, y, z);
	gl_Position = persp * view * pos;
	blockid = id;
	faceid = data.x & 3u;

	float sky = float(data.y & 0xfu) / 15.0 * (1.0 - MIN_LIGHT) + MIN_LIGHT;
	float r = float((data.y >> 4) & 0xfu) / 15.0 * (1.0 - MIN_LIGHT) + MIN_LIGHT;
	float g = float(data.z & 0xfu) / 15.0 * (1.0 - MIN_LIGHT) + MIN_LIGHT;
	float b = float((data.z >> 4) & 0xfu) / 15.0 * (1.0 - MIN_LIGHT) + MIN_LIGHT;
	tint.r = max(sky, r);
	tint.g = max(sky, g);
	tint.b = max(sky, b);
}
