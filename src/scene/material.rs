use vek::Rgba;

#[derive(Debug)]
pub struct Material {
    pub diffuse_color: Rgba<f32>,
}

impl Default for Material {
    fn default() -> Self {
        // Based on the default material in glTF
        // See: https://github.com/KhronosGroup/glTF/tree/92f59a0dbefe2d54cff38dba103cd70462cc778b/specification/2.0#reference-pbrmetallicroughness
        Self {
            diffuse_color: Rgba::white(),
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

impl<'a> From<gltf::Material<'a>> for Material {
    fn from(mat: gltf::Material<'a>) -> Self {
        let [r, g, b, a] = mat.pbr_metallic_roughness().base_color_factor();
        Self {
            diffuse_color: Rgba::new(r, g, b, a),
        }
    }
}
