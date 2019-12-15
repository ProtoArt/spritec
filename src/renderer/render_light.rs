use vek::{Vec3, Rgba};

use super::uniform_map::UniformMap;

/// This struct must match the `DirectionalLight` struct in our shaders
#[derive(Debug)]
pub struct RenderDirectionalLight {
    /// The **normalized** direction of the diffuse light being cast on the model
    pub direction: Vec3<f32>,
    /// The color of the diffuse light
    pub color: Rgba<f32>,
    /// The intensity of the diffuse light
    pub intensity: f32,
}

impl RenderDirectionalLight {
    pub fn to_uniforms(&self) -> UniformMap {
        let &Self {direction, color, intensity} = self;

        let mut uniforms = UniformMap::default();
        // These names must correspond to names in our shaders
        uniforms.insert("direction", direction.into_array());
        uniforms.insert("color", color.into_array());
        uniforms.insert("intensity", intensity);

        uniforms
    }
}
