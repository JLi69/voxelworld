#version 330 core

layout(location = 0) in vec4 pos;

uniform mat4 persp;
uniform mat4 view;
uniform mat4 transform;

out vec3 fragpos;
out vec2 tc;

uniform uint worldsz;

const int LAYERS = 8;

void main() {
	int i = int(gl_InstanceID) / LAYERS;
	int ix = i % int(worldsz);
	int iy = i / int(worldsz);

	vec4 p = vec4(
		pos.x + float(ix) * 2.0 - float(worldsz),
		0.0,
		pos.y + float(iy) * 2.0 - float(worldsz),
		1.0
	);
	vec4 transformed = transform * p;
	//For clouds, to give them a 3D effect
	int layer = int(gl_InstanceID) % LAYERS;
	transformed -= vec4(0.0, float(layer), 0.0, 0.0);
	vec4 glpos = persp * view * transformed;
	tc = pos.xy * 0.5 + vec2(0.5, 0.5);
	tc = tc.yx;
	tc.y = 1.0 - tc.y;
	gl_Position = glpos.xyww;
	fragpos = (transform * p).xyz;
}
