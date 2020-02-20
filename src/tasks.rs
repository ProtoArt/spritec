mod file_cache;

pub use file_cache::*;

use std::sync::{Arc, Mutex};
use std::path::{Path, PathBuf};
use std::num::NonZeroU32;
use std::cmp::max;

use interpolation::lerp;
use thiserror::Error;

use crate::math::{Mat4, Vec3, Rgb, Milliseconds};
use crate::config;
use crate::scene::{CameraType, LightType};
use crate::query3d::{
    File,
    FileError,
    CameraQuery,
    GeometryQuery,
    GeometryFilter,
    AnimationQuery,
    AnimationPosition,
};
use crate::renderer::{
    ThreadRenderContext,
    DrawLayoutError,
    RenderJob,
    RenderNode,
    RenderLayout,
    LayoutType,
    RenderedImage,
    Size,
    Outline,
    Light,
    RenderLights,
    Camera,
    RenderCamera,
    FileQuery,
};

#[derive(Debug, Error)]
#[error(transparent)]
pub enum TaskError {
    DrawLayoutError(#[from] DrawLayoutError),
    ImageError(#[from] image::ImageError),
}

#[derive(Debug)]
pub struct Task {
    /// The absolute path to output the generated file
    pub output_path: PathBuf,
    /// The job to execute that generates the final image
    pub job: RenderJob,
}

impl Task {
    pub fn execute(self, ctx: &mut ThreadRenderContext) -> Result<(), TaskError> {
        let Self {output_path, job} = self;

        let image = job.execute(ctx)?;
        image.save(&output_path)?;

        Ok(())
    }
}

pub fn generate_pose_task(
    pose: config::Pose,
    base_dir: &Path,
    file_cache: &mut WeakFileCache,
) -> Result<Task, FileError> {
    let config::Pose {model, path, width, height, camera, scale, background, outline} = pose;

    let (file, geometry) = match model {
        config::PoseModel::GltfFrame {gltf, animation, time} => {
            let file = file_cache.open_gltf(&gltf.resolve(base_dir))?;

            let geometry = FileQuery {
                query: GeometryQuery {
                    models: GeometryFilter::all_in_default_scene(),
                    animation: Some(AnimationQuery {
                        name: animation,
                        position: AnimationPosition::Time(time),
                    }),
                },
                file: file.clone(),
            };

            (file, geometry)
        },

        config::PoseModel::Model(path) => {
            let file = file_cache.open(&path.resolve(base_dir))?;

            let geometry = FileQuery {
                query: GeometryQuery {
                    models: GeometryFilter::all_in_default_scene(),
                    animation: None,
                },
                file: file.clone(),
            };

            (file, geometry)
        },
    };

    let job = RenderJob {
        scale,
        root: RenderNode::RenderedImage(RenderedImage {
            size: Size {width, height},
            background,
            camera: preset_to_camera(&camera, &file),
            //TODO: Figure out how we want to allow lights to be configured
            lights: RenderLights::Lights(Arc::new(vec![Arc::new(Light {
                data: Arc::new(LightType::Directional {
                    name: None,
                    color: Rgb::white(),
                    intensity: 1.0,
                }),
                world_transform: Mat4::rotation_x((-60.0f32).to_radians()),
            })])),
            ambient_light: Rgb::white() * 0.5,
            geometry,
            outline: config_to_outline(outline),
        }),
    };

    Ok(Task {
        output_path: path.resolve(base_dir),
        job,
    })
}

pub fn generate_spritesheet_task(
    sheet: config::Spritesheet,
    base_dir: &Path,
    file_cache: &mut WeakFileCache,
) -> Result<Task, FileError> {
    let config::Spritesheet {path, animations, scale, background} = sheet;

    let cols = animations.iter().map(|anim| anim.frames.len()).max()
        .expect("zero-length animations are not supported");

    // Flatten all of the animations into a single list of nodes, inserting empty nodes along the
    // way to fill any gaps in the grid
    let mut nodes = Vec::new();
    for anim in animations {
        let extra = cols - anim.frames.len();

        let config::Animation {frames, frame_width, frame_height, camera, outline} = anim;
        let outline = config_to_outline(outline);

        let frame_size = Size {width: frame_width, height: frame_height};

        use config::AnimationFrames::*;
        match frames {
            GltfFrames {gltf, animation: name, start_time, end_time, steps} => {
                let file = file_cache.open_gltf(&gltf.resolve(base_dir))?;
                let camera = preset_to_camera(&camera, &file);

                //           | step       => weight
                // steps = 1 | 0          => 0.0
                // steps = 2 | 0, 1       => 0.0, 1.0
                // steps = 3 | 0, 1, 2    => 0.0, 0.5, 1.0
                // steps = 4 | 0, 1, 2, 3 => 0.0, 0.33, 0.66, 1.0
                let steps = steps.get();
                for step in 0..steps {
                    let weight = step as f32 / max(steps - 1, 1) as f32;

                    nodes.push(RenderNode::RenderedImage(RenderedImage {
                        size: frame_size,
                        background,
                        camera: camera.clone(),
                        //TODO: Figure out how we want to allow lights to be configured
                        lights: RenderLights::Lights(Arc::new(vec![Arc::new(Light {
                            data: Arc::new(LightType::Directional {
                                name: None,
                                color: Rgb::white(),
                                intensity: 1.0,
                            }),
                            world_transform: Mat4::rotation_x((-60.0f32).to_radians()),
                        })])),
                        ambient_light: Rgb::white() * 0.5,
                        geometry: FileQuery {
                            query: GeometryQuery {
                                models: GeometryFilter::all_in_default_scene(),

                                animation: Some(AnimationQuery {
                                    name: name.clone(),
                                    position: match end_time {
                                        Some(end_time) => AnimationPosition::Time(
                                            Milliseconds::from_msec(lerp(&start_time.to_msec(), &end_time.to_msec(), &weight))
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
                    let camera = preset_to_camera(&camera, &file);

                    nodes.push(RenderNode::RenderedImage(RenderedImage {
                        size: frame_size,
                        background,
                        camera,
                        //TODO: Figure out how we want to allow lights to be configured
                        lights: RenderLights::Lights(Arc::new(vec![Arc::new(Light {
                            data: Arc::new(LightType::Directional {
                                name: None,
                                color: Rgb::white(),
                                intensity: 1.0,
                            }),
                            world_transform: Mat4::rotation_x((-60.0f32).to_radians()),
                        })])),
                        ambient_light: Rgb::white() * 0.5,
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

    let job = RenderJob {
        scale,
        root: RenderNode::Layout(RenderLayout {
            nodes,
            layout: LayoutType::Grid {
                cols: NonZeroU32::new(cols).expect("zero-length animations are not supported"),
            },
        }),
    };

    Ok(Task {
        output_path: path.resolve(base_dir),
        job,
    })
}

fn preset_to_camera(cam: &config::PresetCamera, file: &Arc<Mutex<File>>) -> RenderCamera {
    use config::PresetCamera::*;
    match cam {
        &Perspective(persp) => config_to_camera(&persp.into()),
        Named(named) => named_to_camera(named, file),
        Custom(cam) => config_to_camera(cam),
    }
}

fn config_to_camera(cam: &config::Camera) -> RenderCamera {
    let &config::Camera {eye, target, aspect_ratio, fov_y, near_z, far_z} = cam;
    let field_of_view_y = fov_y.into();
    let cam_type = CameraType::Perspective {
        name: None,
        aspect_ratio,
        field_of_view_y,
        near_z,
        far_z,
    };

    RenderCamera::Camera(Arc::new(Camera {
        view: Mat4::look_at_rh(eye, target, Vec3::up()),
        projection: cam_type.to_projection(),
    }))
}

fn named_to_camera(named: &config::NamedCamera, file: &Arc<Mutex<File>>) -> RenderCamera {
    let config::NamedCamera {name, scene} = named;

    RenderCamera::Query(FileQuery {
        query: CameraQuery::Named {
            name: name.clone(),
            scene: scene.clone(),
        },

        file: file.clone(),
    })
}

fn config_to_outline(outline: config::Outline) -> Outline {
    let config::Outline {thickness, color} = outline;

    Outline {thickness, color}
}
