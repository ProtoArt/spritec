use std::sync::Arc;

use crate::math::Rgba;

use super::Texture;

#[derive(Debug)]
pub struct Material {
    pub diffuse_color: Rgba,
    pub texture: Option<Arc<Texture>>,
}

impl Default for Material {
    fn default() -> Self {
        // Based on the default material in glTF
        // See: https://github.com/KhronosGroup/glTF/tree/92f59a0dbefe2d54cff38dba103cd70462cc778b/specification/2.0#reference-pbrmetallicroughness
        Self {
            diffuse_color: Rgba::white(),
            texture: None,
        }
    }
}

impl From<tobj::Material> for Material {
    fn from(mat: tobj::Material) -> Self {
        Self {
            diffuse_color: Rgba::from_opaque(mat.diffuse),
            texture: None,
        }
    }
}

impl Material {
    pub fn from_gltf(mat: gltf::Material, textures: &[Arc<Texture>]) -> Self {
        let pbr = mat.pbr_metallic_roughness();
        let [r, g, b, a] = pbr.base_color_factor();
        Self {
            diffuse_color: Rgba {r, g, b, a},
            texture: pbr.base_color_texture().map(|info| {
                assert_eq!(info.tex_coord(), 0, "Only TEXCOORD_0 is supported in glTF files");
                textures[info.texture().index()].clone()
            }),
        }
    }
}
