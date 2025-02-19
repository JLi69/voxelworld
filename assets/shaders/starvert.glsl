#version 330 core

layout(location = 0) in vec4 pos;

uniform mat4 persp;
uniform mat4 view;
uniform mat4 transform;
uniform float rotation;

out vec3 fragpos;
out vec2 tc;

#define PI 3.1415926535

//Copied from: https://stackoverflow.com/questions/4200224/random-noise-functions-for-glsl
float rand(vec2 co){
    return fract(sin(dot(co, vec2(12.9898, 78.233))) * 43758.5453);
}

mat4 rotationX(float rad) {
	return mat4(
		1, 0, 0, 0,
		0, cos(rad), -sin(rad), 0,
		0, sin(rad), cos(rad), 0,
		0, 0, 0, 1
	);
}

mat4 rotationY(float rad) {
	return mat4(
		cos(rad), 0, -sin(rad), 0,
		0, 1, 0, 0,
		sin(rad), 0, cos(rad), 0,
		0, 0, 0, 1
	);
}

mat4 rotationZ(float rad) {
	return mat4(
		cos(rad), -sin(rad), 0, 0,
		sin(rad), cos(rad), 0, 0,
		0, 0, 1, 0,
		0, 0, 0, 1
	);
}

mat4 getRandRotationMat() {
	float i = float(gl_InstanceID) / 512.0 + 100.0;
	vec2 p = vec2(i, 0.0);
	float angle1 = rand(p) * 2.0 * PI;
	float angle2 = rand(p.yx + p.yy + p.xx) * 2.0 * PI;
	return rotationZ(-radians(rotation) + radians(30.0)) * rotationY(angle1) * rotationZ(angle2);
}

void main() {
	vec4 p = vec4(pos.x, 0.0, pos.y, 1.0);
	mat4 rotationMat = getRandRotationMat();
	vec4 transformed = rotationMat * transform * p;
	vec4 glpos = persp * view * transformed;
	tc = pos.xy * 0.5 + vec2(0.5, 0.5);
	tc = tc.yx;
	tc.y = 1.0 - tc.y;
	gl_Position = glpos.xyww;
	fragpos = transformed.xyz;
}
