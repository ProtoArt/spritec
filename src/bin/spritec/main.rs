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
    tasks::{self, Task, WeakFileCache},
    query3d::FileError,
    config::{TaskConfig, Spritesheet, Pose},
    renderer::ThreadRenderContext,
};

use crate::args::AppArgs;

fn main() -> Result<(), Box<dyn Error>> {
    let args = AppArgs::from_args();
    let TaskConfig {spritesheets, poses} = args.load_config()?;
    let base_dir = args.base_directory()?;

    let tasks = create_tasks(spritesheets, poses, &base_dir)?;

    let mut ctx = ThreadRenderContext::new()?;
    // This loop should not be parallelised. Rendering is done in parallel on the
    // GPU and is orchestrated by the renderer. Trying to do that here with threads
    // will only create contention.
    for task in tasks {
        task.execute(&mut ctx)?;
    }

    Ok(())
}

fn create_tasks(
    spritesheets: Vec<Spritesheet>,
    poses: Vec<Pose>,
    base_dir: &Path,
) -> Result<Vec<Task>, FileError> {
    let mut file_cache = WeakFileCache::default();

    let mut tasks = Vec::new();
    for sheet in spritesheets {
        tasks.push(tasks::generate_spritesheet_task(sheet, base_dir, &mut file_cache)?);
    }
    for pose in poses {
        tasks.push(tasks::generate_pose_task(pose, base_dir, &mut file_cache)?);
    }

    Ok(tasks)
}
