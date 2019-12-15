#version 140

// The Model View Projection matrix
uniform mat4 mvp;
// The transpose of the inverse of the matrix view * model, used for
// transforming the vertex's normal
uniform mat4 model_view_inverse_transpose;

in vec3 position;
in vec3 normal;

out vec3 v_normal;

void main() {
    // Transform normals to preserve orthogonality after non-uniform transformations.
    v_normal = mat3(model_view_inverse_transpose) * normal;
    // Transforms the position to screen space
    gl_Position = mvp * vec4(position, 1.0);
}
