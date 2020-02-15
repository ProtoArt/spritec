#version 140

// The Model View Projection matrix
uniform mat4 mvp;
// The model matrix
uniform mat4 model_transform;
// The transpose of the inverse of the model matrix, used for
// transforming the vertex's normal
uniform mat4 model_inverse_transpose;

in vec3 position;
in vec3 normal;
in uvec4 joint_influences;
in vec4 joint_weights;

// The normal, in the world coordinate system
out vec3 v_normal;
// The position, in the world coordinate system
out vec3 v_position;

void main() {
    // Transform normals to preserve orthogonality after non-uniform transformations.
    v_normal = mat3(model_inverse_transpose) * normal;
    v_position = vec3(model_transform * vec4(position, 1.0));

    // Transforms the position to screen space
    gl_Position = mvp * vec4(position, 1.0);
}
