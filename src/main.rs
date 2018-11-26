use std::fs::File;
use std::io::Error as IOError;

use fbx_direct::reader::{EventReader, FbxEvent::*, Error as FBXError};
use nalgebra::Point2D; //TODO: Add the nalgebra crate and lookup the closest struct to Point2D

// TODO: Pick a better name than AppError
#[derive(Debug)]
enum AppError {
    IOError(IOError),
    FBXError(FBXError),
}

impl From<IOError> for AppError {
    fn from(error: IOError) -> Self {
        AppError::IOError(error)
    }
}

impl From<FBXError> for AppError {
    fn from(error: FBXError) -> Self {
        AppError::FBXError(error)
    }
}

//TODO: Read the modules chapter and split this file up

/// The ID of a vertex within a specific mesh
/// Only to be used for the mesh where it came from
#[derive(Debug, Clone)]
struct VertexId(usize);
/// The ID of a edge within a specific mesh
/// Only to be used for the mesh where it came from
#[derive(Debug, Clone)]
struct EdgeId(usize);
/// The ID of a face within a specific mesh
/// Only to be used for the mesh where it came from
#[derive(Debug, Clone)]
struct FaceId(usize); //TODO: lookup "rust newtype pattern"
//TODO: Add good docs to everything
#[derive(Debug, Clone)]
struct MeshId(usize); //TODO: index into scene mesh array?

#[derive(Debug, Clone)]
struct Mesh {
    // Used to associate with other things
    id: MeshId, //TODO: Figure out the right type for this
    vertices: Vec<Point2D>,
    /// List of edge indexes (a, b) == (b, a)
    edges: Vec<(VertexId, VertexId)>,
    faces: Vec<Vec<VertexId>>,
    //TODO: Associate vertices with material IDs
}

impl Mesh {
    pub fn vertex(&self, vid: VertexId) -> Point2D {
        unimplemented!();
    }

    pub fn edge(&self, eid: EdgeId) -> (Point2D, Point2D) {
        unimplemented!();
    }

    pub fn face(&self, fid: usize) -> Vec<Point2D> {
        unimplemented!();
    }

    //TODO: Lookup "impl trait" syntax and learn about iterators
    pub fn vertices(&self) -> impl Iterator<Item=(VertexId, Point2D)> {
        unimplemented!();
    }

    pub fn edges(&self) -> impl Iterator<Item=(EdgeId, (Point2D, Point2D))> {
        unimplemented!();
    }

    pub fn faces(&self) -> impl Iterator<Item=(FaceId, Vec<Point2D>)> {
        unimplemented!();
    }
}

#[derive(Debug, Clone)]
struct Object {
    id: ObjectId, //TODO
    mesh: MeshId,
    //TODO: See if we need to have a list of material IDs here
}

#[derive(Debug, Clone, Default)] //TODO: Look up "Default trait" and "rust auto derive"
struct Scene {
    objects: Vec<Object>,
    meshes: Vec<Mesh>,
    materials: Vec<Material>, //TODO: figure out what to get from the material (at least diffuse color)
}

fn main() -> Result<(), AppError> {
    let file = File::open("samples/pyramid.fbx")?;
    let reader = EventReader::new(file);

    let mut scene = Scene::default();

    //TODO: Lookup "rust vecdeque"
    let mut found_objects = false;
    // For reference of how FBX meshes are exported, see this function: https://developer.blender.org/diffusion/BA/browse/master/io_scene_fbx/export_fbx_bin.py;2532b96844c121b710e1a1973d2a5ff824ab3be4$815
    for event_res in reader {
        match event_res? {
            StartFbx(_) => {},
            EndFbx => {},
            //TODO: Look up "rust pattern guard" to understand this "if" syntax
            StartNode {ref name, properties} if name == "Objects" => {
                found_objects = true;
            },
            StartNode {ref name, properties} if name == "Geometry" && found_objects => {
                let mesh = Mesh::new(); //TODO: implement new()
                // Get vertices, polygons, and edges, and create a mesh
                scene.meshes.push(mesh);
            },
            //TODO: Figure out materials, objects, cameras, lights, etc.
            StartNode {..} => {}, // ignore
            EndNode => indent -= 1,
            Comment(_) => {},
        }
    }

    Ok(())
}
