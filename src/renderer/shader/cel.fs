#version 140

struct DirectionalLight {
    // The **normalized** direction of the diffuse light being cast on the model
    vec3 direction;
    // The color of the diffuse light
    vec4 color;
    // The intensity of the diffuse light
    float intensity;
};

struct Material {
    vec4 diffuse_color;
};

// Light parameters
uniform DirectionalLight light;
uniform float ambient_intensity;

// Material data
uniform Material material;

// Both of these vectors are assumed to be normalized
in vec3 v_normal;
in vec3 v_position;

out vec4 color;

// A Cel/Toon shader implementation
// Initial version based on this article: http://rbwhitaker.wikidot.com/toon-shader
void main() {
    // Calculate diffuse light amount
    // max() is used to bottom out at zero if the dot product is negative
    float diffuse_intensity = max(dot(v_normal, light.direction), 0.0);

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

    // Gamma correction is NOT necessary here because GL_FRAMEBUFFER_SRGB is set
    // See: https://docs.rs/glium/0.26.0-alpha5/glium/texture/index.html#about-srgb

    // Reassign the final alpha because we don't actually want the calculations above to
    // influence this value
    color.a = alpha;
}
