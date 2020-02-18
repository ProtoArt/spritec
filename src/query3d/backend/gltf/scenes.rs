use std::sync::Arc;
use std::ops::Index;

use crate::scene::Scene;

use super::super::QueryError;

#[derive(Debug, Clone)]
pub struct Scenes {
    scenes: Vec<Arc<Scene>>,
    /// The default scene when no scene name is provided (index into `scenes`)
    default_scene: usize,
}

impl Scenes {
    pub fn new(scenes: Vec<Arc<Scene>>, default_scene: usize) -> Self {
        Self {
            scenes,
            default_scene,
        }
    }

    /// Attempts to find the index of a scene with the given name. If name is None, the default
    /// scene is returned.
    pub fn query(&self, name: Option<&str>) -> Result<usize, QueryError> {
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

impl Index<usize> for Scenes {
    type Output = Arc<Scene>;

    fn index(&self, index: usize) -> &Self::Output {
        &self.scenes[index]
    }
}
