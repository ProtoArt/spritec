pub mod obj;
pub mod gltf;

use std::sync::Arc;
use std::path::{Path, PathBuf};

use thiserror::Error;
use glium::texture::TextureCreationError;

use crate::renderer::{Display, ShaderGeometry, ShaderGeometryError, Camera, Light};

use super::query::{GeometryQuery, CameraQuery, LightQuery};

#[derive(Debug, Error)]
pub enum QueryError {
    #[error(transparent)]
    ShaderGeometryError(#[from] ShaderGeometryError),
    #[error(transparent)]
    TextureCreationError(#[from] TextureCreationError),

    #[error("Could not find scene named `{name}` in model file")]
    UnknownScene {name: String},

    #[error("Could not find animation named `{name}` in model file")]
    UnknownAnimation {name: String},
    #[error("Could not find any matching animation in model file")]
    NoAnimationFound,
    #[error("Multiple animations matched for a single node, please specify an animation name")]
    AmbiguousAnimation,

    #[error("Could not find any matching geometry in model file")]
    NoGeometryFound,

    #[error("Could not find camera named `{name}` in model file")]
    UnknownCamera {name: String},
    #[error("Could not find any matching cameras in model file")]
    NoCameraFound,

    #[error("Could not find light named `{name}` in model file")]
    UnknownLight {name: String},
    #[error("Could not find any matching lights in model file")]
    NoLightsFound,
}

pub trait QueryBackend {
    /// Attempts to find geometry matching the given query in this file. Only returns success
    /// if at least one geometry was found.
    fn query_geometry(&mut self, query: &GeometryQuery, display: &Display) -> Result<Arc<Vec<Arc<ShaderGeometry>>>, QueryError>;
    /// Attempts to find a camera matching the given query in this file.
    fn query_camera(&mut self, query: &CameraQuery) -> Result<Arc<Camera>, QueryError>;
    /// Attempts to find lights matching the given query in this file. Only returns success
    /// if at least one light was found.
    fn query_lights(&mut self, query: &LightQuery) -> Result<Arc<Vec<Arc<Light>>>, QueryError>;
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
    fn query_geometry(&mut self, query: &GeometryQuery, display: &Display) -> Result<Arc<Vec<Arc<ShaderGeometry>>>, QueryError> {
        use File::*;
        match self {
            Obj(objs) => objs.query_geometry(query, display),
            Gltf(gltf) => gltf.query_geometry(query, display),
        }
    }

    fn query_camera(&mut self, query: &CameraQuery) -> Result<Arc<Camera>, QueryError> {
        use File::*;
        match self {
            Obj(objs) => objs.query_camera(query),
            Gltf(gltf) => gltf.query_camera(query),
        }
    }

    fn query_lights(&mut self, query: &LightQuery) -> Result<Arc<Vec<Arc<Light>>>, QueryError> {
        use File::*;
        match self {
            Obj(objs) => objs.query_lights(query),
            Gltf(gltf) => gltf.query_lights(query),
        }
    }
}
