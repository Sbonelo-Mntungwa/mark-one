//! 8-bit register: on each rising edge of CLK, Q takes D if LOAD is 1.

use super::clocked::{state, Clocked};
use super::{norm, Model, Spec};
use crate::signal::Sig;

pub const SPEC: Spec = Spec {
    id: "reg8",
    label: "REG8",
    name: "Byte register",
    inputs: &[
        "D0", "D1", "D2", "D3", "D4", "D5", "D6", "D7", "LOAD", "CLK",
    ],
    outputs: &["Q0", "Q1", "Q2", "Q3", "Q4", "Q5", "Q6", "Q7"],
    sequential: true,
    transistors: 480,
    summary: "Stores 8 bits on a clock edge when LOAD is 1.",
};

fn next(m: &[Sig], q: &[Sig]) -> Vec<Sig> {
    match m[8].as_bool() {
        Some(true) => m[..8].iter().map(|&s| norm(s)).collect(),
        Some(false) => q.to_vec(),
        None => vec![Sig::X; 8],
    }
}

pub fn model() -> Box<dyn Model> {
    Box::new(Clocked::new(10, 8, vec![Sig::L; 8], next, state))
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
        for v in 0..1u32 << 8 {
            let mut d = from_num(v, 8);
            d.push(true);
            clock(&mut *m, &d);
            // Present a different value without LOAD: Q must not change.
            let mut other = from_num(!v & 255, 8);
            other.push(false);
            let q = bools(&clock(&mut *m, &other)).unwrap();
            assert_eq!(to_num(&q), v);
        }
    }
}
