#version 140

in vec3 frag_position;
in vec3 frag_normal;
in vec3 frag_face;
in vec3 frag_color;

uniform vec3 light_dir;
uniform vec3 aos_team_color;

out vec4 out_color;

void main() {
    vec3 vox_color = frag_color;
    if (vox_color == vec3(0.0)) {
        vox_color = aos_team_color;
    }

    float ambient_strength = 0.1;
    vec3 ambient = ambient_strength * vec3(1.0);

    vec3 voxel_norm = normalize(frag_normal);
    vec3 light_norm = normalize(light_dir);

    float voxel_diff = max(dot(voxel_norm, light_norm) * 0.5 + 0.5, 0.0);
    float face_diff = max(dot(frag_face, light_norm) * 0.6 + 0.45, 0.0);
    vec3 diffuse = (voxel_diff * 0.75 + face_diff * 0.15) * vec3(1.0);

    vec3 result = (ambient + diffuse) * (vox_color / 255.0);
    out_color = vec4(result, 1.0);
}