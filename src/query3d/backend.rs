pub mod obj;
pub mod gltf;
pub mod blend;

use std::path::{Path, PathBuf};

use thiserror::Error;

use crate::model::Model;

use super::query::GeometryQuery;

#[derive(Debug, Error)]
#[error(transparent)]
pub enum QueryError {
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
    Objs(obj::ObjFiles),
    Gltf(gltf::GltfFile),
    Blend(blend::BlendFile),
}

impl File {
    /// Opens a 3D file based on its extension
    pub fn open(path: &Path) -> Result<Self, FileError> {
        match path.extension().and_then(|p| p.to_str()) {
            Some("obj") => Ok(File::Objs(obj::ObjFiles::open(path)?)),
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
            Objs(objs) => objs.query_geometry(query),
            Gltf(gltf) => gltf.query_geometry(query),
            Blend(blend) => blend.query_geometry(query),
        }
    }
}
