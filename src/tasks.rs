mod file_cache;

pub use file_cache::*;

use std::sync::Arc;
use std::path::Path;
use std::num::NonZeroU32;

use vek::{Mat4, Vec3};
use interpolation::lerp;

use crate::config;
use crate::scene::CameraType;
use crate::query3d::{
    FileError,
    GeometryQuery,
    GeometryFilter,
    AnimationQuery,
    AnimationPosition,
};
use crate::renderer::{
    RenderJob,
    RenderNode,
    RenderLayout,
    LayoutType,
    RenderedImage,
    Size,
    Outline,
    Camera,
    RenderCamera,
    FileQuery,
};

pub fn generate_pose_job(
    pose: config::Pose,
    base_dir: &Path,
    file_cache: &mut WeakFileCache,
) -> Result<RenderJob, FileError> {
    let config::Pose {model, path, width, height, camera, scale, background, outline} = pose;

    Ok(RenderJob {
        output_path: path.resolve(base_dir),
        scale,
        root: RenderNode::RenderedImage(RenderedImage {
            size: Size {width, height},
            background,
            camera: RenderCamera::Camera(Arc::new(config_to_camera(camera))),
            lights: Vec::new(), // TODO
            geometry: match model {
                config::PoseModel::GltfFrame {gltf, animation, time} => FileQuery {
                    query: GeometryQuery {
                        models: GeometryFilter::all_in_default_scene(),
                        animation: Some(AnimationQuery {
                            name: animation,
                            position: AnimationPosition::Time(time.unwrap_or(0.0)),
                        }),
                    },
                    file: file_cache.open_gltf(&gltf.resolve(base_dir))?,
                },

                config::PoseModel::Model(path) => FileQuery {
                    query: GeometryQuery {
                        models: GeometryFilter::all_in_default_scene(),
                        animation: None,
                    },
                    file: file_cache.open(&path.resolve(base_dir))?,
                },
            },
            outline: config_to_outline(outline),
        }),
    })
}

pub fn generate_spritesheet_job(
    sheet: config::Spritesheet,
    base_dir: &Path,
    file_cache: &mut WeakFileCache,
) -> Result<RenderJob, FileError> {
    let config::Spritesheet {path, animations, scale, background} = sheet;

    let cols = animations.iter().map(|anim| anim.frames.len()).max()
        .expect("zero-length animations are not supported");

    // Flatten all of the animations into a single list of nodes, inserting empty nodes along the
    // way to fill any gaps in the grid
    let mut nodes = Vec::new();
    for anim in animations {
        let extra = cols - anim.frames.len();

        let config::Animation {frames, frame_width, frame_height, camera, outline} = anim;
        let camera = config_to_camera(camera);
        let outline = config_to_outline(outline);

        let frame_size = Size {width: frame_width, height: frame_height};

        use config::AnimationFrames::*;
        match frames {
            GltfFrames {gltf, animation: name, start_time, end_time, steps} => {
                let file = file_cache.open_gltf(&gltf.resolve(base_dir))?;

                let steps = steps.get();
                for step in 0..steps {
                    let weight = step as f32 / steps as f32;

                    nodes.push(RenderNode::RenderedImage(RenderedImage {
                        size: frame_size,
                        background,
                        camera: RenderCamera::Camera(Arc::new(camera.clone())),
                        lights: Vec::new(), // TODO
                        geometry: FileQuery {
                            query: GeometryQuery {
                                models: GeometryFilter::all_in_default_scene(),

                                animation: Some(AnimationQuery {
                                    name: name.clone(),
                                    position: match end_time {
                                        Some(end_time) => AnimationPosition::Time(
                                            lerp(&start_time, &end_time, &weight)
                                        ),

                                        None => AnimationPosition::RelativeTime {
                                            start_time,
                                            weight,
                                        },
                                    }
                                })
                            },

                            file: file.clone(),
                        },
                        outline: outline.clone(),
                    }));
                }
            },

            Models(models) => {
                // Use each model as a frame in the animation
                for model_path in models {
                    let file = file_cache.open(&model_path.resolve(base_dir))?;

                    nodes.push(RenderNode::RenderedImage(RenderedImage {
                        size: frame_size,
                        background,
                        camera: RenderCamera::Camera(Arc::new(camera.clone())),
                        lights: Vec::new(), //TODO
                        geometry: FileQuery {
                            query: GeometryQuery {
                                models: GeometryFilter::all_in_default_scene(),
                                // Use the default state of the scene
                                animation: None,
                            },

                            file,
                        },
                        outline: outline.clone(),
                    }));
                }
            },
        }

        // Fill out the rest of the row with extra empty cells
        for _ in 0..extra {
            nodes.push(RenderNode::Empty {size: frame_size});
        }
    }

    Ok(RenderJob {
        output_path: path.resolve(base_dir),
        scale,
        root: RenderNode::Layout(RenderLayout {
            nodes,
            layout: LayoutType::Grid {
                cols: NonZeroU32::new(cols).expect("zero-length animations are not supported"),
            },
        }),
    })
}

fn config_to_camera(cam: config::PresetCamera) -> Camera {
    use config::PresetCamera::*;
    let cam = match cam {
        Perspective(persp) => persp.into(),
        Custom(cam) => cam,
    };

    let config::Camera {eye, target, aspect_ratio, fov_y, near_z, far_z} = cam;
    let cam_type = CameraType::Perspective {
        aspect_ratio,
        field_of_view_y: fov_y,
        near_z,
        far_z,
    };

    Camera {
        view: Mat4::look_at_rh(eye, target, Vec3::up()),
        projection: cam_type.to_projection(),
    }
}

fn config_to_outline(outline: config::Outline) -> Outline {
    let config::Outline {thickness, color} = outline;

    Outline {thickness, color}
}
