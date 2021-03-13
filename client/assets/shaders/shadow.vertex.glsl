#version 330 core

uniform mat4 mvp;

layout (location = 0) in vec3 position;
layout (location = 3) in vec4 t0;
layout (location = 4) in vec4 t1;
layout (location = 5) in vec4 t2;
layout (location = 6) in vec4 t3;

void main() {
    mat4 transform = mat4(t0, t1, t2, t3);
    mat4 tr = mvp * transform;

    gl_Position = tr * vec4(position, 1.0);
}