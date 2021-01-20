//precision mediump float;

uniform vec3 lights[3];

varying vec3 N;
varying vec3 v;

void main() {
    // gl_FragColor = vec4(0.6, 0.2, 0.9, 1);

   vec3 L = normalize(vec3(2, 0, 0) - v);
   vec3 E = normalize(-v); // we are in Eye Coordinates, so EyePos is (0,0,0)
   vec3 R = reflect(-L, N);

   //calculate Ambient Term:
   vec4 Iamb = vec4(0.3, 0.2, 0.9, 1.0);

   //calculate Diffuse Term:
   vec4 Idiff = Iamb * max(dot(N, L), 0.0);
   Idiff = clamp(Idiff, 0.0, 1.0);

   // calculate Specular Term:
   const float specular = 0.5;
   const float shininess = 10.0;
   vec4 Ispec = vec4(specular, specular, specular, 1) * pow(max(dot(R, E), 0.0), 0.3 * shininess);
   Ispec = clamp(Ispec, 0.0, 1.0);
   // write Total Color:
   vec4 sceneColor = vec4(0, 0, 0, 1);

   gl_FragColor = sceneColor + Iamb + Idiff + Ispec;
}
