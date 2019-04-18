use std::io;
use std::path::{Path, PathBuf};
use std::borrow::Cow;
use std::num::{NonZeroUsize, NonZeroU32};

use rayon::prelude::*;
use euc::{buffer::Buffer2d, Target};
use image::ImageBuffer;
use vek::Rgba;

use crate::config;
use crate::camera::Camera;
use crate::color::vek_rgba_to_image_rgba;
use crate::loaders::{self, Model, LoaderError, gltf::GltfFile};
use crate::scale::{copy, scale_with};

/// The dimensions of any 2D array/grid
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GridSize {
    pub rows: usize,
    pub cols: usize,
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

    /// Returns the size of the spritesheet grid
    pub fn grid_size(&self) -> GridSize {
        GridSize {
            rows: self.animations.len(),
            cols: self.animations.iter().map(|a| a.frames.len()).max().unwrap_or_default(),
        }
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
    pub fn generate(&self) -> Result<(), io::Error> {
        let GridSize {rows, cols} = self.grid_size();
        let ImageSize {width, height} = self.image_size();

        // An unscaled version of the final image
        let mut sheet = Buffer2d::new([width as usize, height as usize], self.background);

        let mut y_offset = 0;
        for anim in &self.animations {
            let frame_size = anim.frame_size();
            let mut color = Buffer2d::new(frame_size, self.background);
            let mut depth = Buffer2d::new(frame_size, 1.0);

            let [frame_width, frame_height] = frame_size;

            let view = anim.camera.view();
            let projection = anim.camera.projection();

            for (i, frame) in anim.frames.iter().enumerate() {
                crate::render(&mut color, &mut depth, view, projection, &*frame, anim.outline);

                let x_offset = i * frame_width;
                // Unsafe because we are guaranteeing that the provided offset is not out of bounds
                unsafe { copy(&mut sheet, &color, (x_offset, y_offset)); }

                color.clear(self.background);
                depth.clear(1.0);
            }

            y_offset += frame_height;
        }

        let scale = self.scale.get();
        let mut img = ImageBuffer::new(width * scale, height * scale);
        let img_size = [img.width() as usize, img.height() as usize];

        scale_with(img_size, &sheet, |[x, y], rgba| {
            let pixel = img.get_pixel_mut(x as u32, y as u32);
            *pixel = vek_rgba_to_image_rgba(rgba);
        });

        img.save(&self.path)
    }
}

#[derive(Debug)]
pub struct Animation {
    frames: AnimationFrames,
    /// The width at which to render each frame
    frame_width: NonZeroUsize,
    /// The height at which to render each frame
    frame_height: NonZeroUsize,
    /// The camera perspective from which to render each frame
    camera: Camera,
    /// The outline thickness to use when drawing each frame
    outline: f32,
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
            outline,
        })
    }

    /// Returns the total width of all the animation frames adjacent to each other
    pub fn width(&self) -> u32 {
        (self.frames.len() * self.frame_width.get()) as u32
    }

    /// Returns the dimensions of each frame of this animation
    pub fn frame_size(&self) -> [usize; 2] {
        [self.frame_width.get(), self.frame_height.get()]
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
                    .map(loaders::load_file).collect::<Result<_, _>>()?
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
    outline: f32,
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
            outline,
        })
    }
}
