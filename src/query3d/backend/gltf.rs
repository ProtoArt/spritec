// JAMES ADDED - to be moved to separate file afterwards

use std::sync::Arc;
use std::path::Path;
use std::collections::HashMap;
use std::cmp::min;

use crate::query3d::query::AnimationPosition;
use crate::math::{Vec3, Quaternion, Mat3, Mat4, Decompose, Milliseconds};
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
        use Interpolation::*;
        match interp {
            Linear => {
                let [x, y, z] = interpolation::lerp(&prev_keyframe.into_array(), &next_keyframe.into_array(), &t);
                Vec3 {x, y, z}
            },
            Step => prev_keyframe,
        }
    }
}

impl Interpolate for Quaternion {
    fn interpolate(interp: &Interpolation, t: f32, prev_keyframe: Quaternion, next_keyframe: Quaternion) -> Quaternion {
        use Interpolation::*;
        match interp {
            Linear => Quaternion::slerp(prev_keyframe, next_keyframe, t),
            Step => prev_keyframe,
        }
    }
}

#[derive(Debug, Default)]
struct Animation {
    name: Option<String>,
    scale_keyframes: Option<Keyframes<Vec3>>,
    rotation_keyframes: Option<Keyframes<Quaternion>>,
    translation_keyframes: Option<Keyframes<Vec3>>,
}

impl Animation {
    // with_name takes Option instead of String to include with Animations without names
    fn with_name(name: Option<String>) -> Self {
        Self {
            name,
            ..Self::default()
        }
    }

