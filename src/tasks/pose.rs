use std::io;
use std::path::{Path, PathBuf};
use std::num::NonZeroU32;

use thiserror::Error;
use image::RgbaImage;
use vek::Rgba;

use crate::config;
use crate::model::Scene;
use crate::camera::Camera;
use crate::loaders::{self, LoaderError, gltf::GltfFile};
use crate::renderer::{ThreadRenderContext, BeginRenderError};

#[derive(Debug, Error)]
#[error(transparent)]
pub enum PoseError {
    BeginRenderError(#[from] BeginRenderError),
    DrawError(#[from] glium::DrawError),
    SwapBuffersError(#[from] glium::SwapBuffersError),
    ReadError(#[from] glium::ReadError),
    IOError(#[from] io::Error),
}

#[derive(Debug)]
pub struct Pose {
    /// The scene to render
    scene: Scene,
    /// The absolute path to output the generated image
    path: PathBuf,
    /// The width at which to render each frame
    width: NonZeroU32,
    /// The height at which to render each frame
    height: NonZeroU32,
    /// The camera perspective from which to render each frame
    camera: Camera,
    /// A scale factor to apply to the generated image. The image is scaled without interpolation.
    /// The value must be greater than zero. (default: 1).
    scale: NonZeroU32,
    /// The background color of the generated image
    background: Rgba<f32>,
    /// The outline thickness to use when drawing the generated image
    outline_thickness: f32,
    /// The outline color to use when drawing the generated image
    outline_color: Rgba<f32>,
}

impl Pose {
    /// Generates a pose from the given configuration, resolving all paths relative to
    /// the given base directory
    pub fn from_config(pose: config::Pose, base_dir: &Path) -> Result<Self, LoaderError> {
        let config::Pose {model, path, width, height, camera, scale, background, outline} = pose;

        use config::PoseModel::*;
        let scene = match model {
            GltfFrame {gltf, animation, frame} => {
                let file_path = gltf.resolve(base_dir);
                let gltf_file = GltfFile::load_file(file_path)?;
                //FIXME: Change to `as_deref` instead of `as_ref().map(...)` when this issue is
                // resolved: https://github.com/rust-lang/rust/issues/50264
                // To understand this code: https://stackoverflow.com/a/31234028/551904
                gltf_file.frame(animation.as_ref().map(|s| &**s), frame)
            },
            Model(path) => {
                let model_path = path.resolve(base_dir);
                loaders::load_file(&model_path)?
            },
        };

        Ok(Self {
            scene,
            path: path.resolve(base_dir),
            width,
            height,
            camera: camera.into(),
            scale,
            background,
            outline_thickness: outline.thickness,
            outline_color: outline.color,
        })
    }

    /// Draw the image and write the result to the configured file
    pub fn generate(&self, ctx: &mut ThreadRenderContext) -> Result<(), PoseError> {
        let width = self.width.get();
        let height = self.height.get();

        let view = self.camera.view();
        let projection = self.camera.projection();

        // An unscaled version of the final image
        let (render_id, mut renderer) = ctx.begin_render((width, height))?;
        renderer.clear(self.background);
        renderer.render(&self.scene, view, projection,
            self.outline_thickness, self.outline_color)?;

        let image = ctx.finish_render(render_id)?;

        //TODO: Could optimize the case of scale == 1
        let scale = self.scale.get();
        let mut scaled_image = RgbaImage::new(width * scale, height * scale);
        crate::scale::scale(&image, &mut scaled_image);

        scaled_image.save(&self.path)?;

        Ok(())
    }
}
