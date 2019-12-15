//! The spritec command line interface

// See TOOL POLICY in src/lib.rs
#![deny(clippy::all)] // Deny clippy warnings when running clippy (used for CI)
#![allow(
    clippy::identity_op,
    clippy::let_and_return,
    clippy::cast_lossless,
    clippy::redundant_closure,
    clippy::len_without_is_empty,
    clippy::large_enum_variant,
)]
#![deny(bare_trait_objects)] // Prefer Box<dyn Trait> over Box<Trait>

mod args;

use std::error::Error;

use structopt::StructOpt;
use spritec::{config::TaskConfig, tasks::{Spritesheet, Pose}};

use crate::args::AppArgs;

fn main() -> Result<(), Box<dyn Error>> {
    let args = AppArgs::from_args();
    let TaskConfig {spritesheets, poses} = args.load_config()?;
    let base_dir = args.base_directory()?;

    // These loops should not be parallelised. Rendering is done in parallel on the
    // GPU and is orchestrated by the renderer. Trying to do that here with threads
    // will only create contention.
    for sheet in spritesheets {
        let sheet = Spritesheet::from_config(sheet, &base_dir)?;
        sheet.generate()?;
    }
    for pose in poses {
        let pose = Pose::from_config(pose, &base_dir)?;
        pose.generate()?;
    }

    Ok(())
}
