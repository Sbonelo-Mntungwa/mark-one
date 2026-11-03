//! The Mark One engine: a mixed-level digital circuit simulator and the
//! component library it runs.
//!
//! - [`circuit`]: circuits as data (parts and wires).
//! - [`signal`]: the four values a net can carry.
//! - [`sim`]: switch-level transistors plus behavioural chips.
//! - [`components`]: every component, each with its spec, model and tests.
//! - [`workbench`]: open circuits that name their own pins, and the library
//!   of parts they may use.
//! - [`custom`]: designs saved and reused as components.
//! - [`analysis`]: questions about a circuit: truth tables, test benches,
//!   equivalence checks.
//! - [`timing`]: gate delays for timing mode.
//! - [`export`]: Verilog and KiCad netlists for other tools.
//!
//! The engine has no I/O and no game rules. The course and the server are
//! built on top of it.

pub mod analysis;
pub mod circuit;
pub mod components;
pub mod custom;
pub mod export;
pub mod signal;
pub mod sim;
pub mod timing;
pub mod workbench;

pub use circuit::{Circuit, End, Part, PartKind, Wire};
pub use signal::Sig;
pub use sim::Sim;
