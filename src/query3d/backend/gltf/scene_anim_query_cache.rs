use std::cmp::Ordering;
use std::collections::HashMap;

use approx::relative_eq;

use crate::query3d::{AnimationQuery, AnimationPosition};

/// If this value is too small, our cache will be bloated. If the value is too big, the cache will
/// incorrectly treat different values as the same.
const TOLERANCE: f32 = 0.0001;

/// Defines a total ordering for floating point numbers by treating numbers that are approximately
/// the same as equal.
///
/// This does not account for NaN values and will probably panic if they are encountered.
fn approx_cmp(left: f32, right: f32) -> Ordering {
    if relative_eq!(left, right, epsilon = TOLERANCE) {
        Ordering::Equal
    } else {
        // This will panic if we get NaN
        left.partial_cmp(&right).unwrap()
    }
}

/// Defines a total ordering for animation positions
fn pos_cmp(left: &AnimationPosition, right: &AnimationPosition) -> Ordering {
    use AnimationPosition::*;
    match (left, right) {
        // Arbitrarily order the different variants
        (Time(_), RelativeTime {..}) => Ordering::Less,
        (RelativeTime {..}, Time(_)) => Ordering::Greater,

        // Compare the values if the variants are the same
        (Time(left), Time(right)) => approx_cmp(left.to_msec(), right.to_msec()),

        (
            &RelativeTime {start_time: left_start_time, weight: left_weight},
            &RelativeTime {start_time: right_start_time, weight: right_weight},
        ) => {
            // Arbitrarily decided to order by start_time first and then weight
            match approx_cmp(left_start_time.to_msec(), right_start_time.to_msec()) {
                Ordering::Equal => approx_cmp(left_weight, right_weight),
                order => order,
            }
        },
    }
}

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
        self.find(pos).ok().map(|i| &self.entries[i].1)
    }

    pub fn insert(&mut self, pos: &AnimationPosition, value: T) {
        match self.find(pos) {
            Ok(_) => unreachable!("bug: attempt to re-insert a cached value"),
            Err(i) => self.entries.insert(i, (pos.clone(), value)),
        }
    }

    fn find(&self, pos: &AnimationPosition) -> Result<usize, usize> {
        self.entries.binary_search_by(|(epos, _)| pos_cmp(epos, pos))
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
