use std::collections::HashMap;

use crate::query3d::query::AnimationPosition;
use crate::math::{Milliseconds, Vec3, Quaternion, Mat4, Mat3, Decompose};
use crate::scene::NodeId;

use super::keyframes::Keyframes;
use super::interpolate::Interpolation;

#[derive(Debug, Default, Clone)]
pub struct AnimationSet {
    anims: Vec<Animation>,
}

impl AnimationSet {
    pub fn animation_mut(&mut self, name: Option<&str>) -> Option<&mut Animation> {
        // By construction, there should only ever be one animation with a given name
        self.anims.iter_mut().find(|anim| anim.name.as_deref() == name)
    }

    /// Inserts an animation into the animation set. You guarantee that no previously inserted
    /// animation has the same name.
    pub fn insert(&mut self, anim: Animation) -> &mut Animation {
        self.anims.push(anim);
        // This unwrap is safe because we just pushed in an animation
        self.anims.last_mut().unwrap()
    }
}

#[derive(Debug, Default, Clone)]
pub struct Animation {
    pub name: Option<String>,
    pub scale: Option<Keyframes<Vec3>>,
    pub rotation: Option<Keyframes<Quaternion>>,
    pub translation: Option<Keyframes<Vec3>>,
}

impl Animation {
    // with_name takes Option instead of String to include with Animations without names
    pub fn with_name(name: Option<&str>) -> Self {
        Self {
            name: name.map(String::from),
            ..Self::default()
        }
    }

    pub fn add_keyframes(
        &mut self,
        channel: gltf::animation::Channel,
        buffers: &[gltf::buffer::Data],
        interpolation: Interpolation,
    ) {
        let reader = channel.reader(|buffer| Some(&buffers[buffer.index()]));
        let times = reader.read_inputs().expect("Animation detected with no sampler input values")
            .map(|time| Milliseconds::from_sec(time));

        use gltf::animation::util::ReadOutputs::*;
        match reader.read_outputs().expect("Animation detected with no sampler output values") {
            Scales(scales) => {
                assert!(self.scale.is_none(),
                    "Did not expect animation with the same name to have multiple sets of scale keyframes");

                let values = scales.map(Vec3::from);
                self.scale = Some(Keyframes::new(times, values, interpolation));
            },

            Rotations(rotations) => {
                assert!(self.rotation.is_none(),
                    "Did not expect animation with the same name to have multiple sets of rotation keyframes");

                let values = rotations.into_f32().map(|[x, y, z, w]| Quaternion::from_xyzw(x, y, z, w));
                self.rotation = Some(Keyframes::new(times, values, interpolation));
            },

            Translations(translations) => {
                assert!(self.translation.is_none(),
                    "Did not expect animation with the same name to have multiple sets of translation keyframes");

                let values = translations.map(Vec3::from);
                self.translation = Some(Keyframes::new(times, values, interpolation));
            },

            MorphTargetWeights(_) => unimplemented!("Morph target animations are not supported yet"),
        };
    }

    /// Application of animation data by decomposing the current node's transformation matrix and
    /// replacing the different types of transforms if the keyframes for that transform exist
    pub fn apply_at(&self, transform_matrix: &Mat4, pos: &AnimationPosition) -> Mat4 {
        let mut matrix_transforms = transform_matrix.decompose();

        if let Some(keyframes) = &self.scale {
            let new_value = keyframes.value_at(pos);
            matrix_transforms.scale = new_value;
        }
        if let Some(keyframes) = &self.rotation {
            let new_value = keyframes.value_at(pos);
            matrix_transforms.rotation = Mat3::from(new_value);
        }
        if let Some(keyframes) = &self.translation {
            let new_value = keyframes.value_at(pos);
            matrix_transforms.translation = new_value;
        }

        Mat4::from(matrix_transforms)
    }
}

pub fn from_animations<'a>(
    doc_anims: impl Iterator<Item=gltf::Animation<'a>>,
    buffers: &[gltf::buffer::Data],
) -> HashMap<NodeId, AnimationSet> {
    let mut animations: HashMap<NodeId, AnimationSet> = HashMap::new();

    for anim_data in doc_anims {
        let anim_name = anim_data.name();

        for channel in anim_data.channels() {
            let interpolation = channel.sampler().interpolation().into();

            // Create Animation
            let node_id = NodeId::from_gltf(&channel.target().node());
            let anim_set = animations.entry(node_id).or_default();

            let mut anim = match anim_set.animation_mut(anim_name) {
                Some(anim) => anim,
                None => anim_set.insert(Animation::with_name(anim_name)),
            };

            anim.add_keyframes(channel, buffers, interpolation);
        }
    }

    animations
}
