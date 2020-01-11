use crate::math::{Vec3, Rgba};

#[derive(Debug)]
pub struct DirectionalLight {
    /// The **normalized** direction of the diffuse light being cast on the model
    pub direction: Vec3,
    /// The color of the diffuse light
    pub color: Rgba,
    /// The intensity of the diffuse light
    pub intensity: f32,
}
