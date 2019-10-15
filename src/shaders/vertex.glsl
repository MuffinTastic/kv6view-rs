#version 140

in vec3 position;
in vec3 normal;
in vec3 face;
in vec3 color;

uniform mat4 perspective;
uniform mat4 view;
uniform mat4 model;

out vec3 frag_position;
out vec3 frag_normal;
out vec3 frag_face;
out vec3 frag_color;

void main() {
    gl_Position = perspective * view * model * vec4(position, 1.0);
    frag_position = vec3(model * vec4(position, 1.0));
    frag_normal = normal;
    frag_face = face;
    frag_color = color;
}