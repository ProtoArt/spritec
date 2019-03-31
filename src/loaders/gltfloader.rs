use std::rc::Rc;

use gltf::mesh;

use crate::geometry::Mesh;
use crate::loaders::fileloader::FileLoader;
use crate::material::Material;

pub struct GltfLoader {}

impl FileLoader for GltfLoader {
    fn load_file(filepath: &str) -> Vec<Mesh> {
        let (document, buffers, _) = gltf::import(filepath).expect("Could not open Gltf file");

        let mut ret: Vec<Mesh> = Vec::new();

        for mesh in document.meshes() {
            for primitive in mesh.primitives() {
                // We're only dealing with triangle meshes
                assert_eq!(mesh::Mode::Triangles, primitive.mode());

                let reader = primitive.reader(|buffer| Some(&buffers[buffer.index()]));
                let positions = reader.read_positions().unwrap().collect::<Vec<_>>();
                let normals = reader.read_normals().unwrap().collect::<Vec<_>>();
                assert_eq!(positions.len(), normals.len()); // not handling optional normals yet

                let indices = reader
                    .read_indices()
                    .map(|read_indices| read_indices.into_u32().collect::<Vec<_>>())
                    .expect("Failed to read indices");

                let material = Material::from_gltf(&primitive.material());

                ret.push(Mesh::from_gltf(
                    indices,
                    positions,
                    normals,
                    Rc::new(material),
                ));
            }
        }

        ret
    }
}
