//! The spritec-preview WASM library

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
#![deny(unused_must_use)] // Ignoring Result is the source of many common bugs

pub mod image_buffer;
pub mod context;
pub mod wasm;
