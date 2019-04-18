use std::io;
use std::path::{Path, PathBuf};
use std::num::{NonZeroUsize, NonZeroU32};

use euc::buffer::Buffer2d;
use image::ImageBuffer;
use vek::Rgba;

use crate::config;
use crate::model::Model;
use crate::shader::Camera;
use crate::color::vek_rgba_to_image_rgba;
use crate::loaders::{self, LoaderError, gltf::GltfFile};
use crate::scale::scale_with;

#[derive(Debug)]
pub struct Pose {
    /// The model to render
    model: Model,
    /// The absolute path to output the generated image
    path: PathBuf,
    /// The width at which to render each frame
    width: NonZeroUsize,
    /// The height at which to render each frame
    height: NonZeroUsize,
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
        let model = match model {
            GltfFrame {gltf, animation, frame} => {
                let model_path = gltf.resolve(base_dir);
                let model = GltfFile::load_file(model_path)?;
                //FIXME: Change to `as_deref` instead of `as_ref().map(...)` when this issue is
                // resolved: https://github.com/rust-lang/rust/issues/50264
                // To understand this code: https://stackoverflow.com/a/31234028/551904
                model.frame(animation.as_ref().map(|s| &**s), frame)
            },
            Model(path) => {
                let model_path = path.resolve(base_dir);
                loaders::load_file(model_path)?
            },
        };

        Ok(Self {
            model,
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

    /// Returns the [width, height] dimensions of the generated image
    pub fn size(&self) -> [usize; 2] {
        [self.width.get(), self.height.get()]
    }

    /// Draw the image and write the result to the configured file
    pub fn generate(&self) -> Result<(), io::Error> {
        let size = self.size();
        // An unscaled version of the final image
        let mut color = Buffer2d::new(size, self.background);
        let mut depth = Buffer2d::new(size, 1.0);

        let [width, height] = size;

        let view = self.camera.view();
        let projection = self.camera.projection();

        crate::render(&mut color, &mut depth, view, projection, &self.model,
            self.outline_thickness, self.outline_color);

        //FIXME: Could optimize the case of scale == 1
        let scale = self.scale.get();
        let mut img = ImageBuffer::new(width as u32 * scale, height as u32 * scale);
        let img_size = [img.width() as usize, img.height() as usize];

        scale_with(img_size, &color, |[x, y], rgba| {
            let pixel = img.get_pixel_mut(x as u32, y as u32);
            *pixel = vek_rgba_to_image_rgba(rgba);
        });

        img.save(&self.path)
    }
}
