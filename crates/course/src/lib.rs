//! The Mark One course: a learning path through the engine's component
//! library. Each Mark asks for one component, tests the player's circuit
//! against the engine's reference model, and unlocks the next.

pub mod bench;
pub mod catalog;
pub mod level;
pub mod programs;
pub mod progress;
pub mod reference;
mod rng;
pub mod state;
pub mod verify;

pub use catalog::{index_of, level, levels};
pub use level::{Level, Test};
pub use programs::REF_PROGRAMS;
pub use progress::{Ctx, Save, ROM_COST};
pub use reference::model;
pub use state::state_json;
pub use verify::{verify, VerifyResult};
