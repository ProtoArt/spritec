// JAMES ADDED - to be moved to separate file afterwards
use gltf::animation::util::ReadOutputs::*;
use crate::math::{Vec3, Quaternion, Mat3, Mat4, Decompose};
use interpolation;


use std::sync::Arc;
use std::path::Path;
use std::collections::HashMap;
use std::cmp::min;

use crate::scene::{Scene, NodeTree, NodeId, Node, Mesh, Skin, Material, CameraType, LightType};
use crate::renderer::{Display, ShaderGeometry, JointMatrixTexture, Camera, Light};
use crate::query3d::{GeometryQuery, GeometryFilter, CameraQuery, LightQuery};

use super::{QueryBackend, QueryError};

/// Represents a single glTF file
#[derive(Debug)]
pub struct GltfFile {
    default_scene: usize,
    nodes: Arc<NodeTree>,
    scenes: Vec<Arc<Scene>>,
    /// Cache the default joint matrix texture so we don't upload it over and over again
    default_joint_matrix_texture: Option<Arc<JointMatrixTexture>>,
    /// Cache the geometry of the entire scene, referenced by scene index
    scene_shader_geometry: HashMap<usize, Arc<Vec<Arc<ShaderGeometry>>>>,
    /// Cache all of the lights in an entire scene, referenced by scene index
    scene_lights: HashMap<usize, Arc<Vec<Arc<Light>>>>,
    /// Cache the first camera in the scene
    scene_first_camera: Option<Arc<Camera>>,
    /// Cache each camera by scene index and name
    scene_cameras: HashMap<(usize, String), Arc<Camera>>,
    /// Contains the transformation data of the animations
    // This is a mapping of Node ID to all the animations that act on that node
    animations: HashMap<NodeId, Vec<Animation>>,
}

#[derive(Debug, Default)]
struct Animation {
    name: Option<String>,
    scale_keyframes: Option<Keyframes<Vec3>>,
    rotation_keyframes: Option<Keyframes<Quaternion>>,
    translation_keyframes: Option<Keyframes<Vec3>>,
}

#[derive(Debug)]
enum Interpolation {
    Linear,
    Step,
}

trait Interpolate {
    fn interpolate(interp: &Interpolation, t: f32, prev_keyframe: Self, next_keyframe: Self) -> Self;
}

impl Interpolate for Vec3 {
    fn interpolate(interp: &Interpolation, t: f32, prev_keyframe: Vec3, next_keyframe: Vec3) -> Vec3 {
        match interp {
            Linear => {
                let [x, y, z] = interpolation::lerp(&prev_keyframe.into_array(), &next_keyframe.into_array(), &t);
                Vec3::new(x, y, z)
            },
            Step => prev_keyframe,
        }
    }
}

impl Interpolate for Quaternion {
    fn interpolate(interp: &Interpolation, t: f32, prev_keyframe: Quaternion, next_keyframe: Quaternion) -> Quaternion {
        match interp {
            Linear => Quaternion::slerp(prev_keyframe, next_keyframe, t),
            Step => prev_keyframe,
        }
    }
}


impl Animation {
    // with_name takes Option instead of String to include with Animations without names
    fn with_name(name: Option<String>) -> Self {
        Self {
            name,
            ..Self::default()
        }
    }

    // Application of animation data by decomposing the current node's transformation matrix and
    // replacing the different types of transforms if the keyframes for that transform exist
    fn apply_at(&self, transform_matrix: &Mat4, time: f32) -> Mat4 {
        let mut matrix_transforms = transform_matrix.decompose();

        if let Some(scale) = &self.scale_keyframes {
            let new_scale = match scale.surrounding(time) {
                KeyframeRange::Before(kf) => kf.value,
                KeyframeRange::After(kf) => kf.value,
                KeyframeRange::Between(kf1, kf2) => {
                    let start = kf1.time;
                    let end = kf2.time;
                    // The time factor that gives weight to the start or end frame during interpolation
                    let factor = (time - start) / (end - start);
                    Vec3::interpolate(&scale.interpolation, factor, kf1.value, kf2.value)
                },
            };
            matrix_transforms.scale = new_scale;
        }
        if let Some(rot) = &self.rotation_keyframes {
            let new_rot = match rot.surrounding(time) {
                KeyframeRange::Before(kf) => kf.value,
                KeyframeRange::After(kf) => kf.value,
                KeyframeRange::Between(kf1, kf2) => {
                    let start = kf1.time;
                    let end = kf2.time;
                    // The time factor that gives weight to the start or end frame during interpolation
                    let factor = (time - start) / (end - start);
                    Quaternion::interpolate(&rot.interpolation, factor, kf1.value, kf2.value)
                },
            };
            matrix_transforms.rotation = Mat3::from(new_rot);
        }
        if let Some(trans) = &self.translation_keyframes {
            let new_trans = match trans.surrounding(time) {
                KeyframeRange::Before(kf) => kf.value,
                KeyframeRange::After(kf) => kf.value,
                KeyframeRange::Between(kf1, kf2) => {
                    let start = kf1.time;
                    let end = kf2.time;
                    // The time factor that gives weight to the start or end frame during interpolation
                    let factor = (time - start) / (end - start);
                    Vec3::interpolate(&trans.interpolation, factor, kf1.value, kf2.value)
                },
            };
            matrix_transforms.translation = new_trans;
        }

        Mat4::from(matrix_transforms)
    }
}

