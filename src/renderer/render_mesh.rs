use std::borrow::Cow;
use std::sync::Arc;

use vek::{Vec3, Mat4};
use glium::{
    VertexBuffer,
    IndexBuffer,
    VertexFormat,
    index::{self, PrimitiveType},
    vertex::{self, AttributeType},
    backend::glutin::headless::Headless,
};
use thiserror::Error;

use crate::model::{Mesh, Material};

#[derive(Debug, Error)]
pub enum RenderMeshCreationError {
    #[error("{0}")]
    IndexBufferCreationError(#[from] index::BufferCreationError),
    #[error("{0}")]
    VertexBufferCreationError(#[from] vertex::BufferCreationError),
}

#[derive(Debug)]
pub struct RenderModel {
    pub meshes: Vec<RenderMesh>,
}

/// The data for a given mesh, stored on the GPU
#[derive(Debug)]
pub struct RenderMesh {
    pub indices: IndexBuffer<u32>,
    pub positions: VertexBuffer<Vec3<f32>>,
    pub normals: VertexBuffer<Vec3<f32>>,
    pub material: Arc<Material>,
    pub model_transform: Mat4<f32>,
}

impl RenderMesh {
    pub fn new(display: &Headless, mesh: &Mesh) -> Result<Self, RenderMeshCreationError> {
        const POSITION_ATTR_TYPE: AttributeType = AttributeType::F32F32F32;
        let position_bindings: VertexFormat = Cow::Borrowed(&[
            // This name must correspond to the name in our shaders
            (Cow::Borrowed("position"), 0, POSITION_ATTR_TYPE, false),
        ]);
        const NORMAL_ATTR_TYPE: AttributeType = AttributeType::F32F32F32;
        let normal_bindings: VertexFormat = Cow::Borrowed(&[
            // This name must correspond to the name in our shaders
            (Cow::Borrowed("normal"), 0, NORMAL_ATTR_TYPE, false),
        ]);

        // NOTE: By using `immutable`, we are guranteeing that the data in these buffers will
        //   *never* change.
        // See: https://docs.rs/glium/0.26.0-alpha3/glium/buffer/enum.BufferMode.html
        Ok(Self {
            indices: IndexBuffer::immutable(display, PrimitiveType::TrianglesList, mesh.indices())?,
            // These calls to new_raw are safe assuming that the specified attribute types
            // correspond to the types of the items stored in the `positions` and `normals` fields
            // of Mesh. This should be the case because `Vec3<f32>` is #[repr(C)] and therefore it
            // should have the same layout as a C struct with three 32-bit floating point values.
            //TODO: Use BufferMode::Immutable here too. glium doesn't currently have a
            // new_raw_immutable method.
            positions: unsafe { VertexBuffer::new_raw(display, mesh.positions(), position_bindings,
                POSITION_ATTR_TYPE.get_size_bytes())? },
            normals: unsafe { VertexBuffer::new_raw(display, mesh.normals(), normal_bindings,
                NORMAL_ATTR_TYPE.get_size_bytes())? },
            material: mesh.material(),
            model_transform: mesh.transform(),
        })
    }
}
