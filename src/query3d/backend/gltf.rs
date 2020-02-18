mod scenes;
mod animation;
mod keyframes;
mod interpolate;
mod scene_anim_query_cache;

use std::sync::Arc;
use std::path::Path;
use std::collections::HashMap;

use crate::math::Mat4;
use crate::scene::{Scene, NodeTree, NodeWorldTransforms, NodeId, Node, Mesh, Skin, Material, CameraType, LightType};
use crate::renderer::{Display, ShaderGeometry, JointMatrixTexture, Camera, Light};
use crate::query3d::{GeometryQuery, GeometryFilter, AnimationQuery, CameraQuery, LightQuery};

use super::{QueryBackend, QueryError};

use scenes::Scenes;
use animation::AnimationSet;
use scene_anim_query_cache::SceneAnimQueryCache;

/// Represents a single glTF file
#[derive(Debug)]
pub struct GltfFile {
    nodes: NodeTree,
    scenes: Scenes,
    /// This is a mapping of Node ID to all the animations that act on that node
    animations: HashMap<NodeId, AnimationSet>,

    /// Cache the default joint matrix texture so we don't upload it over and over again
    default_joint_matrix_texture: Option<Arc<JointMatrixTexture>>,
    /// Cache the geometry of the entire scene, referenced by scene index (no animation query)
    scene_shader_geometry: HashMap<usize, Arc<Vec<Arc<ShaderGeometry>>>>,
    /// Cache the geometry of the entire scene, referenced by scene index and animation query
    scene_anim_shader_geometry: SceneAnimQueryCache<Arc<Vec<Arc<ShaderGeometry>>>>,
    /// Cache all of the lights in an entire scene, referenced by scene index
    scene_lights: HashMap<usize, Arc<Vec<Arc<Light>>>>,
    /// Cache the first camera in the scene
    scene_first_camera: Option<Arc<Camera>>,
    /// Cache each camera by scene index and name
    scene_cameras: HashMap<(usize, String), Arc<Camera>>,
}

impl GltfFile {
    /// Opens a glTF file
    pub fn open(path: &Path) -> Result<Self, gltf::Error> {
        let (document, buffers, _images) = gltf::import(path)?;

        let materials: Vec<_> = document.materials()
            .map(|mat| Arc::new(Material::from(mat)))
            .collect();

        let meshes: Vec<_> = document.meshes()
            .map(|mesh| Arc::new(Mesh::from_gltf(mesh, &materials, &buffers)))
            .collect();

        let skins: Vec<_> = document.skins()
            .map(|skin| Arc::new(Skin::from_gltf(skin, &buffers)))
            .collect();

        let cameras: Vec<_> = document.cameras()
            .map(|cam| Arc::new(CameraType::from(cam)))
            .collect();

        let lights: Vec<_> = document.lights().map(|lights| {
            lights.map(|light| Arc::new(LightType::from(light))).collect()
        }).unwrap_or_default();

        let nodes = NodeTree::from_ordered_nodes(document.nodes().map(|node| {
            let children = node.children().map(|node| NodeId::from_gltf(&node)).collect();
            let node = Node::from_gltf(node, &meshes, &skins, &cameras, &lights);
            (node, children)
        }));

        let scenes: Vec<_> = document.scenes()
            .map(|scene| Arc::new(Scene::from_gltf(scene)))
            .collect();
        assert!(!scenes.is_empty(), "glTF file must have at least one scene");

        // Get the default scene, or just use the first scene if no default is provided
        let default_scene = document.default_scene().map(|scene| scene.index()).unwrap_or(0);
        let scenes = Scenes::new(scenes, default_scene);

        let animations = animation::from_animations(document.animations(), &buffers);

        Ok(Self {
            nodes,
            scenes,
            animations,

            default_joint_matrix_texture: None,
            scene_shader_geometry: HashMap::new(),
            scene_anim_shader_geometry: SceneAnimQueryCache::default(),
            scene_lights: HashMap::new(),
            scene_first_camera: None,
            scene_cameras: HashMap::new(),
        })
    }
}

