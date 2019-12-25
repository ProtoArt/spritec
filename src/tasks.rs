mod file_cache;

pub use file_cache::*;

use std::path::Path;
use std::num::NonZeroU32;

use interpolation::lerp;

use crate::config;
use crate::camera::Camera;
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
    Render,
    Size,
    Outline,
    RenderCamera,
    RenderGeometry,
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
        root: RenderNode::Render(Render {
            size: Size {width, height},
            scale,
            background,
            camera: RenderCamera::Camera(camera.into()),
            lights: Vec::new(), // TODO
            models: vec![RenderGeometry {
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

                outline: outline.into(),
            }],
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
        let camera: Camera = camera.into();
        let outline: Outline = outline.into();

        let frame_size = Size {width: frame_width, height: frame_height};

        use config::AnimationFrames::*;
        match frames {
            GltfFrames {gltf, animation: name, start_time, end_time, steps} => {
                let file = file_cache.open_gltf(&gltf.resolve(base_dir))?;

                let steps = steps.get();
                for step in 0..steps {
                    let weight = step as f32 / steps as f32;

                    nodes.push(RenderNode::Render(Render {
                        size: frame_size,
                        scale,
                        background,
                        camera: RenderCamera::Camera(camera.clone()),
                        lights: Vec::new(), // TODO
                        models: vec![RenderGeometry {
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
                        }],
                    }));
                }
            },

            Models(models) => {
                // Use each model as a frame in the animation
                for (frame, model_path) in models.into_iter().enumerate() {
                    let file = file_cache.open(&model_path.resolve(base_dir))?;

                    nodes.push(RenderNode::Render(Render {
                        size: frame_size,
                        scale,
                        background,
                        camera: RenderCamera::Camera(camera.clone()),
                        lights: Vec::new(), //TODO
                        models: vec![RenderGeometry {
                            geometry: FileQuery {
                                query: GeometryQuery {
                                    models: GeometryFilter::all_in_default_scene(),

                                    animation: Some(AnimationQuery {
                                        name: None,
                                        position: AnimationPosition::Frame(frame),
                                    })
                                },

                                file,
                            },

                            outline: outline.clone(),
                        }],
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
        root: RenderNode::Layout(RenderLayout {
            nodes,
            layout: LayoutType::Grid {
                cols: NonZeroU32::new(cols).expect("zero-length animations are not supported"),
            },
        }),
    })
}
