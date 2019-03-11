use vek::Rgba;

#[derive(Debug)]
pub struct Material {
    pub diffuse_color: Rgba<f32>,
}

impl Default for Material {
    fn default() -> Self {
        Self {
            diffuse_color: Rgba::black(),
        }
    }
}

impl From<tobj::Material> for Material {
    fn from(mat: tobj::Material) -> Self {
        Self {
            diffuse_color: Rgba::from_opaque(mat.diffuse),
        }
    }
}
