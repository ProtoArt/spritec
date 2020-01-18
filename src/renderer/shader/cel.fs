#version 140

// A directional, point or spot light.
//
// For directional lights,
// * position.w = 0.0
// * range - ignored (suggested: 0.0)
// * cone_direction - ignored (suggested: vec3(0.0, 0.0, 0.0))
// * light_angle_scale - ignored (suggested: 0.0)
// * light_angle_offset - ignored (suggested: 0.0)
//
// For point lights,
// * position.w = 1.0
// * cone_direction = vec3(0.0, 0.0, 0.0)
// * light_angle_scale = 0.0
// * light_angle_offset = 0.0
//
// For spot lights,
// * position.w = 1.0
struct Light {
    // If w = 1.0, the position of the light in world coordinates
    // If w = 0.0, the **normalized** direction that the light travels in
    vec4 position;
    // The intensities of the red, green, and blue components of the light
    // When converting from glTF, this is `light.color * light.intensity`
    //
    // All components should be between 0.0 and 1.0
    vec3 color;

    // Hint defining a distance cutoff at which the light's intensity may be
    // considered to have reached zero. Supported only for point and spot
    // lights. Must be non-negative. When 0.0, range is assumed to be infinite.
    float range;

    // SPOT LIGHT CONE SETTINGS
    //
    // Based on reference code provided here:
    // https://github.com/KhronosGroup/glTF/tree/92f59a0dbefe2d54cff38dba103cd70462cc778b/extensions/2.0/Khronos/KHR_lights_punctual#inner-and-outer-cone-angles

    // the direction from the point of the cone, through the center of the cone
    vec3 cone_direction;
    // light_angle_scale = 1.0f / max(0.001f, cos(inner_cone_angle) - cos(outer_cone_angle))
    float light_angle_scale;
    // light_angle_offset = -cos(outer_cone_angle) * light_angle_scale
    float light_angle_offset;

    // Angle, in radians, from centre of spotlight where falloff begins. Must be
    // greater than or equal to 0 and less than outer_cone_angle.
    //float inner_cone_angle;
    // Angle, in radians, from centre of spotlight where falloff ends. Must be
    // greater than inner_cone_angle and less than or equal to PI / 2.0.
    //
    // To disable angular attenuation, set this value to PI radians
    //float outer_cone_angle;
};

struct Material {
    vec4 diffuse_color;
};

// Light parameters
#define MAX_LIGHTS 10
uniform int num_lights;
uniform Light lights[MAX_LIGHTS];
uniform vec3 ambient_light;

// Material data
uniform Material material;

// This is assumed to be normalized
in vec3 v_normal;
in vec3 v_position;

out vec4 frag_color;

// https://github.com/KhronosGroup/glTF-Sample-Viewer/blob/a18868cfe652bab4c084c751c80a6cfb55ae0f2f/src/shaders/metallic-roughness.frag#L199-L208
float range_attenuation(float distance, float range) {
    if (range <= 0.0) {
        // range is unlimited
        return 1.0;
    }

    return max(min(1.0 - pow(distance / range, 4), 1.0), 0.0) / pow(distance, 2);
}

// Uses the lighting model to compute the color of a point on a surface.
//
// Both position and normal should be in the world coordinate system.
vec3 apply_light(Light light, vec3 position, vec3 normal) {
    // The lighting model implemented here is designed around supporting the
    // glTF punctual lights extension. The calculations performed conform to
    // that spec. Some features found in other lighting implementations may be
    // omitted here if they are not supported by glTF.
    //
    // glTF punctual lights: https://github.com/KhronosGroup/glTF/tree/92f59a0dbefe2d54cff38dba103cd70462cc778b/extensions/2.0/Khronos/KHR_lights_punctual
    // General articles about lighting used here:
    // * https://www.tomdalling.com/blog/modern-opengl/06-diffuse-point-lighting/
    // * https://www.tomdalling.com/blog/modern-opengl/07-more-lighting-ambient-specular-attenuation-gamma/
    // * https://www.tomdalling.com/blog/modern-opengl/08-even-more-lighting-directional-lights-spotlights-multiple-lights/

    vec3 surface_to_light;
    float attenuation = 1.0;
    if (light.position.w == 0.0) {
        // Directional light

        surface_to_light = -normalize(light.position.xyz);
        // No attenuation for directional lights
        attenuation = 1.0;

    } else {
        // Point / spot light
        surface_to_light = normalize(light.position.xyz - position);
        float distance_to_light = length(surface_to_light);
        attenuation = range_attenuation(distance_to_light, light.range);

        if (light.light_angle_scale != 0.0) {
            // From the reference code provided by glTF:
            // https://github.com/KhronosGroup/glTF/tree/92f59a0dbefe2d54cff38dba103cd70462cc778b/extensions/2.0/Khronos/KHR_lights_punctual#inner-and-outer-cone-angles
            float cd = dot(light.cone_direction, -surface_to_light);
            float angular_attenuation = clamp(cd * light.light_angle_scale + light.light_angle_offset, 0.0, 1.0);
            angular_attenuation *= angular_attenuation;

            // https://github.com/KhronosGroup/glTF-Sample-Viewer/blob/a18868cfe652bab4c084c751c80a6cfb55ae0f2f/src/shaders/metallic-roughness.frag#L248
            attenuation *= angular_attenuation;
        }
    }

    // Calculate diffuse light amount
    // max() is used to bottom out at zero if the dot product is negative
    float diffuse_intensity = max(dot(v_normal, surface_to_light), 0.0);

    // Calculate what would normally be the final color, including texturing and
    // diffuse lighting
    float light_intensity = diffuse_intensity;
    light_intensity *= attenuation;
    // Discards the material alpha component
    vec3 color = vec3(material.diffuse_color) * light.color;

    // A Cel/Toon shader implementation
    // Discretises the color to produce a "toon" effect
    // Initial version based on this article: http://rbwhitaker.wikidot.com/toon-shader

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

    return color;
}

void main() {
    // Discards the material alpha component
    vec3 final_color = vec3(material.diffuse_color) * ambient_light;
    for (int i = 0; i < num_lights; i++) {
        Light light = lights[i];
        final_color += apply_light(light, v_position, v_normal);
    }

    // Gamma correction -- apply at the very end
    // Technique from: https://learnopengl.com/Advanced-Lighting/Gamma-Correction
    float gamma = 2.2;
    final_color = pow(final_color, vec3(1.0/gamma));

    frag_color = vec4(final_color, 1.0);
}
