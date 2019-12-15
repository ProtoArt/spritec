use vek::Rgba;

use crate::model::Material;

use super::uniform_map::UniformMap;

/// This struct must match the `Material` struct in our shaders
#[derive(Debug)]
pub struct RenderMaterial {
    pub diffuse_color: Rgba<f32>,
}

impl RenderMaterial {
    pub fn new(material: &Material) -> Self {
        let &Material {diffuse_color} = material;

        Self {
            diffuse_color,
        }
    }

    pub fn to_uniforms(&self) -> UniformMap {
        let &Self {diffuse_color} = self;

        let mut uniforms = UniformMap::default();
        // These names must correspond to names in our shaders
        uniforms.insert("diffuse_color", diffuse_color.into_array());

        uniforms
    }
}
