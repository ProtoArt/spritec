use std::collections::HashMap;

use crate::query3d::{AnimationQuery, AnimationPosition};

/// A cache for animation positions
///
/// The idea is that because floating point numbers can't be hashed, we'll instead define a
/// tolerance within which two floating point numbers are considered equal. We can then use
/// binary search to fairly accurately find a match. Note that we are ignoring NaN here.
#[derive(Debug)]
pub struct AnimPosCache<T> {
    entries: Vec<(AnimationPosition, T)>,
}

// Need to manually implement default because the derive requires T: Default
impl<T> Default for AnimPosCache<T> {
    fn default() -> Self {
        Self {
            entries: Default::default(),
        }
    }
}

impl<T> AnimPosCache<T> {
    pub fn get(&self, pos: &AnimationPosition) -> Option<&T> {
        todo!()
    }

    pub fn insert(&mut self, pos: &AnimationPosition, value: T) {
        todo!()
    }
}

/// A cache based on the scene index and the animation query
#[derive(Debug)]
pub struct SceneAnimQueryCache<T> {
    /// A cache of (scene index, animation name) to a cache for the animation positions
    cache: HashMap<(usize, Option<String>), AnimPosCache<T>>,
}

// Need to manually implement default because the derive requires T: Default
impl<T> Default for SceneAnimQueryCache<T> {
    fn default() -> Self {
        Self {
            cache: Default::default(),
        }
    }
}

impl<T> SceneAnimQueryCache<T> {
    pub fn get(&self, scene_index: usize, anim_query: &AnimationQuery) -> Option<&T> {
        let AnimationQuery {name, position} = anim_query;
        //TODO: There are potentially some (complex) ways to get around the allocation here, but
        // it's probably not worth the effort so I opted to ignore it for now.
        self.cache.get(&(scene_index, name.clone()))
            .and_then(|pos_cache| pos_cache.get(position))
    }

    pub fn insert(&mut self, scene_index: usize, anim_query: &AnimationQuery, value: T) {
        let AnimationQuery {name, position} = anim_query;
        let pos_cache = self.cache.entry((scene_index, name.clone())).or_default();
        pos_cache.insert(position, value);
    }
}
