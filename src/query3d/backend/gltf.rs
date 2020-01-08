use std::sync::Arc;
use std::path::Path;

use crate::scene::{Scene, Mesh, Material, CameraType};
use crate::renderer::{Display, ShaderGeometry, Camera, DirectionalLight};
use crate::query3d::{GeometryQuery, CameraQuery, LightQuery};

use super::{QueryBackend, QueryError};

/// Represents a single glTF file
#[derive(Debug)]
pub struct GltfFile {
}

impl GltfFile {
    /// Opens a glTF file
    pub fn open(path: &Path) -> Result<Self, gltf::Error> {
        let (document, buffers, images) = gltf::import(path)?;

        let materials: Vec<_> = document.materials()
            .map(|mat| Arc::new(Material::from(mat)))
            .collect();
        let meshes: Vec<_> = document.meshes()
            .map(|mesh| Arc::new(Mesh::from_gltf(mesh, &materials, &buffers)))
            .collect();

        let cameras: Vec<_> = document.cameras()
            .map(|cam| Arc::new(CameraType::from(cam)))
            .collect();

        //TODO: Lights
        let scenes: Vec<_> = document.scenes()
            .map(|scene| Arc::new(Scene::from_gltf(scene, &meshes, &cameras)))
            .collect();

        unimplemented!()
    }
}

impl QueryBackend for GltfFile {
    fn query_geometry(&mut self, query: &GeometryQuery, display: &Display) -> Result<Arc<Vec<Arc<ShaderGeometry>>>, QueryError> {
        unimplemented!()
    }

    fn query_camera(&mut self, query: &CameraQuery) -> Result<Arc<Camera>, QueryError> {
        unimplemented!()
    }

    fn query_lights(&mut self, query: &LightQuery) -> Result<Arc<Vec<Arc<DirectionalLight>>>, QueryError> {
        unimplemented!()
    }
}
