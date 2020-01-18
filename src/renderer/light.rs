use std::sync::Arc;

use crate::scene::LightType;
use crate::math::Mat4;

#[derive(Debug, Clone)]
pub struct Light {
    /// The type of light and its configuration
    pub data: Arc<LightType>,
    /// The world transform of the light
    pub world_transform: Mat4,
}
