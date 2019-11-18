#version 140

// The Model View Projection matrix
uniform mat4 mvp;

// The thickness of the outlines. This may need to change, depending on the
// scale of the objects you are drawing.
uniform float outline_thickness;

in vec3 position;
// This vector is assumed to be normalized
in vec3 normal;

void main() {
    // Translate the position along the normal based on the outline thickness.
    // This has the effect of drawing a slightly expanded version of the object.
    // If we draw this expanded object in the outline color and then draw the
    // original object on top, only the additional "outline" portion will
    // remain. Thus drawing a crude approximation of an outline.
    position += normal * outline_thickness;

    // Transforms the position to screen space
    gl_Position = mvp * vec4(position, 1.0);
}
