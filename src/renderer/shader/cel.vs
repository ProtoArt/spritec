#version 140

// The Model View Projection matrix
uniform mat4 mvp;
// The model matrix
uniform mat4 model_transform;
// The transpose of the inverse of the model matrix, used for
// transforming the vertex's normal
uniform mat4 model_inverse_transpose;

// The joint matrices for each joint
//
// https://github.com/KhronosGroup/glTF-Tutorials/blob/89bb8706ec3037a38e5ed1b77b5e6a4c3038db3d/gltfTutorial/gltfTutorial_020_Skins.md#the-joint-matrices
uniform sampler2D joint_matrices;
bool has_skin;

in vec3 position;
in vec3 normal;

// Indexes into joint_matrices
in uvec4 joint_influences;
in vec4 joint_weights;

// The normal, in the world coordinate system
out vec3 v_normal;
// The position, in the world coordinate system
out vec3 v_position;

void main() {
    // Transform normals to preserve orthogonality after non-uniform transformations.
    v_normal = mat3(model_inverse_transpose) * normal;
    if (has_skin) {
        mat4 skin_mat =
            joint_weights.x * joint_matrices[joint_influences.x] +
            joint_weights.y * joint_matrices[joint_influences.y] +
            joint_weights.z * joint_matrices[joint_influences.z] +
            joint_weights.w * joint_matrices[joint_influences.w];
        v_position = vec3(skin_mat * vec4(position, 1.0));

    } else {
        v_position = vec3(model_transform * vec4(position, 1.0));
    }

    // Transforms the position to screen space
    gl_Position = mvp * vec4(position, 1.0);
}
