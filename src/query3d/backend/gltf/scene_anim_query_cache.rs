use crate::query3d::AnimationQuery;

/// A cache based on the scene index and the animation query
#[derive(Debug, Default)]
pub struct SceneAnimQueryCache<T> {
    //scene_shader_geometry: HashMap<usize, Arc<Vec<Arc<ShaderGeometry>>>>,
    x: T,
}

impl<T> SceneAnimQueryCache<T> {
    pub fn get(&self, scene_index: usize, anim_query: &AnimationQuery) -> Option<&T> {
        todo!()
    }

    pub fn insert(&mut self, scene_index: usize, anim_query: &AnimationQuery, value: T) {
        todo!()
    }
}
