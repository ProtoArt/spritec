#version 140

struct DirectionalLight {
    vec3 direction;
    vec4 color;
    float intensity;
}

struct Material {
    vec4 diffuse_color;
}

// Light parameters
uniform DirectionalLight light;
uniform float ambient_intensity;

// Material data
uniform Material material;

// Both of these vectors are assumed to be normalized
in vec3 v_normal;
in vec3 v_position;

out vec4 color;

void main() {
    // Calculate diffuse light amount
    // max() is used to bottom out at zero if the dot product is negative
    float diffuse_intensity = max(dot(v_normal, u_light), 0.0);

    // Calculate what would normally be the final color, including texturing and
    // diffuse lighting
    float light_intensity = ambient_intensity + diffuse_intensity;
    color = material.diffuse_color * light.intensity;

    // Save alpha for later so we can restore it after changing the color a bunch
    float alpha = color.a;
    if (light_intensity > 0.95) {
        // Leave the color as-is
        // e.g. color *= 1.0;

    } else if (light_intensity > 0.5) {
        color *= 0.7;

    } else if (light_intensity > 0.05) {
        color *= 0.35;

    } else {
        color *= 0.1;
    }

    // Gamma correction
    // Technique from: https://learnopengl.com/Advanced-Lighting/Gamma-Correction
    float gamma = 2.2;
    color = pow(color, vec4(1.0/gamma));

    // Reassign the final alpha because we don't actually want the calculations above to
    // influence this value
    color.a = alpha;
}
