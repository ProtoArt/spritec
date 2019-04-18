use std::path::PathBuf;

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
    pub config_path: PathBuf,
}
