#version 330 core

uniform mat4 mvp;
uniform mat4 light_mvp;

layout (location = 0) in vec3 position;
layout (location = 1) in vec3 normal;
layout (location = 2) in vec2 texcoords;
layout (location = 3) in vec4 t0;
layout (location = 4) in vec4 t1;
layout (location = 5) in vec4 t2;
layout (location = 6) in vec4 t3;
layout (location = 7) in vec3 m_ambient;
layout (location = 8) in vec3 m_diffuse;
layout (location = 9) in vec3 m_specular;
layout (location = 10) in float m_shininess;

out VS_OUT {
    vec3 FragPos;
    vec3 Normal;
    vec2 TexCoords;
    vec4 FragPosLightSpace;
} vs_out;

out Material {
    vec3 ambient;
    vec3 diffuse;
    vec3 specular;
    float shininess;
} material;

void main() {
    mat4 transform = mat4(t0, t1, t2, t3);

    material.ambient = m_ambient;
    material.diffuse = m_diffuse;
    material.specular = m_specular;
    material.shininess = m_shininess;

    vec4 pos = transform * vec4(position, 1.0);

    // vs_out.Normal = mat3(transpose(inverse(transform))) * normal;
    vs_out.Normal = (transform * vec4(normal, 1.0)).xyz;
    // vs_out.Normal = normal;
    vs_out.FragPos = pos.xyz;
    vs_out.TexCoords = texcoords;
    vs_out.FragPosLightSpace = light_mvp * pos;
    gl_Position = mvp * pos;
}