    /// Application of animation data by decomposing the current node's transformation matrix and
    /// replacing the different types of transforms if the keyframes for that transform exist
    fn apply_at(&self, transform_matrix: &Mat4, anim_pos_time: &AnimationPosition) -> Mat4 {
        use interpolation::lerp;
        let mut matrix_transforms = transform_matrix.decompose();

        if let Some(value) = &self.scale_keyframes {
            let time = match *anim_pos_time {
                AnimationPosition::Time(t) => t,
                AnimationPosition::RelativeTime{start_time, weight} => {
                    Milliseconds::from_msec(lerp(&start_time.to_msec(), &value.end_time().to_msec(), &weight))
                },
            };
            let new_value = match value.surrounding(time) {
                KeyframeRange::Before(kf) => kf.value,
                KeyframeRange::After(kf) => kf.value,
                KeyframeRange::Between(kf1, kf2) => {
                    let start = kf1.time;
                    let end = kf2.time;
                    // The time factor that gives weight to the start or end frame during interpolation
                    let factor = (time.to_msec() - start.to_msec()) / (end.to_msec() - start.to_msec());
                    Vec3::interpolate(&value.interpolation, factor, kf1.value, kf2.value)
                },
            };
            matrix_transforms.scale = new_value;
        }
        if let Some(value) = &self.rotation_keyframes {
            let time = match *anim_pos_time {
                AnimationPosition::Time(t) => t,
                AnimationPosition::RelativeTime{start_time, weight} => {
                    Milliseconds::from_msec(lerp(&start_time.to_msec(), &value.end_time().to_msec(), &weight))
                },
            };
            let new_value = match value.surrounding(time) {
                KeyframeRange::Before(kf) => kf.value,
                KeyframeRange::After(kf) => kf.value,
                KeyframeRange::Between(kf1, kf2) => {
                    let start = kf1.time;
                    let end = kf2.time;
                    // The time factor that gives weight to the start or end frame during interpolation
                    let factor = (time.to_msec() - start.to_msec()) / (end.to_msec() - start.to_msec());
                    Quaternion::interpolate(&value.interpolation, factor, kf1.value, kf2.value)
                },
            };
            matrix_transforms.rotation = Mat3::from(new_value);
        }
        if let Some(value) = &self.translation_keyframes {
            let time = match *anim_pos_time {
                AnimationPosition::Time(t) => t,
                AnimationPosition::RelativeTime{start_time, weight} => {
                    Milliseconds::from_msec(lerp(&start_time.to_msec(), &value.end_time().to_msec(), &weight))
                },
            };
            let new_value = match value.surrounding(time) {
                KeyframeRange::Before(kf) => kf.value,
                KeyframeRange::After(kf) => kf.value,
                KeyframeRange::Between(kf1, kf2) => {
                    let start = kf1.time;
                    let end = kf2.time;
                    // The time factor that gives weight to the start or end frame during interpolation
                    let factor = (time.to_msec() - start.to_msec()) / (end.to_msec() - start.to_msec());
                    Vec3::interpolate(&value.interpolation, factor, kf1.value, kf2.value)
                },
            };
            matrix_transforms.translation = new_value;
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
    /// Retrieves the keyframes immediately surrounding the given time
    /// A time smaller than that of all keyframes will get back the first keyframe twice
    /// A time larger than all keyframes gets the last keyframe twice
    fn surrounding(&self, time: Milliseconds) -> KeyframeRange<T> {
        // This unwrap is safe for partial_cmp as long as NaN is not one of the comparison values
        let index = match self.frames.binary_search_by(|frame| frame.time.partial_cmp(&time).unwrap()) {
            Ok(i) | Err(i) => i,
        };

        if index == 0 {
            KeyframeRange::After(&self.frames[index])
        } else if index == self.frames.len() {
            KeyframeRange::Before(&self.frames[index - 1])
        } else {
            let left_index = index - 1;
            let right_index = min(index, self.frames.len() - 1);
            KeyframeRange::Between(&self.frames[left_index], &self.frames[right_index])
        }
    }

    fn end_time(&self) -> Milliseconds {
        let last_index = self.frames.len() - 1;
        self.frames[last_index].time
    }
}

#[derive(Debug)]
struct Frame<T> {
    time: Milliseconds,
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
                use gltf::animation::util::ReadOutputs::*;
                let key_times = reader.read_inputs().expect("Animation detected with no sampler input values");
                match reader.read_outputs().expect("Animation detected with no sampler output values") {
                    Scales(scale) => {
                        assert!(anim.scale_keyframes.is_none(), "Did not expect animation with the same name to have multiple sets of scale keyframes");
                        let keyframes = Keyframes {
                            frames: key_times.zip(scale)
                                .map(|(time, output)| Frame {time: Milliseconds::from_sec(time), value: Vec3::from(output)}).collect::<Vec<Frame<Vec3>>>(),
                            interpolation,
                        };
                        anim.scale_keyframes = Some(keyframes);
                    },
                    Rotations(rot) => {
                        assert!(anim.rotation_keyframes.is_none(), "Did not expect animation with the same name to have multiple sets of rotation keyframes");
                        let keyframes = Keyframes {
                            frames: key_times.zip(rot.into_f32())
                                .map(|(time, [x, y, z, w])| Frame {time: Milliseconds::from_sec(time), value: Quaternion::from_xyzw(x, y, z, w)}).collect::<Vec<Frame<Quaternion>>>(),
                            interpolation,
                        };
                        anim.rotation_keyframes = Some(keyframes);
                    },
                    Translations(trans) => {
                        assert!(anim.translation_keyframes.is_none(), "Did not expect animation with the same name to have multiple sets of translation keyframes");
                        let keyframes = Keyframes {
                            frames: key_times.zip(trans)
                                .map(|(time, output)| Frame {time: Milliseconds::from_sec(time), value: Vec3::from(output)}).collect::<Vec<Frame<Vec3>>>(),
                            interpolation,
                        };
                        anim.translation_keyframes = Some(keyframes);
                    },
                    MorphTargetWeights(_) => unimplemented!("Morph target animations are not supported yet"),
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

        let mut node_tree = self.nodes.clone();

        if let Some(anim_query) = animation {
            let mut animation_found = false;
            node_tree = Arc::new(self.nodes.try_with_replacements(|node: &Node| -> Result<Option<Node>, QueryError> {
                match self.animations.get(&node.id) {
                    // If the node has a list of animations, look for the animations that match the AnimationQuery name
                    Some(anims) => {
                        // anim is the animation that will modify the transformation matrix of the current node
                        let anim = anims.iter().find(|anim| match &anim_query.name {
                            None => true,
                            name@Some(_) if &anim.name == name => true,
                            _ => false,
                        });

                        // Return if no animation matches the name in the animation query
                        let anim = match anim {
                            Some(anim) => anim,
                            None => return Ok(None),
                        };
                        animation_found = true;

                        // Create and set the new transformation matrix of the current node
                        let new_transform = anim.apply_at(&node.transform, &anim_query.position);
                        Ok(Some(node.with_transform(new_transform)))
                    },
                    None => Ok(None),
                }
            })?);

            if !animation_found {
                match &anim_query.name {
                    Some(name) => return Err(QueryError::UnknownAnimation {name: name.to_string()}),
                    None => return Err(QueryError::NoAnimationFound),
                }
            }
        }

        use GeometryFilter::*;
        let scene_index = match models {
            Scene {name} => self.find_scene(name.as_deref())?,
        };

        match self.scene_shader_geometry.get(&scene_index) {
            //Some(scene_geo) => Ok(scene_geo.clone()),

            _ => {
                // Need to split the borrow of self so we don't accidentally get two mut refs
                let Self {nodes, scenes, default_joint_matrix_texture, ..} = self;

                let scene = &scenes[scene_index];
                let node_world_transforms = nodes.world_transforms(&scene.roots);

                let mut scene_geo = Vec::new();
                for (parent_trans, node) in scene.roots.iter().flat_map(|&root| node_tree.traverse(root)) {
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