enum KeyframeRange<'a, T> {
    /// The keyframe before the specified time
    Before(&'a Frame<T>),
    /// The keyframes that immediately surround the specified time
    Between(&'a Frame<T>, &'a Frame<T>),
    /// The keyframe after the specified time
    After(&'a Frame<T>),
}

#[derive(Debug)]
struct Keyframes<T> {
    frames: Vec<Frame<T>>,
    interpolation: Interpolation,
}

impl<T> Keyframes<T> {
    // Retrieves the keyframes immediately surrounding the given time
    // A time smaller than that of all keyframes will get back the first keyframe twice
    // A time larger than all keyframes gets the last keyframe twice
    fn surrounding(&self, time: f32) -> KeyframeRange<T> {
        // This unwrap is safe for partial_cmp as long as NaN is not one of the comparison values
        let index = match self.frames.binary_search_by(|frame| frame.time.partial_cmp(&time).unwrap()) {
            Ok(i) | Err(i) => i,
        };
        let left_index = index.saturating_sub(1);
        let right_index = min(index, self.frames.len() - 1);

        if left_index == 0 {
            KeyframeRange::Before(&self.frames[left_index])
        } else if right_index == self.frames.len() - 1 {
            KeyframeRange::After(&self.frames[right_index])
        } else {
            KeyframeRange::Between(&self.frames[left_index], &self.frames[right_index])
        }
    }
}

#[derive(Debug)]
struct Frame<T> {
    time: f32,
    value: T,
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

        let nodes = document.nodes().map(|node| {
            let children = node.children().map(|node| NodeId::from_gltf(&node)).collect();
            let node = Node::from_gltf(node, &meshes, &skins, &cameras, &lights);
            (node, children)
        });
        let nodes = Arc::new(NodeTree::from_ordered_nodes(nodes));

        let scenes: Vec<_> = document.scenes()
            .map(|scene| Arc::new(Scene::from_gltf(scene)))
            .collect();
        assert!(!scenes.is_empty(), "glTF file must have at least one scene");

        // Get the default scene, or just use the first scene if no default is provided
        let default_scene = document.default_scene().map(|scene| scene.index()).unwrap_or(0);

        let mut animations = HashMap::new();

        for anim_data in document.animations() {
            let anim_name = anim_data.name();
            for channel in anim_data.channels() {
                let reader = channel.reader(|buffer| Some(&buffers[buffer.index()]));
                let interpolation = match channel.sampler().interpolation() {
                    gltf::animation::Interpolation::Linear => Interpolation::Linear,
                    gltf::animation::Interpolation::Step => Interpolation::Step,
                    //TODO - In order to support cubicspline interpolation, we need to change how we're storing the data
                    // https://github.com/KhronosGroup/glTF/tree/master/specification/2.0#animation-samplerinterpolation
                    gltf::animation::Interpolation::CubicSpline => unimplemented!("Cubicspline interpolation is not supported!"),
                };

                // Create Animation
                let anims = animations.entry(NodeId::from_gltf(&channel.target().node())).or_insert_with(|| Vec::new());
                let anim = anims.iter_mut().find(|a: &&mut Animation| a.name.as_deref() == anim_name);
                let mut anim = match anim {
                    Some(anim) => anim,
                    None => {
                        anims.push(Animation::with_name(anim_name.map(String::from)));
                        // This unwrap is safe because we just pushed in an animation
                        anims.last_mut().unwrap()
                    }
                };

                // Create Keyframes
                let key_times = reader.read_inputs().expect("Animation detected with no sampler input values");
                match reader.read_outputs().expect("Animation detected with no sampler output values") {
                    Scales(scale) => {
                        assert!(anim.scale_keyframes.is_none());
                        let key_frames = Keyframes {frames: key_times.zip(scale)
                            .map(|(time, output)| Frame {time, value: Vec3::from(output)}).collect::<Vec<Frame<Vec3>>>(), interpolation};
                        anim.scale_keyframes = Some(key_frames);
                    },
                    Rotations(rot) => {
                        assert!(anim.rotation_keyframes.is_none());
                        let key_frames = Keyframes {frames: key_times.zip(rot.into_f32())
                            .map(|(time, [x, y, z, w])| Frame {time, value: Quaternion::from_xyzw(x, y, z, w)}).collect::<Vec<Frame<Quaternion>>>(), interpolation};
                        anim.rotation_keyframes = Some(key_frames);
                    },
                    Translations(trans) => {
                        assert!(anim.translation_keyframes.is_none());
                        let key_frames = Keyframes {frames: key_times.zip(trans)
                            .map(|(time, output)| Frame {time, value: Vec3::from(output)}).collect::<Vec<Frame<Vec3>>>(), interpolation};
                        anim.translation_keyframes = Some(key_frames);
                    },
                    _ => todo!(), //Morph stuff
                };
            }
        }

        Ok(Self {
            default_scene,
            nodes,
            scenes,
            default_joint_matrix_texture: None,
            scene_shader_geometry: HashMap::new(),
            scene_lights: HashMap::new(),
            scene_first_camera: None,
            scene_cameras: HashMap::new(),
            animations,
        })
    }

    /// Attempts to find the index of a scene with the given name. If name is None, the default
    /// scene is returned.
    fn find_scene(&self, name: Option<&str>) -> Result<usize, QueryError> {
        match name {
            None => Ok(self.default_scene),
            // This assumes that scene names are unique. If they are not unique, we might need to
            // search for all matching scenes and produce an error if there is more than one result
            Some(name) => self.scenes.iter()
                .position(|scene| scene.name.as_deref() == Some(name))
                .ok_or_else(|| QueryError::UnknownScene {name: name.to_string()}),
        }
    }
}

