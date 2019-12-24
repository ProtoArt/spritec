#[derive(Debug)]
pub struct GeometryQuery {
    pub models: GeometryFilter,
    pub animation: Option<AnimationQuery>,
}

#[derive(Debug)]
pub enum GeometryFilter {
    /// Returns all the geometry in the given scene
    Scene {
        /// The name of the scene to look in or None if the default scene should be used
        name: Option<String>,
    },
}

impl GeometryFilter {
    pub fn all_in_default_scene() -> Self {
        GeometryFilter::Scene {name: None}
    }
}

#[derive(Debug)]
pub struct AnimationQuery {
    /// The name of the animation to look in or None if the default animation should be used
    pub name: Option<String>,
    /// The position in the animation to retrieve the current state from
    pub position: AnimationPosition,
}

/// Represents the position in a given animation
///
/// Not all variants are supported by all file formats.
#[derive(Debug)]
pub enum AnimationPosition {
    /// The time in the global animation clock
    Time(f32),
    /// An exact frame number
    Frame(usize),
}

#[derive(Debug)]
pub enum CameraQuery {
    /// Returns the first camera in the given scene
    FirstInScene {
        /// The name of the scene to look in or None if the default scene should be used
        name: Option<String>,
    },
}

#[derive(Debug)]
pub enum LightQuery {
    /// Returns the first light in the given scene
    FirstInScene {
        /// The name of the scene to look in or None if the default scene should be used
        name: Option<String>,
    },
}
