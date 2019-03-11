use vek::{Vec3, Rgba};

#[derive(Debug)]
pub struct DiffuseLight {
    /// The **normalized** direction of the diffuse light being cast on the model
    pub direction: Vec3<f32>,
    /// The color of the diffuse light
    pub color: Rgba<f32>,
    /// The intensity of the diffuse light
    pub intensity: f32,
}
