#[derive(Debug, Clone)]
pub struct Skin {
}

impl Skin {
    pub fn from_gltf(
        mesh: gltf::Skin,
        buffers: &[gltf::buffer::Data],
    ) -> Self {
        unimplemented!()
    }
}
