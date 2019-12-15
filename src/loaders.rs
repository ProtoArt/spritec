pub mod gltf;
pub mod obj;

use std::path::{Path, PathBuf};
use std::ffi::OsStr;

use thiserror::Error;

use crate::model::Model;

#[derive(Debug, Error)]
pub enum LoaderError {
    #[error("{0}")]
    ObjError(#[from] tobj::LoadError),
    #[error("{0}")]
    GltfError(#[from] ::gltf::Error),
    #[error("Unsupported file extension: {path:?}")]
    UnsupportedFileExtension {path: PathBuf},
}

/// Load a model based on the file extension of its path. OBJ files will be used as is. For glTF
/// files, the model will be used as loaded, regardless of the animations present in the file.
pub fn load_file(path: &Path) -> Result<Model, LoaderError> {
    match path.extension().and_then(OsStr::to_str) {
        Some("obj") => obj::load_file(path).map_err(Into::into),
        Some("gltf") | Some("glb") => gltf::load_file(path).map_err(Into::into),
        _ => Err(LoaderError::UnsupportedFileExtension {path: path.to_path_buf()}),
    }
}
