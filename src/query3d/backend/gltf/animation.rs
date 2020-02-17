use std::collections::HashMap;

use crate::query3d::query::AnimationPosition;
use crate::math::{Milliseconds, Vec3, Quaternion, Mat4, Mat3, Decompose};
use crate::scene::NodeId;

use super::keyframes::{Keyframes, Frame};
use super::interpolate::Interpolation;

#[derive(Debug, Default)]
pub struct Animation {
    pub name: Option<String>,
    pub scale_keyframes: Option<Keyframes<Vec3>>,
    pub rotation_keyframes: Option<Keyframes<Quaternion>>,
    pub translation_keyframes: Option<Keyframes<Vec3>>,
}

impl Animation {
    // with_name takes Option instead of String to include with Animations without names
    pub fn with_name(name: Option<String>) -> Self {
        Self {
            name,
            ..Self::default()
        }
    }

    /// Application of animation data by decomposing the current node's transformation matrix and
    /// replacing the different types of transforms if the keyframes for that transform exist
    pub fn apply_at(&self, transform_matrix: &Mat4, pos: &AnimationPosition) -> Mat4 {
        let mut matrix_transforms = transform_matrix.decompose();

        if let Some(keyframes) = &self.scale_keyframes {
            let new_value = keyframes.value_at(pos);
            matrix_transforms.scale = new_value;
        }
        if let Some(keyframes) = &self.rotation_keyframes {
            let new_value = keyframes.value_at(pos);
            matrix_transforms.rotation = Mat3::from(new_value);
        }
        if let Some(keyframes) = &self.translation_keyframes {
            let new_value = keyframes.value_at(pos);
            matrix_transforms.translation = new_value;
        }

        Mat4::from(matrix_transforms)
    }
}

pub fn from_animations<'a>(
    doc_anims: impl Iterator<Item=gltf::Animation<'a>>,
    buffers: &[gltf::buffer::Data],
) -> HashMap<NodeId, Vec<Animation>> {
    let mut animations = HashMap::new();

    for anim_data in doc_anims {
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

    animations
}
