use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, Weak};
use std::collections::HashMap;

use crate::config;
use crate::query3d::{
    File,
    FileError,
    GeometryQuery,
    GeometryFilter,
    AnimationQuery,
    AnimationPosition,
};
use crate::job::{RenderJob, RenderNode, Render, Size, RenderCamera, RenderModel, FileQuery};

/// A cache for Rc<File> that will only keep weak references.
///
/// If the file has not been opened or if the current reference to the file has been dropped,
/// a new Rc<File> will be created. The purpose of using weak references explicitly is to ensure
/// that files are dropped in a timely manner, even if this cache is accidentally left around.
#[derive(Debug, Default)]
pub struct WeakFileCache {
    cache: HashMap<PathBuf, Weak<Mutex<File>>>,
}

impl WeakFileCache {
    /// Attempt to get a file from the cache
    ///
    /// Returns None if the file was never opened or if it has since been closed
    pub fn get(&self, path: &Path) -> Option<Arc<Mutex<File>>> {
        self.cache.get(path).and_then(|f| f.upgrade())
    }

    /// Opens a 3D file based on its extension
    pub fn open(&mut self, path: &Path) -> Result<Arc<Mutex<File>>, FileError> {
        self.open_with(path, File::open)
    }

    /// Opens a glTF file
    pub fn open_gltf(&mut self, path: &Path) -> Result<Arc<Mutex<File>>, FileError> {
        self.open_with(path, File::open_gltf)
    }

    /// Attempts to retrieve the file from the cache, or opens the file with the given function if
    /// it was never opened or has since been closed.
    fn open_with<F>(&mut self, path: &Path, open: F) -> Result<Arc<Mutex<File>>, FileError>
        where F: FnOnce(&Path) -> Result<File, FileError>,
    {
        match self.get(path) {
            Some(file) => Ok(file),
            None => {
                let file = Arc::new(Mutex::new(open(path)?));
                self.cache.insert(path.to_path_buf(), Arc::downgrade(&file));
                Ok(file)
            },
        }
    }
}

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
            models: vec![RenderModel {
                model: match model {
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
