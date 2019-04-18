use std::path::PathBuf;
use std::collections::HashMap;

use serde::{Serialize, Deserialize};

/// A configuration that represents the spritesheets that spritec should generate
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpriteConfig {
    /// A mapping from output file path to the specification of the tiles of a spritesheet
    spritesheets: HashMap<PathBuf, Spritesheet>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Spritesheet {
    /// The 2D layout of the frames on this spritesheet.
    /// Each string can either be a filename or refer to something in `frames` (not both).
    grid: Vec<SpritesheetRow>,
    /// The specification of one or more frames. These can be referred to from within the grid in
    /// order to specify additional properites of individual frames.
    frames: Vec<FrameSpec>,
}

/// A single row of frames, represented by either explicitly writing out the frames or by
/// generating their names with a repetition expression
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum SpritesheetRow {
    Frames(Vec<String>),
    Repetition(Repetition),
}

/// A specification of the properties of one or more frames.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrameSpec {
    /// One or more filenames
    file: FrameSpecFile,
}

/// A single filename, a list of filenames, or a repetition
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum FrameSpecFile {
    Filename(PathBuf),
    Filenames(Vec<PathBuf>),
    Repetition(Repetition),
}

/// A repetitive pattern of a string. Expands into a list of strings where '{}' is replaced in the
/// pattern with each number from start to end, inclusive.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Repetition {
    pattern: String,
    start: usize,
    end: usize,
}

impl From<Repetition> for Vec<String> {
    fn from(rep: Repetition) -> Self {
        (rep.start..=rep.end).map(|i| rep.pattern.replace("{}", &i.to_string())).collect()
    }
}
