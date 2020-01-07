use crate::model::Scene;
use std::path::Path;

pub fn load_file(path: impl AsRef<Path>) -> Result<Scene, tobj::LoadError> {
    Scene::from_obj_file(path)
}