impl QueryBackend for GltfFile {
    fn query_geometry(&mut self, query: &GeometryQuery, display: &Display) -> Result<Arc<Vec<Arc<ShaderGeometry>>>, QueryError> {
        let GeometryQuery {models, animation} = query;

        //TODO: Restructure the code in this file to add animation support

        use GeometryFilter::*;
        let scene_index = match models {
            Scene {name} => self.find_scene(name.as_deref())?,
        };

        match self.scene_shader_geometry.get(&scene_index) {
            Some(scene_geo) => Ok(scene_geo.clone()),

            None => {
                // Need to split the borrow of self so we don't accidentally get two mut refs
                let Self {nodes, scenes, default_joint_matrix_texture, ..} = self;

                let scene = &scenes[scene_index];
                let node_world_transforms = nodes.world_transforms(&scene.roots);

                let mut scene_geo = Vec::new();
                for (parent_trans, node) in scene.roots.iter().flat_map(|&root| nodes.traverse(root)) {
                    let model_transform = parent_trans * node.transform;

                    if let Some((mesh, skin)) = node.mesh() {
                        let joint_matrices_tex = match skin {
                            Some(skin) => {
                                let joint_matrices = skin.joint_matrices(model_transform, &node_world_transforms);
                                //TODO: Find a way to cache this texture so we don't have to upload it over and over again
                                Arc::new(JointMatrixTexture::new(display, joint_matrices)?)
                            },

                            None => match default_joint_matrix_texture {
                                Some(tex) => tex.clone(),
                                None => {
                                    let tex = Arc::new(JointMatrixTexture::identity(display)?);
                                    *default_joint_matrix_texture = Some(tex.clone());
                                    tex
                                },
                            },
                        };

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
                self.scene_shader_geometry.insert(scene_index, scene_geo.clone());
                Ok(scene_geo)
            },
        }
    }

    fn query_camera(&mut self, query: &CameraQuery) -> Result<Arc<Camera>, QueryError> {
        use CameraQuery::*;
        match query {
            FirstInScene {name} => {
                let scene_index = self.find_scene(name.as_deref())?;

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
                let scene_index = self.find_scene(scene.as_deref())?;

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
            Scene {name} => self.find_scene(name.as_deref())?,
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
