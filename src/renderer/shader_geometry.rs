use std::borrow::Cow;
use std::sync::Arc;

use glium::{
    VertexBuffer,
    IndexBuffer,
    VertexFormat,
    Texture2d,
    texture::TextureCreationError,
    index::{self, PrimitiveType},
    vertex::{self, AttributeType},
};
use thiserror::Error;

use crate::math::{Vec2, Vec3, Vec4, Mat4};
use crate::scene::{Geometry, TexImage};
use crate::renderer::{Display, ShaderMaterial, JointMatrixTexture};

#[derive(Debug, Error)]
#[error(transparent)]
pub enum ShaderGeometryError {
    IndexBufferCreationError(#[from] index::BufferCreationError),
    VertexBufferCreationError(#[from] vertex::BufferCreationError),
    TextureCreationError(#[from] TextureCreationError),
}

/// Geometry stored on the GPU
#[derive(Debug)]
pub struct ShaderGeometry {
    pub indices: IndexBuffer<u32>,
    pub positions: VertexBuffer<Vec3>,
    pub normals: VertexBuffer<Vec3>,
    pub tex_coords: VertexBuffer<Vec2>,
    pub joint_influences: VertexBuffer<[u32; 4]>,
    pub joint_weights: VertexBuffer<Vec4>,

    pub joint_matrices: Arc<JointMatrixTexture>,
    pub material: ShaderMaterial,
    /// The world transform of this geometry
    pub model_transform: Mat4,
}

impl ShaderGeometry {
    /// Uploads the given geometry to the GPU
    pub fn new(
        display: &Display,
        geo: &Geometry,
        joint_matrices: &Arc<JointMatrixTexture>,
        model_transform: Mat4,
        image_lookup: impl FnMut(&TexImage) -> Result<Arc<Texture2d>, TextureCreationError>,
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
        const TEX_COORD_ATTR_TYPE: AttributeType = AttributeType::F32F32;
        let tex_coord_bindings: VertexFormat = Cow::Borrowed(&[
            // This name must correspond to the name in our shaders
            (Cow::Borrowed("tex_coord"), 0, TEX_COORD_ATTR_TYPE, false),
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

        let Geometry {name: _, indices, positions, normals, tex_coords, joint_influences, joint_weights, material} = geo;

        let tex_coords = match tex_coords {
            Some(tex_coords) => Cow::Borrowed(tex_coords),
            None => {
                if material.texture.is_some() {
                    panic!("model had a texture in its material but no texture coordinates");
                }
                // Default to a set of zero coordinates for the texture coords
                Cow::Owned(vec![Vec2::zero(); positions.len()])
            },
        };

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

            _ => unreachable!("bug: did not expect geometry to only have either joint influences or joint weights"),
        };

        let material = ShaderMaterial::new(material, image_lookup)?;

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
            tex_coords: unsafe { VertexBuffer::new_raw(display, tex_coords.as_ref(),
                tex_coord_bindings, TEX_COORD_ATTR_TYPE.get_size_bytes())? },
            joint_influences: unsafe { VertexBuffer::new_raw(display, joint_influences.as_ref(),
                joint_influences_bindings, JOINT_INFLUENCES_ATTR_TYPE.get_size_bytes())? },
            joint_weights: unsafe { VertexBuffer::new_raw(display, joint_weights.as_ref(),
                joint_weights_bindings, JOINT_WEIGHTS_ATTR_TYPE.get_size_bytes())? },

            joint_matrices: joint_matrices.clone(),
            material,
            model_transform,
        })
    }
}