/// Returns a new node tree with the animations specified by the query applied to each matching
/// node. Returns an error if the query did not match any of the nodes or if a single node
/// was matched by multiple animations (ambiguous).
fn apply_animation_query(
    anim_query: &AnimationQuery,
    nodes: &NodeTree,
    animations: &HashMap<NodeId, AnimationSet>,
) -> Result<NodeTree, QueryError> {
    // Set to true if at least one animation was found and applied on any node
    // If none are applied, the animation name was probably mispelled or something
    let mut animation_found = false;

    let nodes = nodes.try_with_replacements(|node| {
        match animations.get(&node.id) {
            // If the node has a list of animations, look for the animations that match the AnimationQuery name
            Some(anim_set) => {
                // anim is the animation that will modify the transformation matrix of the current node
                let mut anims = anim_set.filter(anim_query.name.as_deref());

                // Return if no animation matches the name in the animation query
                let anim = match anims.next() {
                    Some(anim) => anim,
                    None => return Ok(None),
                };

                // Return if multiple animations match the query
                if anims.next().is_some() {
                    return Err(QueryError::AmbiguousAnimation);
                }

                animation_found = true;

                // Create and set the new transformation matrix of the current node
                let new_transform = anim.apply_at(&node.transform, &anim_query.position);
                Ok(Some(node.with_transform(new_transform)))
            },

            None => Ok(None),
        }
    })?;

    if !animation_found {
        match &anim_query.name {
            Some(name) => Err(QueryError::UnknownAnimation {name: name.to_string()}),
            None => Err(QueryError::NoAnimationFound),
        }

    } else {
        Ok(nodes)
    }
}

/// Returns the joint matrix texture for the given skin and model matrix
///
/// If `skin` is None, a default joint matrix texture will be returned
fn joint_matrices_texture(
    skin: Option<&Arc<Skin>>,
    model_transform: Mat4,
    node_world_transforms: &NodeWorldTransforms,
    display: &Display,
    default_joint_matrix_texture: &mut Option<Arc<JointMatrixTexture>>,
) -> Result<Arc<JointMatrixTexture>, QueryError> {
    Ok(match skin {
        Some(skin) => {
            let joint_matrices = skin.joint_matrices(model_transform, node_world_transforms);
            // This texture is cached in ShaderGeometry and is specific to this node + skin
            Arc::new(JointMatrixTexture::new(display, joint_matrices)?)
        },

        // The default joint matrix texture is always the same, so we try to only generate it once
        None => match default_joint_matrix_texture {
            Some(tex) => tex.clone(),

            None => {
                let tex = Arc::new(JointMatrixTexture::identity(display)?);
                *default_joint_matrix_texture = Some(tex.clone());
                tex
            },
        },
    })
}

/// Given nodes and their model/world transforms, uploads each node's geometry
///
/// This can't be a method because we need to keep the &mut self borrow split
fn upload_geometry<'a>(
    nodes: impl Iterator<Item=(&'a Node, Mat4)>,
    node_world_transforms: &NodeWorldTransforms,
    display: &Display,
    default_joint_matrix_texture: &mut Option<Arc<JointMatrixTexture>>,
) -> Result<Arc<Vec<Arc<ShaderGeometry>>>, QueryError> {
    let mut scene_geo = Vec::new();

    for (node, model_transform) in nodes {
        if let Some((mesh, skin)) = node.mesh() {
            let joint_matrices_tex = joint_matrices_texture(
                skin,
                model_transform,
                node_world_transforms,
                display,
                default_joint_matrix_texture,
            )?;

            for geo in &mesh.geometry {
                let geo = ShaderGeometry::new(
                    display,
                    geo,
                    &joint_matrices_tex,
                    model_transform,
                )?;

                scene_geo.push(Arc::new(geo));
            }
        }
    }

    if scene_geo.is_empty() {
        return Err(QueryError::NoGeometryFound);
    }

    let scene_geo = Arc::new(scene_geo);
    Ok(scene_geo)
}

