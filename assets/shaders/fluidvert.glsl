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

const float MIN_LIGHT = 0.05;

void main() {
	float x = float(vertdata.x);
	float y = float(vertdata.y);
	float z = float(vertdata.z);
	uint id = vertdata.w;

	uint level = (data.x & (7u << 2)) >> 2;

	vec4 pos = vec4(x, y, z, 0.0) + vec4(chunkpos.xyz, 1.0);
	pos.y -= 1.0 / 8.0 * float(level);
	fragpos = pos.xyz;
	chunkfragpos = vec3(x, y, z);
	chunkfragpos.y -= 1.0 / 8.0 * float(level);
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
