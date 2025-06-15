//! Command line interface for the wasm module.
//!
//! Offers subcommands to upload, instantiate and execute contracts as well as to
//! query state. These commands will integrate with the `clap` based CLI used by
//! the rest of Gears.
pub mod query;
pub mod tx;
