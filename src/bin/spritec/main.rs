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
use spritec::{config::TaskConfig, spritesheet::{Spritesheet, Pose}};
use rayon::prelude::*;

use crate::args::AppArgs;

fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    let args = AppArgs::from_args();
    let TaskConfig {spritesheets, poses} = args.load_config()?;
    let base_dir = args.base_directory()?;

    spritesheets.into_par_iter().map(|sheet| -> Result<(), Box<dyn Error + Send + Sync>> {
        let sheet = Spritesheet::from_config(sheet, &base_dir)?;
        sheet.generate()?;
        Ok(())
    }).collect::<Result<Vec<()>, Box<dyn Error + Send + Sync>>>()?;

    poses.into_par_iter().map(|pose| -> Result<(), Box<dyn Error + Send + Sync>> {
        let pose = Pose::from_config(pose, &base_dir)?;
        pose.generate()?;
        Ok(())
    }).collect::<Result<Vec<()>, Box<dyn Error + Send + Sync>>>()?;

    Ok(())
}
