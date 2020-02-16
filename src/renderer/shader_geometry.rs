use std::borrow::Cow;
use std::sync::Arc;

use glium::{
    VertexBuffer,
    IndexBuffer,
    VertexFormat,
    index::{self, PrimitiveType},
    vertex::{self, AttributeType},
};
use thiserror::Error;

use crate::math::{Vec3, Vec4, Mat4};
use crate::scene::{Geometry, Material};
use crate::renderer::{Display, JointMatrixTexture};

#[derive(Debug, Error)]
#[error(transparent)]
pub enum ShaderGeometryError {
    IndexBufferCreationError(#[from] index::BufferCreationError),
    VertexBufferCreationError(#[from] vertex::BufferCreationError),
}

/// Geometry stored on the GPU
#[derive(Debug)]
pub struct ShaderGeometry {
    pub indices: IndexBuffer<u32>,
    pub positions: VertexBuffer<Vec3>,
    pub normals: VertexBuffer<Vec3>,
    pub joint_influences: VertexBuffer<[u32; 4]>,
    pub joint_weights: VertexBuffer<Vec4>,

    pub joint_matrices: Arc<JointMatrixTexture>,
    pub material: Arc<Material>,
    /// The world transform of this geometry
    pub model_transform: Mat4,
}

impl ShaderGeometry {
    pub fn new(
        display: &Display,
        geo: &Geometry,
        joint_matrices: &Arc<JointMatrixTexture>,
        model_transform: Mat4,
    ) -> Result<Self, ShaderGeometryError> {
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
        const JOINT_INFLUENCES_ATTR_TYPE: AttributeType = AttributeType::U32U32U32U32;
        let joint_influences_bindings: VertexFormat = Cow::Borrowed(&[
            // This name must correspond to the name in our shaders
            (Cow::Borrowed("joint_influences"), 0, JOINT_INFLUENCES_ATTR_TYPE, false),
        ]);
        const JOINT_WEIGHTS_ATTR_TYPE: AttributeType = AttributeType::F32F32F32F32;
        let joint_weights_bindings: VertexFormat = Cow::Borrowed(&[
            // This name must correspond to the name in our shaders
            (Cow::Borrowed("joint_weights"), 0, JOINT_WEIGHTS_ATTR_TYPE, false),
        ]);

        let Geometry {name: _, indices, positions, normals, joint_influences, joint_weights, material} = geo;

        let (joint_influences, joint_weights) = match (joint_influences, joint_weights) {
            (Some(joint_influences), Some(joint_weights)) => {
                (Cow::Borrowed(joint_influences), Cow::Borrowed(joint_weights))
            },

            (None, None) => {
                // This only works because we use JointMatrixTexture::identity as the default value
                // for joint_matrices. That means that an index of 0 in joint_influences will
                // always give you the identity matrix. By setting the weights to
                // (1.0, 0.0, 0.0, 0.0), we ensure that a single identity matrix will be multipled
                // in the shader.
                let joint_influences = Cow::Owned(vec![[0; 4]; positions.len()]);
                let default_weights = Vec4 {x: 1.0, y: 0.0, z: 0.0, w: 0.0};
                let joint_weights = Cow::Owned(vec![default_weights; positions.len()]);

                (joint_influences, joint_weights)
            },

            _ => unreachable!("Did not expect geometry to only have either joint influences or joint weights"),
        };

        //let tex: Vec<Vec<(f32, f32, f32, f32)>> = unsafe { joint_matrices.as_texture().unchecked_read() };
        //println!("after: {:?}", tex);
        //let into_array = |(x, y, z, w)| [x, y, z, w];
        //let joint_matrix = |i| Mat4::from_col_arrays([
        //    into_array(tex[0][i]),
        //    into_array(tex[1][i]),
        //    into_array(tex[2][i]),
        //    into_array(tex[3][i]),
        //]);
        //for (influences, weights) in joint_influences.iter().zip(joint_weights.iter()) {
        //    println!("joint_influences = {:?}", influences);
        //    println!("joint_weights = {:?}", weights);
        //    println!("joint_matrix(joint_influences[0]) = \n{}", joint_matrix(influences[0] as usize));
        //    println!("joint_matrix(joint_influences[1]) = \n{}", joint_matrix(influences[1] as usize));
        //    let skin_mat =
        //        joint_matrix(influences[0] as usize) * weights.x +
        //        joint_matrix(influences[1] as usize) * weights.y +
        //        joint_matrix(influences[2] as usize) * weights.z +
        //        joint_matrix(influences[3] as usize) * weights.w;
        //    println!("skin_mat =\n{}", skin_mat);
        //    println!("\n");
        //}

        // NOTE: By using `immutable`, we are guranteeing that the data in these buffers will
        //   *never* change.
        // See: https://docs.rs/glium/0.26.0/glium/buffer/enum.BufferMode.html
        Ok(Self {
            indices: IndexBuffer::immutable(display, PrimitiveType::TrianglesList, indices)?,

            // These calls to new_raw are safe assuming that the specified attribute types
            // correspond to the types of the items stored in data passed to `new_raw`. This should
            // be the case because `Vec3` is `#[repr(C)]` and therefore it should have the same
            // layout as a C struct with three 32-bit floating point values. Similar reasoning
            // applies to `[u32; 4]` and `Vec4`.
            //
            //TODO: Use BufferMode::Immutable here too. glium doesn't currently have a
            // new_raw_immutable method.
            positions: unsafe { VertexBuffer::new_raw(display, positions, position_bindings,
                POSITION_ATTR_TYPE.get_size_bytes())? },
            normals: unsafe { VertexBuffer::new_raw(display, normals, normal_bindings,
                NORMAL_ATTR_TYPE.get_size_bytes())? },
            joint_influences: unsafe { VertexBuffer::new_raw(display, joint_influences.as_ref(),
                joint_influences_bindings, JOINT_INFLUENCES_ATTR_TYPE.get_size_bytes())? },
            joint_weights: unsafe { VertexBuffer::new_raw(display, joint_weights.as_ref(),
                joint_weights_bindings, JOINT_WEIGHTS_ATTR_TYPE.get_size_bytes())? },

            joint_matrices: joint_matrices.clone(),
            material: material.clone(),
            model_transform,
        })
    }
}
