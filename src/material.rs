use vek::Rgba;

#[derive(Debug)]
pub struct Material {
    pub diffuse_color: Rgba<f32>,
}

impl Material {
    pub fn from_gltf(mat: &gltf::Material) -> Self {
        let [r, g, b, a] = mat.pbr_metallic_roughness().base_color_factor();
        Self {
            diffuse_color: Rgba::new(r, g, b, a),
        }
    }
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
