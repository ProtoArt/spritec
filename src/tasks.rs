mod file_cache;

pub use file_cache::*;

use std::path::Path;

use crate::config;
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
    Render,
    Size,
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
    unimplemented!()
}
