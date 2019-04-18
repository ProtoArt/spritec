pub mod gltf;
pub mod obj;

use std::path::{Path, PathBuf};
use std::error::Error;
use std::ffi::OsStr;
use std::fmt;

use crate::geometry::Mesh;

#[derive(Debug)]
pub enum LoaderError {
    ObjError(tobj::LoadError),
    GltfError(::gltf::Error),
    UnsupportedFileExtension {path: PathBuf},
}

impl From<tobj::LoadError> for LoaderError {
    fn from(err: tobj::LoadError) -> Self {
        LoaderError::ObjError(err)
    }
}

impl From<::gltf::Error> for LoaderError {
    fn from(err: ::gltf::Error) -> Self {
        LoaderError::GltfError(err)
    }
}

impl Error for LoaderError {}

impl fmt::Display for LoaderError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use LoaderError::*;
        match self {
            ObjError(err) => write!(f, "{}", err),
            GltfError(err) => write!(f, "{}", err),
            UnsupportedFileExtension {path} => {
                write!(f, "Unsupported file extension: `{}`", path.display())
            },
        }
    }
}

#[derive(Debug)]
pub struct Model {
    pub meshes: Vec<Mesh>,
}

/// Load a model based on the file extension of its path. OBJ files will be used as is. For glTF
/// files, the model will be used as loaded, regardless of the animations present in the file.
pub fn load_file(path: impl AsRef<Path>) -> Result<Model, LoaderError> {
    let path = path.as_ref();
    match path.extension().and_then(OsStr::to_str) {
        Some("obj") => obj::load_file(path).map_err(Into::into),
        Some("gltf") => gltf::load_file(path).map_err(Into::into),
        _ => Err(LoaderError::UnsupportedFileExtension {path: path.to_path_buf()}),
    }
}
