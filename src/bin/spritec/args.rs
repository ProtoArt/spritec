use std::fs;
use std::io;
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
use spritec::config::TaskConfig;

/// A tool for generating pixel art from 3D models.
///
/// Rather than have you specify too many options on the command line, the spritec tool takes in a
/// configuration file in the TOML format.
#[derive(Debug, StructOpt)]
#[structopt(author = "The ProtoArt Team <https://protoart.me>")]
#[structopt(raw(setting = "ColoredHelp", setting = "DontCollapseArgsInUsage",
    setting = "ArgRequiredElseHelp"))]
pub struct AppArgs {
    /// Path to the configuration file to execute tasks from
    #[structopt(name = "config-file", default_value = "spritec.toml", parse(from_os_str))]
    config_path: PathBuf,
}

impl AppArgs {
    /// Loads the configuration file provided as an argument
    pub fn load_config(&self) -> Result<TaskConfig, Box<Error>> {
        Ok(toml::from_str(&fs::read_to_string(&self.config_path)?)?)
    }

    /// Determines the base directory of the configuration file, used to resolve all paths within
    /// the configuration file
    pub fn base_directory(&self) -> Result<PathBuf, io::Error> {
        self.config_path.canonicalize().and_then(|p| p.parent().ok_or_else(|| io::Error::new(
            io::ErrorKind::Other,
            "No parent directory for configuration path",
        )).map(|p| p.to_path_buf()))
    }
}
