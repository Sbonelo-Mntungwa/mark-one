//! Program counter. On each rising edge: hold if EN is 0; otherwise load D
//! if LOAD is 1; otherwise count up by one, wrapping 15 to 0.

use super::clocked::{state, Clocked};
use super::{bools, from_num, norm, sigs, to_num, Model, Spec};
use crate::signal::Sig;

pub const SPEC: Spec = Spec {
    id: "pc4",
    label: "PC4",
    name: "Program counter",
    inputs: &["D0", "D1", "D2", "D3", "LOAD", "EN", "CLK"],
    outputs: &["Q0", "Q1", "Q2", "Q3"],
    sequential: true,
    transistors: 384,
    summary: "Each clock edge: hold if EN is 0, else load D if LOAD, else count up.",
};

fn next(m: &[Sig], q: &[Sig]) -> Vec<Sig> {
    match (m[5].as_bool(), m[4].as_bool()) {
        (Some(false), _) => q.to_vec(),
        (Some(true), Some(true)) => m[..4].iter().map(|&s| norm(s)).collect(),
        (Some(true), Some(false)) => match bools(q) {
            Some(q) => sigs(&from_num((to_num(&q) + 1) & 15, 4)),
            None => vec![Sig::X; 4],
        },
        _ => vec![Sig::X; 4],
    }
}

pub fn model() -> Box<dyn Model> {
    Box::new(Clocked::new(7, 4, vec![Sig::L; 4], next, state).named("PC"))
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

    fn step(m: &mut dyn crate::components::Model, d: u32, load: bool, en: bool) -> u32 {
        let mut ins = from_num(d, 4);
        ins.extend([load, en]);
        to_num(&bools(&clock(m, &ins)).unwrap())
    }

    #[test]
    fn counts_wraps_jumps_and_freezes() {
        let m = &mut *super::model();
        for want in 1..=15 {
            assert_eq!(step(m, 0, false, true), want);
        }
        assert_eq!(step(m, 0, false, true), 0, "15 wraps to 0");
        assert_eq!(step(m, 9, true, true), 9, "jump");
        assert_eq!(step(m, 3, true, false), 9, "EN=0 holds even with LOAD");
        assert_eq!(step(m, 3, false, false), 9);
        assert_eq!(step(m, 3, false, true), 10);
    }
}
