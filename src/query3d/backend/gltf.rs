use std::path::Path;

/// Represents a single glTF file
#[derive(Debug)]
pub struct GltfFile {
}

impl GltfFile {
    /// Opens a glTF file
    pub fn open(path: &Path) -> Result<Self, tobj::LoadError> {
        unimplemented!()
    }
}
