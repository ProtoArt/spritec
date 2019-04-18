use std::fmt;
use std::str::FromStr;
use std::path::PathBuf;
use std::error::Error;

use structopt::{
    StructOpt,
    clap::{
        AppSettings::{
            ColoredHelp,
            DontCollapseArgsInUsage,
            ArgRequiredElseHelp,
        },
    },
};

/// A tool for generating pixel art spritesheets from 3D models.
///
/// The spritec tool will make some attempts to determine how to output a spritesheet for the model
/// you provide. For example, if you pass in a glTF file with some animations, spritec will output
/// a spritesheet with each animation rendered frame-by-frame. If you pass in an OBJ file, spritec
/// will output a single image of that model rendered in the specified pose.
#[derive(Debug, StructOpt)]
#[structopt(author = "The ProtoArt Team <https://protoart.me>")]
#[structopt(raw(setting = "ColoredHelp", setting = "DontCollapseArgsInUsage",
    setting = "ArgRequiredElseHelp"))]
pub struct AppArgs {
    /// Path to a .obj file or a .gltf file.
    #[structopt(parse(from_os_str))]
    model: PathBuf,
    /// The "output" path where the generated result should be written.
    /// The file format will be determined from the extension (.jpg, .png, .gif).
    ///
    /// This will default to the filename of the model with its extension replaced with `.png` and
    /// its parent path stripped away. That is, the default behaviour is that the model is
    /// outputted into the current directory.
    #[structopt(long = "output-path", short = "o", parse(from_os_str))]
    spritesheet: Option<PathBuf>,
    /// The size to render each frame
    #[structopt(long = "size", default_value = "32x32", raw(possible_values = "RenderSize::variants()", case_insensitive = "true"))]
    pub size: RenderSize,
    /// An additional scale factor multiplied against the size to get the total size of the
    /// outputted image. The image is scaled without interpolation. The value must be greater than
    /// 0.
    #[structopt(long = "scale-factor", short = "S", default_value = "1")]
    pub scale_factor: ScaleFactor,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RenderSize {
    D16,
    D32,
    D64,
    D128,
}

impl From<RenderSize> for (usize, usize) {
    fn from(size: RenderSize) -> Self {
        use RenderSize::*;
        match size {
            D16 => (16, 16),
            D32 => (32, 32),
            D64 => (64, 64),
            D128 => (128, 128),
        }
    }
}

impl RenderSize {
    fn variants() -> &'static [&'static str] {
        &["16x16", "32x32", "64x64", "128x128"]
    }
}

impl fmt::Display for RenderSize {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use RenderSize::*;
        write!(f, "{}", match self {
            D16 => "16x16",
            D32 => "32x32",
            D64 => "64x64",
            D128 => "128x128",
        })
    }
}

#[derive(Debug, Clone)]
pub struct InvalidRenderSize(String);

impl Error for InvalidRenderSize {}

impl fmt::Display for InvalidRenderSize {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Invalid size: '{}'", self.0)
    }
}

impl FromStr for RenderSize {
    type Err = InvalidRenderSize;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use RenderSize::*;
        Ok(match s {
             "16x16" => D16,
             "32x32" => D32,
             "64x64" => D64,
             "128x128" => D128,
             _ => Err(InvalidRenderSize(s.to_string()))?,
        })
    }
}

//TODO: The ScaleFactor type can be completely replaced with std::num::NonZeroUsize once Rust 1.35
// is released. That release contains a FromStr impl for NonZeroUsize
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ScaleFactor(usize);

impl From<ScaleFactor> for usize {
    fn from(ScaleFactor(factor): ScaleFactor) -> Self {
        factor
    }
}

impl fmt::Display for ScaleFactor {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone)]
pub struct InvalidScaleFactor(String);

impl Error for InvalidScaleFactor {}

impl fmt::Display for InvalidScaleFactor {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Invalid scale factor: '{}'", self.0)
    }
}

impl FromStr for ScaleFactor {
    type Err = InvalidScaleFactor;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "0" => Err(InvalidScaleFactor(s.to_string())),
            _ => s.parse().map(ScaleFactor).map_err(|_| InvalidScaleFactor(s.to_string())),
        }
    }
}
