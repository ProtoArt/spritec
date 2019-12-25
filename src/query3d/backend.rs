pub mod obj;
pub mod gltf;
pub mod blend;

use std::path::{Path, PathBuf};

use thiserror::Error;

use crate::model::Model;

use super::query::GeometryQuery;

#[derive(Debug, Error)]
pub enum QueryError {
    #[error("Could not find scene named `{name}` in model file")]
    UnknownScene {name: String},
    #[error("Could not find animation named `{}` in model file",
        .name.as_deref().unwrap_or("<unnamed>"))]
    UnknownAnimation {name: Option<String>},
}

pub trait QueryBackend {
    fn query_geometry(&mut self, query: GeometryQuery) -> Result<Vec<&Model>, QueryError>;
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
    Blend(blend::BlendFile),
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
    fn query_geometry(&mut self, query: GeometryQuery) -> Result<Vec<&Model>, QueryError> {
        use File::*;
        match self {
            Obj(objs) => objs.query_geometry(query),
            Gltf(gltf) => gltf.query_geometry(query),
            Blend(blend) => blend.query_geometry(query),
        }
    }
}
