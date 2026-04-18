// velo-types: Shared primitives with zero external dependencies
// Requirement 1.2, 1.6

pub mod position;
pub mod events;
pub mod errors;

#[cfg(test)]
mod position_test;

pub use position::{Position, Range, Selection};
pub use events::EditorEvent;
pub use errors::VeloError;
