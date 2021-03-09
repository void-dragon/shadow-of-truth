#version 330 core

struct Material {
    vec3 ambient;
    vec3 diffuse;
    vec3 specular;
    float shininess;
};

uniform mat4 mvp;

attribute vec3 position;
attribute vec3 normal;
attribute vec4 t0, t1, t2, t3;
attribute vec3 m_ambient;
attribute vec3 m_diffuse;
attribute vec3 m_specular;
attribute float m_shininess;

varying vec3 N;
varying vec3 v;
varying Material material;

void main() {
    mat4 transform = mat4(t0, t1, t2, t3);

    material.ambient = m_ambient;
    material.diffuse = m_diffuse;
    material.specular = m_specular;
    material.shininess = m_shininess;

    mat4 tr = mvp * transform;

    // N = (transform * vec4(normal, 0.1)).xyz;
    N = mat3(transpose(inverse(transform))) * normal;
    v = (transform * vec4(position, 1.0)).xyz;
    gl_Position = tr * vec4(position, 1.0);
}