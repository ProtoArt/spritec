use std::io;
use std::path::{Path, PathBuf};
use std::borrow::Cow;
use std::num::NonZeroU32;

use thiserror::Error;
use rayon::prelude::*;
use image::RgbaImage;
use vek::Rgba;

use crate::config;
use crate::model::Model;
use crate::camera::Camera;
use crate::loaders::{self, LoaderError, gltf::GltfFile};
use crate::renderer::{ThreadRenderContext, BeginRenderError};
use crate::scale::copy;

#[derive(Debug, Error)]
pub enum SpritesheetError {
    #[error("{0}")]
    BeginRenderError(#[from] BeginRenderError),
    #[error("{0}")]
    DrawError(#[from] glium::DrawError),
    #[error("{0}")]
    SwapBuffersError(#[from] glium::SwapBuffersError),
    #[error("{0}")]
    ReadError(#[from] glium::ReadError),
    #[error("{0}")]
    IOError(#[from] io::Error),
}

/// The dimensions of an image (in pixels)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ImageSize {
    pub width: u32,
    pub height: u32,
}

#[derive(Debug)]
pub struct Spritesheet {
    /// The absolute path to output the generated spritesheet
    path: PathBuf,
    /// Animations to include in the spritesheet
    animations: Vec<Animation>,
    /// A scale factor to apply to the generated images. Each image is scaled without interpolation.
    /// The value must be greater than zero. (default: 1).
    scale: NonZeroU32,
    /// The background color of the generated image
    background: Rgba<f32>,
}

impl Spritesheet {
    /// Generates a spritesheet from the given configuration, resolving all paths relative to
    /// the given base directory
    pub fn from_config(sheet: config::Spritesheet, base_dir: &Path) -> Result<Self, LoaderError> {
        let config::Spritesheet {path, animations, scale, background} = sheet;
        Ok(Self {
            path: path.resolve(base_dir),
            animations: animations.into_par_iter()
                .map(|a| Animation::from_config(a, base_dir))
                .collect::<Result<_, _>>()?,
            scale,
            background,
        })
    }

    /// Returns the size of image needed to uniformly layout a single animation per row.
    /// Does not take into account the scale factor.
    pub fn image_size(&self) -> ImageSize {
        ImageSize {
            width: self.animations.iter().map(|a| a.width()).max().unwrap_or_default(),
            height: self.animations.iter().map(|a| a.frame_height.get() as u32).sum(),
        }
    }

    /// Draw the spritesheet and write the result to the configured file
    pub fn generate(&self, ctx: &mut ThreadRenderContext) -> Result<(), SpritesheetError> {
        let ImageSize {width, height} = self.image_size();

        // An unscaled version of the final image
        let mut sheet = RgbaImage::new(width, height);

        let mut y_offset = 0;
        for anim in &self.animations {
            let frame_size = anim.frame_size();

            let (frame_width, frame_height) = frame_size;

            let view = anim.camera.view();
            let projection = anim.camera.projection();

            for (i, frame_model) in (0..).zip(anim.frames.iter()) {
                let (render_id, mut renderer) = ctx.begin_render(frame_size)?;
                renderer.clear(self.background);
                renderer.render(&*frame_model, view, projection,
                    anim.outline_thickness, anim.outline_color)?;

                let image = ctx.finish_render(render_id)?;

                let x_offset = i * frame_width;
                copy(&image, &mut sheet, (x_offset, y_offset));
            }

            y_offset += frame_height;
        }

        //FIXME: Could optimize the case of scale == 1
        let scale = self.scale.get();
        let mut scaled_image = RgbaImage::new(width * scale, height * scale);
        crate::scale::scale(&sheet, &mut scaled_image);

        scaled_image.save(&self.path)?;

        Ok(())
    }
}

#[derive(Debug)]
pub struct Animation {
    frames: AnimationFrames,
    /// The width at which to render each frame
    frame_width: NonZeroU32,
    /// The height at which to render each frame
    frame_height: NonZeroU32,
    /// The camera perspective from which to render each frame
    camera: Camera,
    /// The outline thickness to use when drawing each frame
    outline_thickness: f32,
    /// The outline color to use when drawing each frame
    outline_color: Rgba<f32>,
}

impl Animation {
    /// Generates an Animation from the given configuration, resolving all paths relative to
    /// the given base directory
    pub fn from_config(anim: config::Animation, base_dir: &Path) -> Result<Self, LoaderError> {
        let config::Animation {frames, frame_width, frame_height, camera, outline} = anim;

        Ok(Self {
            frames: AnimationFrames::from_config(frames, base_dir)?,
            frame_width,
            frame_height,
            camera: camera.into(),
            outline_thickness: outline.thickness,
            outline_color: outline.color,
        })
    }

    /// Returns the total width of all the animation frames adjacent to each other
    pub fn width(&self) -> u32 {
        self.frames.len() as u32 * self.frame_width.get()
    }

    /// Returns the [width, height] dimensions of each frame of this animation
    pub fn frame_size(&self) -> (u32, u32) {
        (self.frame_width.get(), self.frame_height.get())
    }
}

#[derive(Debug)]
pub enum AnimationFrames {
    GltfFrames {
        /// The glTF file to load each frame from
        model: GltfFile,
        /// The name of the animation to select. Can be omitted if there is no animation in the
        /// glTF file.
        animation: Option<String>,
        /// The frame to start the animation from
        start_frame: usize,
        /// The frame to end the animation at
        end_frame: usize,
    },
    /// An array of models to render for each frame
    Models(Vec<Model>),
}

impl AnimationFrames {
    /// Generates animation frames from the given configuration, resolving all paths relative to
    /// the given base directory
    pub fn from_config(frames: config::AnimationFrames, base_dir: &Path) -> Result<Self, LoaderError> {
        use config::AnimationFrames::*;
        Ok(match frames {
            GltfFrames {gltf, animation, start_frame, end_frame} => {
                let model_path = gltf.resolve(base_dir);
                let model = GltfFile::load_file(model_path)?;
                //FIXME: Change to `as_deref` instead of `as_ref().map(...)` when this issue is
                // resolved: https://github.com/rust-lang/rust/issues/50264
                // To understand this code: https://stackoverflow.com/a/31234028/551904
                let end_frame = end_frame.unwrap_or_else(|| {
                    model.end_frame(animation.as_ref().map(|s| &**s))
                });

                AnimationFrames::GltfFrames {model, animation, start_frame, end_frame}
            },
            Models(models) => AnimationFrames::Models(
                models.into_iter()
                    .map(|path| path.resolve(base_dir))
                    .map(|path| loaders::load_file(&path)).collect::<Result<_, _>>()?
            ),
        })
    }

    /// Returns the number of animation frames
    pub fn len(&self) -> usize {
        use AnimationFrames::*;
        match self {
            GltfFrames {start_frame, end_frame, ..} => end_frame - start_frame + 1,
            Models(frames) => frames.len(),
        }
    }

    /// Returns an iterator to the frames in this animation
    pub fn iter(&self) -> impl Iterator<Item=Cow<Model>> {
        // Having this method avoids the awkward syntax: (&self.frames).into_iter()
        self.into_iter()
    }
}

impl<'a> IntoIterator for &'a AnimationFrames {
    type Item = Cow<'a, Model>;
    type IntoIter = Box<dyn Iterator<Item=Self::Item> + 'a>;

    /// Returns an iterator over each model (frame) of this animation
    fn into_iter(self) -> Self::IntoIter {
        use AnimationFrames::*;
        match self {
            GltfFrames {model, animation, start_frame, end_frame} => Box::new((*start_frame..=*end_frame).map(move |frame| {
                //FIXME: Change to `as_deref` instead of `as_ref().map(...)` when this issue is
                // resolved: https://github.com/rust-lang/rust/issues/50264
                // To understand this code: https://stackoverflow.com/a/31234028/551904
                Cow::Owned(model.frame(animation.as_ref().map(|s| &**s), Some(frame)))
            })),
            Models(models) => Box::new(models.iter().map(Cow::Borrowed)),
        }
    }
}
