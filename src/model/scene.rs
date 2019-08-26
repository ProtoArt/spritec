use crate::model::{Material, Mesh, Node};
use rayon::iter::{ParallelIterator, IntoParallelIterator};
use std::collections::HashMap;
use std::collections::hash_map::Values;
use std::path::Path;
use std::sync::Arc;

/// A scene is the root node of a tree hierarchy of nodes.
#[derive(Debug, Clone)]
pub struct Scene {
    /// A map from node identifier to node, contains all nodes in the scene.
    node_map: HashMap<usize, Arc<Node>>,
    /// Top-level children nodes of a scene.
    nodes: Vec<Arc<Node>>,
}

impl Scene {
    /// Takes gltf data and reads the default scene to produce our
    /// own Scene that doesn't rely on the gltf crate.
    #[allow(clippy::map_entry)] // recursive mutable borrow of map
    pub fn from_gltf(
        document: &gltf::Document,
        buffers: &[gltf::buffer::Data],
        materials: &[Arc<Material>]
    ) -> Self {
        let node_iterator = document.nodes();
        let mut node_map =
            HashMap::<usize, Arc<Node>>::with_capacity(node_iterator.len());

        node_iterator.for_each(|gltf_node: gltf::Node| {
            if !node_map.contains_key(&gltf_node.index()) {
                let node = Arc::new(Node::from_gltf_node(
                    &buffers, &gltf_node, &node_map, &materials,
                ));
                node_map.insert(gltf_node.index(), node);
            }
        });

        let gltf_scene = document.default_scene()
            .expect("default glTF scene to exist");

        let nodes = gltf_scene.nodes().map(|gltf_node| {
            node_map.get(&gltf_node.index())
                .expect("Scene nodes to exist in the node map")
                .clone()
        }).collect();

        Self { node_map, nodes }
    }

    /// Creates a 1-node scene from an obj file.
    pub fn from_obj_file(
        path: impl AsRef<Path>
    ) -> Result<Scene, tobj::LoadError> {
        let (t_meshes, t_materials) = tobj::load_obj(path.as_ref())?;
        let materials: Vec<_> = t_materials
            .into_par_iter()
            .map(|mat| Arc::new(Material::from(mat)))
            .collect();
        let meshes = t_meshes
            .into_par_iter()
            .map(|model| Mesh::from_obj(model.mesh, &materials))
            .collect();

        let node = Arc::new(Node::new(meshes));

        let mut node_map = HashMap::<usize, Arc<Node>>::new();
        // Since obj files don't use identifiers, let's just choose `1`.
        node_map.insert(1, node.clone());

        Ok(Scene {node_map, nodes: vec![node.clone()]})
    }

    /// Collect all nodes in a scene and return it
    pub fn gather_nodes(&self) -> Values<usize, Arc<Node>> {
        self.node_map.values()
    }
}
