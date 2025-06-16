//! Placeholder CosmWasm module.

pub mod message;
pub mod types;

/// Minimal keeper used for compilation tests.
#[derive(Debug, Default, Clone)]
pub struct Keeper;

impl Keeper {
    pub fn new() -> Self {
        Self
    }
}
