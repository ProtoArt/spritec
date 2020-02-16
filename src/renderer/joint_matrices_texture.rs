use std::iter::once;

use glium::Texture2d;
use glium::texture::{MipmapsOption, TextureCreationError};

use crate::math::Mat4;
use crate::renderer::Display;

/// Stores the joint matrices as a texture so they can be used in the shader
///
/// This is a way to simulate dynamic storage in GLSL. The matrices are stored as one long texture
/// where each x coordinate is a 4D matrix. The columns of the matrix can be accessed using the 4D
/// values at each y coordinate 0 through 3. (See GLSL `texelFetch`)
///
/// Reference: https://webgl2fundamentals.org/webgl/lessons/webgl-skinning.html
#[derive(Debug)]
pub struct JointMatrixTexture(Texture2d);

impl JointMatrixTexture {
    pub fn new(
        display: &Display,
        joint_matrices: impl Iterator<Item=Mat4>,
    ) -> Result<Self, TextureCreationError> {
        // Separate the columns into separate Vecs representing the rows of the texture
        let mut col0 = Vec::new();
        let mut col1 = Vec::new();
        let mut col2 = Vec::new();
        let mut col3 = Vec::new();

        fn into_tuple([a, b, c, d]: [f32; 4]) -> (f32, f32, f32, f32) {
            (a, b, c, d)
        }

        for mat in joint_matrices {
            let [c0, c1, c2, c3] = mat.into_col_arrays();
            col0.push(into_tuple(c0));
            col1.push(into_tuple(c1));
            col2.push(into_tuple(c2));
            col3.push(into_tuple(c3));
        }

        // Create the data for the texture with 4 rows
        let tex_data = vec![col0, col1, col2, col3];
        let tex = Texture2d::with_mipmaps(display, tex_data, MipmapsOption::NoMipmap)?;

        Ok(JointMatrixTexture(tex))
    }

    /// Returns a joint matrix texture with a single identity matrix in it
    pub fn identity(display: &Display) -> Result<Self, TextureCreationError> {
        Self::new(display, once(Mat4::identity()))
    }

    pub fn as_texture(&self) -> &Texture2d {
        &self.0
    }
}
