use vek::Rgba;

#[derive(Debug)]
pub struct Material {
    pub diffuse_color: Rgba<f32>,
}

impl Default for Material {
    fn default() -> Self {
        Self {
            diffuse_color: Rgba {r: 0.0, g: 0.0, b: 0.0, a: 0.0},
        }
    }
}

impl<'a> From<gltf::Material<'a>> for Material {
    fn from(mat: gltf::Material<'a>) -> Self {
        let [r, g, b, a] = mat.pbr_metallic_roughness().base_color_factor();
        Self {
            diffuse_color: Rgba::new(r, g, b, a),
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
