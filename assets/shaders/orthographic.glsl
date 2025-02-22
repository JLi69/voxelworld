#version 330 core

layout(location = 0) in uvec4 vertdata;
layout(location = 1) in uvec3 data;

uniform mat4 screen;
uniform mat4 transform;
uniform vec3 offset;

out vec3 fragpos;
out vec3 chunkfragpos;
flat out uint blockid;
flat out uint faceid;
out vec3 tint;

void main() {
	float geox = float((vertdata.x & 0x40u) >> 6) * 0.5;
	float geoy = float((vertdata.y & 0x40u) >> 6) * 0.5;
	float geoz = float((vertdata.z & 0x40u) >> 6) * 0.5;
	float x = float(vertdata.x & 0x3Fu) + geox;
	float y = float(vertdata.y & 0x3Fu) + geoy;
	float z = float(vertdata.z & 0x3Fu) + geoz;
	uint id = vertdata.w;

	vec4 pos = vec4(x + offset.x, y + offset.y, z + offset.z, 1.0);
	fragpos = pos.xyz;
	chunkfragpos = vec3(x, y, z);
	vec4 transformed = transform * pos;
	vec4 screenPos = screen * vec4(transformed.xyz, 1.0);
	screenPos.z -= offset.z;
	gl_Position = screenPos;
	blockid = id;
	faceid = data.x & 3u;
	tint = vec3(1.0);
}
