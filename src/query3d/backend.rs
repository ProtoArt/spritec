pub mod obj;
pub mod gltf;

use std::sync::Arc;
use std::path::{Path, PathBuf};

use thiserror::Error;

use crate::model::Model;
use crate::camera::Camera;
use crate::light::DirectionalLight;

use super::query::{GeometryQuery, CameraQuery, LightQuery};

#[derive(Debug, Error)]
pub enum QueryError {
    #[error("Could not find scene named `{name}` in model file")]
    UnknownScene {name: String},

    #[error("Could not find animation named `{}` in model file",
        .name.as_deref().unwrap_or("<unnamed>"))]
    UnknownAnimation {name: Option<String>},

    #[error("Could not find any cameras in model file")]
    NoCameraFound,

    #[error("Could not find any lights in model file")]
    NoLightsFound,
}

pub trait QueryBackend {
    fn query_geometry(&mut self, query: &GeometryQuery) -> Result<Vec<Arc<Model>>, QueryError>;
    fn query_camera(&mut self, query: &CameraQuery) -> Result<Arc<Camera>, QueryError>;
    fn query_lights(&mut self, query: &LightQuery) -> Result<Vec<Arc<DirectionalLight>>, QueryError>;
}

#[derive(Debug, Error)]
#[error(transparent)]
pub enum FileError {
    ObjError(#[from] tobj::LoadError),
    GltfError(#[from] ::gltf::Error),
    #[error("Unsupported file extension: {path:?}")]
    UnsupportedFileExtension {path: PathBuf},
}

#[derive(Debug)]
pub enum File {
    Obj(obj::ObjFile),
    Gltf(gltf::GltfFile),
}

impl File {
    /// Opens a 3D file based on its extension
    pub fn open(path: &Path) -> Result<Self, FileError> {
        match path.extension().and_then(|p| p.to_str()) {
            Some("obj") => Ok(File::Obj(obj::ObjFile::open(path)?)),
            Some("gltf") | Some("glb") => Ok(File::Gltf(gltf::GltfFile::open(path)?)),
            _ => Err(FileError::UnsupportedFileExtension {path: path.to_path_buf()}),
        }
    }

    /// Opens a glTF file
    pub fn open_gltf(path: &Path) -> Result<Self, FileError> {
        Ok(File::Gltf(gltf::GltfFile::open(path)?))
    }
}

impl QueryBackend for File {
    fn query_geometry(&mut self, query: &GeometryQuery) -> Result<Vec<Arc<Model>>, QueryError> {
        use File::*;
        match self {
            Obj(objs) => objs.query_geometry(query),
            Gltf(gltf) => gltf.query_geometry(query),
        }
    }

    fn query_camera(&mut self, query: &CameraQuery) -> Result<Arc<Camera>, QueryError> {
        use File::*;
        match self {
            Obj(objs) => objs.query_camera(query),
            Gltf(gltf) => gltf.query_camera(query),
        }
    }

    fn query_lights(&mut self, query: &LightQuery) -> Result<Vec<Arc<DirectionalLight>>, QueryError> {
        use File::*;
        match self {
            Obj(objs) => objs.query_lights(query),
            Gltf(gltf) => gltf.query_lights(query),
        }
    }
}
