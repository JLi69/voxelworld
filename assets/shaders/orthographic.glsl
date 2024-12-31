#version 330 core

layout(location = 0) in uvec4 vertdata;
layout(location = 1) in uint data;

uniform mat4 screen;
uniform mat4 transform;
uniform vec3 offset;

out vec3 fragpos;
out vec3 chunkfragpos;
flat out uint blockid;
flat out uint faceid;

void main() {
	float x = float(vertdata.x);
	float y = float(vertdata.y);
	float z = float(vertdata.z);
	uint id = vertdata.w;

	vec4 pos = vec4(x + offset.x, y + offset.y, z + offset.z, 1.0);
	fragpos = pos.xyz;
	chunkfragpos = vec3(x, y, z);
	vec4 transformed = transform * pos;
	gl_Position = screen * vec4(transformed.x, transformed.y, 0.0, 1.0);
	blockid = id;
	faceid = data & 3u;
}
