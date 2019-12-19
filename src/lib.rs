//! The main spritec executable

// TOOL POLICY:
// - We add tools in order to help *us* improve our code
// - If they are not doing that, we can configure them or even elect to remove them
// - No tool is perfect and we are allowed to disagree with its results
// - If the tool warns about something that isn't actually an issue worth caring about, add it to
//   the list below and explain your change in your PR
// - We don't want to litter our code with #[allow] attributes unnecessarily, so try to either
//   globally disable that aspect of the tool or live with it and do what the tool says
// - If we make a mistake and find that one of these lints shouldn't have been added here, we can
//   always remove it later
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

pub mod config;
pub mod tasks;
pub mod renderer;

mod loaders;
mod scale;
mod camera;
mod model;
mod light;
