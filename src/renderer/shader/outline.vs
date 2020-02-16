#version 140

// The Model View Projection matrix
uniform mat4 mvp;

// The joint matrices for each joint
//
// https://github.com/KhronosGroup/glTF-Tutorials/blob/89bb8706ec3037a38e5ed1b77b5e6a4c3038db3d/gltfTutorial/gltfTutorial_020_Skins.md#the-joint-matrices
uniform sampler2D joint_matrices;

// The thickness of the outlines. This may need to change, depending on the
// scale of the objects you are drawing.
uniform float outline_thickness;

in vec3 position;
// This vector is assumed to be normalized
in vec3 normal;

// Indexes into joint_matrices
in uvec4 joint_influences;
in vec4 joint_weights;

mat4 joint_matrix(uint i) {
    return mat4(
        texelFetch(joint_matrices, ivec2(i, 0), 0),
        texelFetch(joint_matrices, ivec2(i, 1), 0),
        texelFetch(joint_matrices, ivec2(i, 2), 0),
        texelFetch(joint_matrices, ivec2(i, 3), 0)
    );
}

void main() {
    // Translate the position along the normal based on the outline thickness.
    // This has the effect of drawing a slightly expanded version of the object.
    // If we draw this expanded object in the outline color and then draw the
    // original object on top, only the additional "outline" portion will
    // remain. Thus drawing a crude approximation of an outline.
    vec3 outline_position = position + normal * outline_thickness;

    mat4 skin_mat =
        joint_weights.x * joint_matrix(joint_influences.x) +
        joint_weights.y * joint_matrix(joint_influences.y) +
        joint_weights.z * joint_matrix(joint_influences.z) +
        joint_weights.w * joint_matrix(joint_influences.w);

    // Transforms the position to screen space
    gl_Position = mvp * skin_mat * vec4(outline_position, 1.0);
}
