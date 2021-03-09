#version 330 core

struct Material {
    vec3 ambient;
    vec3 diffuse;
    vec3 specular;
    float shininess;
};

// struct Light {
//   vec3 position;
//   vec3 ambient;
//   vec3 diffuse;
//   vec3 specular;
// };

// uniform Light light;
// uniform Material material;

varying vec3 N;
varying vec3 v;
varying Material material;

void main() {
    vec3 light_pos = vec3(2, 0, 0);
    vec3 light_ambient = vec3(1.0, 1.0, 1.0);

    vec3 norm = normalize(N);
    vec3 L = normalize(light_pos - v); // direction
    vec3 E = normalize(-v); // we are in Eye Coordinates, so EyePos is (0,0,0)
    vec3 R = reflect(-L, norm);

    //calculate Ambient Term:
    vec3 Iamb = light_ambient * material.ambient;

    //calculate Diffuse Term:
    vec3 Idiff = light_ambient * (material.diffuse * max(dot(norm, L), 0.0));
    Idiff = clamp(Idiff, 0.0, 1.0);

    // calculate Specular Term:
    vec3 Ispec = light_ambient * (material.specular * pow(max(dot(R, E), 0.0), material.shininess));
    Ispec = clamp(Ispec, 0.0, 1.0);

    gl_FragColor = vec4(Iamb + Idiff + Ispec, 1.0);
}
