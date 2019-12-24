use std::path::Path;

/// Represents one or more OBJ files
///
/// With multiple OBJ files, each file can be treated as an animation frame, indexed by frame
/// number only.
#[derive(Debug)]
pub struct ObjFiles {
}

impl ObjFiles {
    /// Opens an OBJ file
    pub fn open(path: &Path) -> Result<Self, tobj::LoadError> {
        unimplemented!()
    }
}
