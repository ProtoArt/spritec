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
use std::path::Path;

use structopt::StructOpt;
use spritec::{
    tasks::{self, WeakFileCache},
    query3d::FileError,
    config::{TaskConfig, Spritesheet, Pose},
    renderer::{ThreadRenderContext, RenderJob},
};

use crate::args::AppArgs;

fn main() -> Result<(), Box<dyn Error>> {
    let args = AppArgs::from_args();
    let TaskConfig {spritesheets, poses} = args.load_config()?;
    let base_dir = args.base_directory()?;

    let jobs = create_jobs(spritesheets, poses, &base_dir)?;

    let mut ctx = ThreadRenderContext::new()?;
    // This loop should not be parallelised. Rendering is done in parallel on the
    // GPU and is orchestrated by the renderer. Trying to do that here with threads
    // will only create contention.
    for job in jobs {
        job.execute(&mut ctx)?;
    }

    Ok(())
}

fn create_jobs(
    spritesheets: Vec<Spritesheet>,
    poses: Vec<Pose>,
    base_dir: &Path,
) -> Result<Vec<RenderJob>, FileError> {
    let mut file_cache = WeakFileCache::default();

    let mut jobs = Vec::new();
    for sheet in spritesheets {
        jobs.push(tasks::generate_spritesheet_job(sheet, base_dir, &mut file_cache)?);
    }
    for pose in poses {
        jobs.push(tasks::generate_pose_job(pose, base_dir, &mut file_cache)?);
    }

    Ok(jobs)
}
