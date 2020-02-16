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

in vec3 position;
in vec3 normal;

// Indexes into joint_matrices
in uvec4 joint_influences;
in vec4 joint_weights;

// The normal, in the world coordinate system
out vec3 v_normal;
// The position, in the world coordinate system
out vec3 v_position;

mat4 joint_matrix(uint i) {
    return mat4(
        texelFetch(joint_matrices, ivec2(0, i), 0),
        texelFetch(joint_matrices, ivec2(1, i), 0),
        texelFetch(joint_matrices, ivec2(2, i), 0),
        texelFetch(joint_matrices, ivec2(3, i), 0)
    );
}

void main() {
    // Transform normals to preserve orthogonality after non-uniform transformations.
    v_normal = mat3(model_inverse_transpose) * normal;

    mat4 skin_mat =
        joint_weights.x * joint_matrix(joint_influences.x) +
        joint_weights.y * joint_matrix(joint_influences.y) +
        joint_weights.z * joint_matrix(joint_influences.z) +
        joint_weights.w * joint_matrix(joint_influences.w);

    // We guarantee that if no skinning information is available, skin_mat
    // will be the identity matirx, thus ensuring that only model_transform gets
    // multiplied. If there is skinning information, the joint matrices have
    // been premultiplied by model_transform.
    //
    // Additionally, this all only works out if the weights are guranteed to add
    // up to 1.0. Luckily, the glTF spec says:
    //
    // > The joint weights for each vertex must be non-negative, and normalized
    // > to have a linear sum of 1.0. No joint may have more than one non-zero
    // > weight for a given vertex.
    v_position = vec3(model_transform * skin_mat * vec4(position, 1.0));

    // Transforms the position to screen space
    gl_Position = mvp * vec4(position, 1.0);
}
