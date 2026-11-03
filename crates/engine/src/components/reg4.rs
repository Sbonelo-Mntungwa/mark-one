//! 4-bit register: on each rising edge of CLK, Q takes D if LOAD is 1.

use super::clocked::{state, Clocked};
use super::{norm, Model, Spec};
use crate::signal::Sig;

pub const SPEC: Spec = Spec {
    id: "reg4",
    label: "REG4",
    name: "Nibble register",
    inputs: &["D0", "D1", "D2", "D3", "LOAD", "CLK"],
    outputs: &["Q0", "Q1", "Q2", "Q3"],
    sequential: true,
    transistors: 240,
    summary: "Stores 4 bits on a clock edge when LOAD is 1.",
};

fn next(m: &[Sig], q: &[Sig]) -> Vec<Sig> {
    match m[4].as_bool() {
        Some(true) => m[..4].iter().map(|&s| norm(s)).collect(),
        Some(false) => q.to_vec(),
        None => vec![Sig::X; 4],
    }
}

pub fn model() -> Box<dyn Model> {
    Box::new(Clocked::new(6, 4, vec![Sig::L; 4], next, state))
}

pub fn component() -> super::Component {
    super::Component {
        spec: SPEC,
        make: model,
    }
}

#[cfg(test)]
mod tests {
    use crate::components::testkit::*;
    use crate::components::{bools, from_num, to_num};

    #[test]
    fn every_value_loads_and_holds() {
        let mut m = super::model();
        for v in 0..1u32 << 4 {
            let mut d = from_num(v, 4);
            d.push(true);
            clock(&mut *m, &d);
            // Present a different value without LOAD: Q must not change.
            let mut other = from_num(!v & 15, 4);
            other.push(false);
            let q = bools(&clock(&mut *m, &other)).unwrap();
            assert_eq!(to_num(&q), v);
        }
    }
}
