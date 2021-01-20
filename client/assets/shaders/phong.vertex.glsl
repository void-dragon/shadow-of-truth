uniform mat4 mvp;

attribute vec3 position;
attribute vec3 normal;
attribute vec4 t0, t1, t2, t3;

varying vec3 N;
varying vec3 v;

void main() {
    mat4 transform = mat4(t0, t1, t2, t3);

    mat4 tr = mvp * transform;

    N = normalize(tr * vec4(normal, 1.0)).xyz;
    v = normalize(transform * vec4(position, 1.0)).xyz;
    gl_Position = tr * vec4(position, 1.0);
}