use crate::geometry::Mesh;

pub trait FileLoader {
    fn load_file(filepath: &str) -> Vec<Mesh>;
}
