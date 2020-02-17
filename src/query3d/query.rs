use crate::math::Milliseconds;

#[derive(Debug, Clone)]
pub struct GeometryQuery {
    pub models: GeometryFilter,
    pub animation: Option<AnimationQuery>,
}

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
pub struct AnimationQuery {
    /// The name of the animation to look in or None if the default animation should be used
    pub name: Option<String>,
    /// The position in the animation to retrieve the current state from
    pub position: AnimationPosition,
}

/// Represents the position in a given animation
#[derive(Debug, Clone)]
pub enum AnimationPosition {
    /// The time in ms on the global animation clock
    Time(Milliseconds),
    /// The time interpolated between the given start time and up to the time of the last keyframe of the
    /// animation
    RelativeTime {
        /// The start time in ms
        start_time: Milliseconds,
        /// A value between 0.0 and 1.0 that specifies the interpolation factor between the
        /// provided start time and the end of the animation. The end of the animation is defined
        /// as the time of its last keyframe.
        weight: f32,
    },
}

#[derive(Debug, Clone)]
pub enum CameraQuery {
    /// Returns the first camera in the given scene
    FirstInScene {
        /// The name of the scene to look in or None if the default scene should be used
        name: Option<String>,
    },
    /// Returns the camera with the given name
    Named {
        /// The name of the camera to look for
        name: String,
        /// The name of the scene to look in or None if the default scene should be used
        scene: Option<String>,
    },
}

impl CameraQuery {
    pub fn first_in_default_scene() -> Self {
        CameraQuery::FirstInScene {name: None}
    }
}

#[derive(Debug, Clone)]
pub enum LightQuery {
    /// Returns all the lights in the given scene
    Scene {
        /// The name of the scene to look in or None if the default scene should be used
        name: Option<String>,
    },
}

impl LightQuery {
    pub fn all_in_default_scene() -> Self {
        LightQuery::Scene {name: None}
    }
}
