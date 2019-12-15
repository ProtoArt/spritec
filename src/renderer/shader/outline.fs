#version 140

// The color for drawing the outline
uniform vec4 outline_color;

// This vector is unused because the outline is the same color everywhere
in vec3 v_position;

out vec4 color;

void main() {
    // Draw everything in the outline color
    color = outline_color;
}
