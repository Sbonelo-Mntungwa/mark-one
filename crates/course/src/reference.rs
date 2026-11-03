//! The model each Mark is checked against.

use mark_one_engine::components::{self, Comb, Model};

fn f_hilo(_: &[bool]) -> Vec<bool> {
    vec![true, false]
}

fn f_relay(b: &[bool]) -> Vec<bool> {
    vec![b[0]]
}

/// The model a level is checked against: its component, or for the first two
/// Marks (plain wiring, no component) a fixed function.
pub fn model(id: &str) -> Option<Box<dyn Model>> {
    match id {
        "power" => Some(Box::new(Comb {
            n_in: 0,
            n_out: 2,
            f: f_hilo,
        })),
        "relay" => Some(Box::new(Comb {
            n_in: 1,
            n_out: 1,
            f: f_relay,
        })),
        _ => components::model(id),
    }
}
