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

    /// Iterates over the animation set, optionally only returning animations with the given name
    ///
    /// If `name` is None, all the animations will be returned
    pub fn filter<'a>(&'a self, name: Option<&'a str>) -> impl Iterator<Item=&Animation> + 'a {
        self.anims.iter().filter(move |anim| match name {
            None => true,
            Some(name) => anim.name.as_deref() == Some(name),
        })
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
    /// Creates a new animation with the given (optional) name
    pub fn with_name(name: Option<&str>) -> Self {
        Self {
            name: name.map(String::from),
            ..Self::default()
        }
    }

    /// Sets the keyframes from the given glTF data.
    ///
    /// Panics if this operation would overwrite any of the existing keyframes.
    pub fn set_keyframes(
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

    /// Applies the animation to the given transform by finding the value of its components at the
    /// given position.
    pub fn apply_at(&self, transform: &Mat4, pos: &AnimationPosition) -> Mat4 {
        let mut components = transform.decompose();

        if let Some(keyframes) = &self.scale {
            let new_value = keyframes.value_at(pos);
            components.scale = new_value;
        }
        if let Some(keyframes) = &self.rotation {
            let new_value = keyframes.value_at(pos);
            components.rotation = Mat3::from(new_value);
        }
        if let Some(keyframes) = &self.translation {
            let new_value = keyframes.value_at(pos);
            components.translation = new_value;
        }

        Mat4::from(components)
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

            let anim = match anim_set.animation_mut(anim_name) {
                Some(anim) => anim,
                None => anim_set.insert(Animation::with_name(anim_name)),
            };

            anim.set_keyframes(channel, buffers, interpolation);
        }
    }

    animations
}