impl QueryBackend for GltfFile {
    fn query_geometry(&mut self, query: &GeometryQuery, display: &Display) -> Result<Arc<Vec<Arc<ShaderGeometry>>>, QueryError> {
        let GeometryQuery {models, animation} = query;

        // Need to split the borrow of self so we don't accidentally get two mut refs
        let Self {nodes, scenes, animations, default_joint_matrix_texture, ..} = self;

        use GeometryFilter::*;
        match animation {
            Some(anim_query) => match models {
                Scene {name} => {
                    let scene_index = scenes.query(name.as_deref())?;

                    match self.scene_anim_shader_geometry.get(scene_index, anim_query) {
                        Some(scene_geo) => Ok(scene_geo.clone()),

                        None => {
                            let nodes = apply_animation_query(anim_query, nodes, animations)?;

                            let scene = &scenes[scene_index];
                            let node_world_transforms = nodes.world_transforms(&scene.roots);

                            // Upload the geometry for the entire scene since that's what the
                            // filter requested
                            let scene_geo = upload_geometry(
                                node_world_transforms.iter(&nodes),
                                &node_world_transforms,
                                display,
                                default_joint_matrix_texture,
                            )?;
                            self.scene_anim_shader_geometry.insert(scene_index, anim_query, scene_geo.clone());

                            Ok(scene_geo)
                        },
                    }
                },
            },

            None => match models {
                Scene {name} => {
                    let scene_index = scenes.query(name.as_deref())?;

                    match self.scene_shader_geometry.get(&scene_index) {
                        Some(scene_geo) => Ok(scene_geo.clone()),

                        None => {
                            let scene = &scenes[scene_index];
                            let node_world_transforms = nodes.world_transforms(&scene.roots);

                            // Upload the geometry for the entire scene since that's what the
                            // filter requested
                            let scene_geo = upload_geometry(
                                node_world_transforms.iter(nodes),
                                &node_world_transforms,
                                display,
                                default_joint_matrix_texture,
                            )?;
                            self.scene_shader_geometry.insert(scene_index, scene_geo.clone());

                            Ok(scene_geo)
                        },
                    }
                },
            },
        }
    }

    fn query_camera(&mut self, query: &CameraQuery) -> Result<Arc<Camera>, QueryError> {
        use CameraQuery::*;
        match query {
            FirstInScene {name} => {
                let scene_index = self.scenes.query(name.as_deref())?;

                match &self.scene_first_camera {
                    Some(cam) => Ok(cam.clone()),

                    None => {
                        let scene = &self.scenes[scene_index];

                        let mut nodes = scene.roots.iter().flat_map(|&root| self.nodes.traverse(root));
                        let scene_first_camera = nodes.find_map(|(parent_trans, node)| {
                            let world_transform = parent_trans * node.transform;

                            match node.camera() {
                                Some(cam) => Some(Arc::new(Camera {
                                    view: world_transform.inverted(),
                                    projection: cam.to_projection(),
                                })),

                                None => None,
                            }
                        });

                        match scene_first_camera {
                            Some(cam) => {
                                self.scene_first_camera = Some(cam.clone());
                                Ok(cam)
                            },

                            None => Err(QueryError::NoCameraFound),
                        }
                    },
                }
            },

            Named {name, scene} => {
                let scene_index = self.scenes.query(scene.as_deref())?;

                let cam_key = (scene_index, name.clone());
                match self.scene_cameras.get(&cam_key) {
                    Some(cam) => Ok(cam.clone()),

                    None => {
                        let scene = &self.scenes[scene_index];

                        let mut nodes = scene.roots.iter().flat_map(|&root| self.nodes.traverse(root));
                        // This code assumes that camera names are unique
                        let found_camera = nodes.find_map(|(parent_trans, node)| {
                            let world_transform = parent_trans * node.transform;

                            match node.camera() {
                                Some(cam) if node.name.as_ref() == Some(name) || cam.name() == Some(name) => {
                                    Some(Arc::new(Camera {
                                        view: world_transform.inverted(),
                                        projection: cam.to_projection(),
                                    }))
                                },

                                Some(_) |
                                None => None,
                            }
                        });

                        match found_camera {
                            Some(cam) => {
                                self.scene_cameras.insert(cam_key, cam.clone());
                                Ok(cam)
                            },

                            None => Err(QueryError::UnknownCamera {name: name.to_string()}),
                        }
                    },
                }
            },
        }
    }

    fn query_lights(&mut self, query: &LightQuery) -> Result<Arc<Vec<Arc<Light>>>, QueryError> {
        use LightQuery::*;
        let scene_index = match query {
            Scene {name} => self.scenes.query(name.as_deref())?,
        };

        match self.scene_lights.get(&scene_index) {
            Some(scene_lights) => Ok(scene_lights.clone()),

            None => {
                let scene = &self.scenes[scene_index];

                let mut scene_lights = Vec::new();
                for (parent_trans, node) in scene.roots.iter().flat_map(|&root| self.nodes.traverse(root)) {
                    let world_transform = parent_trans * node.transform;

                    if let Some(light) = node.light() {
                        let light = Light {data: light.clone(), world_transform};
                        scene_lights.push(Arc::new(light));
                    }
                }

                if scene_lights.is_empty() {
                    return Err(QueryError::NoLightsFound);
                }

                let scene_lights = Arc::new(scene_lights);
                self.scene_lights.insert(scene_index, scene_lights.clone());
                Ok(scene_lights)
            },
        }
    }
}
