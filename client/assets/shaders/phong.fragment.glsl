#version 330 core

struct Light {
  vec3 position;
  vec3 ambient;
  vec3 diffuse;
  vec3 specular;
};

uniform sampler2D shadowMap;
uniform Light light;

in VS_OUT {
  vec3 FragPos;
  vec3 Normal;
  vec2 TexCoords;
  vec4 FragPosLightSpace;
} fs_in;

in Material {
  vec3 ambient;
  vec3 diffuse;
  vec3 specular;
  float shininess;
} material;

float shadow_calculation() {
  vec3 projCoords = fs_in.FragPosLightSpace.xyz / fs_in.FragPosLightSpace.w;
  projCoords = projCoords * 0.5 + 0.5;
  float closestDepth = texture(shadowMap, projCoords.xy).r;
  float currentDepth = projCoords.z;
  float bias = 0.05;
  float shadow = (currentDepth - bias) > closestDepth  ? 1.0 : 0.0;

  return shadow;
}

void main() {
  vec3 norm = normalize(fs_in.Normal);
  vec3 L = normalize(light.position - fs_in.FragPos); // direction
  vec3 E = normalize(-fs_in.FragPos); // we are in Eye Coordinates, so EyePos is (0,0,0)
  vec3 R = reflect(-L, norm);

  //calculate Ambient Term:
  vec3 Iamb = light.ambient * material.ambient;

  //calculate Diffuse Term:
  vec3 Idiff = light.diffuse * (material.diffuse * max(dot(norm, L), 0.0));
  Idiff = clamp(Idiff, 0.0, 1.0);

  // calculate Specular Term:
  vec3 Ispec = light.specular * (material.specular * pow(max(dot(R, E), 0.0), material.shininess));
  Ispec = clamp(Ispec, 0.0, 1.0);

  float shadow = shadow_calculation();

  gl_FragColor = vec4(Iamb + (1.0 - shadow) * (Idiff + Ispec), 1.0);
}
